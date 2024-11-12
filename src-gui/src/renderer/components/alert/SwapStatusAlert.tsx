import { Box, LinearProgress, makeStyles, Paper, Tooltip } from "@material-ui/core";
import { Alert, AlertTitle } from "@material-ui/lab/";
import { ExpiredTimelocks, GetSwapInfoResponse } from "models/tauriModel";
import {
  BobStateName,
  getAbsoluteBlock,
  GetSwapInfoResponseExt,
  isGetSwapInfoResponseRunningSwap,
  TimelockCancel,
  TimelockNone,
} from "models/tauriModelExt";
import { ReactNode } from "react";
import { exhaustiveGuard } from "utils/typescriptUtils";
import HumanizedBitcoinBlockDuration from "../other/HumanizedBitcoinBlockDuration";
import TruncatedText from "../other/TruncatedText";
import {
  SwapCancelRefundButton,
  SwapResumeButton,
} from "../pages/history/table/HistoryRowActions";
import { SwapMoneroRecoveryButton } from "../pages/history/table/SwapMoneroRecoveryButton";
import { useTheme } from "@material-ui/core/styles";
import { Typography } from "@material-ui/core";

const useStyles = makeStyles({
  box: {
    display: "flex",
    flexDirection: "column",
    gap: "0.5rem",
  },
  list: {
    padding: "0px",
    margin: "0px",
  },
  alertMessage: {
    flexGrow: 1,
  },
  overlayedProgressBar: {
    borderRadius: 0,
  }
});

/**
 * Component for displaying a list of messages.
 * @param messages - Array of messages to display.
 * @returns JSX.Element
 */
function MessageList({ messages }: { messages: ReactNode[]; }) {
  const classes = useStyles();
  return (
    <ul className={classes.list}>
      {messages.map((msg, i) => (
        <li key={i}>{msg}</li>
      ))}
    </ul>
  );
}

/**
 * Sub-component for displaying alerts when the swap is in a safe state.
 * @param swap - The swap information.
 * @returns JSX.Element
 */
function BitcoinRedeemedStateAlert({ swap }: { swap: GetSwapInfoResponseExt; }) {
  const classes = useStyles();
  return (
    <Box className={classes.box}>
      <MessageList
        messages={[
          "The Bitcoin has been redeemed by the other party",
          "There is no risk of losing funds. You can take your time",
          "The Monero will be automatically redeemed to the address you provided as soon as you resume the swap",
          "If this step fails, you can manually redeem the funds",
        ]} />
      <SwapMoneroRecoveryButton swap={swap} size="small" variant="contained" />
    </Box>
  );
}

/**
 * Sub-component for displaying alerts when the swap is in a state with no timelock info.
 * @param swap - The swap information.
 * @param punishTimelockOffset - The punish timelock offset.
 * @returns JSX.Element
 */
function BitcoinLockedNoTimelockExpiredStateAlert({
  timelock, cancelTimelockOffset, punishTimelockOffset,
}: {
  timelock: TimelockNone;
  cancelTimelockOffset: number;
  punishTimelockOffset: number;
}) {
  return (
    <MessageList
      messages={[
        "Your Bitcoin have been locked",
        <>
          The swap will be refunded if it is not completed within{" "}
          <HumanizedBitcoinBlockDuration blocks={cancelTimelockOffset} />
        </>,
        "You need to have the GUI running at some point within the refund window",
        <>
          You risk loss of funds if you do not refund or complete the swap
          within{" "}
          <HumanizedBitcoinBlockDuration
            blocks={timelock.content.blocks_left + punishTimelockOffset} />
        </>,
      ]} />
  );
}

/**
 * Sub-component for displaying alerts when the swap timelock is expired
 * The swap could be cancelled but not necessarily (the transaction might not have been published yet)
 * But it doesn't matter because the swap cannot be completed anymore
 * @param swap - The swap information.
 * @returns JSX.Element
 */
function BitcoinPossiblyCancelledAlert({
  swap, timelock,
}: {
  swap: GetSwapInfoResponseExt;
  timelock: TimelockCancel;
}) {
  const classes = useStyles();
  return (
    <Box className={classes.box}>
      <MessageList
        messages={[
          "The swap was cancelled because it did not complete in time",
          "You must resume the swap immediately to refund your Bitcoin",
          <>
            You might lose your funds if you do not refund within{" "}
            <HumanizedBitcoinBlockDuration
              blocks={timelock.content.blocks_left} />
          </>,
        ]} />
    </Box>
  );
}

/**
 * Sub-component for displaying alerts requiring immediate action.
 * @returns JSX.Element
 */
function ImmediateActionAlert() {
  return (
    <>Resume the swap immediately to avoid losing your funds</>
  );
}

/**
 * Main component for displaying the appropriate swap alert status text.
 * @param swap - The swap information.
 * @returns JSX.Element | null
 */
export function SwapAlertStatusText({ swap }: { swap: GetSwapInfoResponseExt }) {
  switch (swap.state_name) {
    // This is the state where the swap is safe because the other party has redeemed the Bitcoin
    // It cannot be punished anymore
    case BobStateName.BtcRedeemed:
      return <BitcoinRedeemedStateAlert swap={swap} />;

    // These are states that are at risk of punishment because the Bitcoin have been locked
    // but has not been redeemed yet by the other party
    case BobStateName.BtcLocked:
    case BobStateName.XmrLockProofReceived:
    case BobStateName.XmrLocked:
    case BobStateName.EncSigSent:
    case BobStateName.CancelTimelockExpired:
    case BobStateName.BtcCancelled:
      if (swap.timelock != null) {
        switch (swap.timelock.type) {
          case "None":
            return (
              <BitcoinLockedNoTimelockExpiredStateAlert
                timelock={swap.timelock}
                cancelTimelockOffset={swap.cancel_timelock}
                punishTimelockOffset={swap.punish_timelock}
              />
            );
          case "Cancel":
            return (
              <BitcoinPossiblyCancelledAlert
                timelock={swap.timelock}
                swap={swap}
              />
            );
          case "Punish":
            return <ImmediateActionAlert />;
          default:
            // We have covered all possible timelock states above
            // If we reach this point, it means we have missed a case
            exhaustiveGuard(swap.timelock);
        }
      }
      return <ImmediateActionAlert />;
    default:
      // TODO: fix the exhaustive guard
      // return exhaustiveGuard(swap.state_name);
      return <></>;
  }
}

interface TimelineSegment {
  title: string;
  label: string;
  bgcolor: string;
  startBlock: number;
}

interface TimelineSegmentProps {
  segment: TimelineSegment;
  isActive: boolean;
  absoluteBlock: number;
  durationOfSegment: number | null;
  totalBlocks: number;
}

function TimelineSegment({ 
  segment, 
  isActive, 
  absoluteBlock, 
  durationOfSegment,
  totalBlocks 
}: TimelineSegmentProps) {
  const theme = useTheme();

  return (
    <Tooltip title={<Typography variant="caption">{segment.title}</Typography>}>
      <Box sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        bgcolor: segment.bgcolor,
        width: `${durationOfSegment ? ((durationOfSegment / totalBlocks) * 90) : 10}%`,
        position: 'relative',
      }} style={{
        opacity: isActive ? 1 : 0.3
      }}>
        {isActive && (
          <Box sx={{
            position: 'absolute',
            top: 0,
            left: 0,
            height: '100%',
            width: `${Math.max(2.5, ((absoluteBlock - segment.startBlock) / durationOfSegment) * 100)}%`,
            zIndex: 1,
          }}>
            <LinearProgress
              variant="indeterminate"
              color="primary"
              style={{
                height: '100%',
                backgroundColor: theme.palette.primary.dark,
                opacity: 0.3,
              }}
            />
          </Box>
        )}
        <Typography variant="subtitle2" color="inherit" align="center" style={{ zIndex: 2 }}>
          {segment.label}
        </Typography>
        {durationOfSegment && (
          <Typography 
            variant="caption" 
            color="inherit" 
            align="center" 
            style={{ 
              zIndex: 2,
              opacity: 0.8 
            }}
          >
            <HumanizedBitcoinBlockDuration 
              blocks={durationOfSegment} 
            />
          </Typography>
        )}
      </Box>
    </Tooltip>
  );
}

export function TimelockTimeline({ timelock, swap }: { 
  timelock: ExpiredTimelocks, 
  swap: GetSwapInfoResponseExt
}) {
  const theme = useTheme();

  const timelineSegments: TimelineSegment[] = [
    {
      title: "Normally a swap is completed during this period",
      label: "Normal",
      bgcolor: theme.palette.success.main,
      startBlock: 0,
    },
    {
      title: "If the swap hasn't been completed before we reach this period, the swap will be refunded. You need to have the GUI running for it to be refunded",
      label: "Refund",
      bgcolor: theme.palette.warning.main,
      startBlock: swap.cancel_timelock,
    },
    {
      title: "If you were offline for the entirety of the refund window, this period is reached. Recovery of your funds is still possible but requires cooperation from the other party",
      label: "Danger",
      bgcolor: theme.palette.error.main,
      startBlock: swap.cancel_timelock + swap.punish_timelock,
    }
  ];

  const totalBlocks = swap.cancel_timelock + swap.punish_timelock;
  const absoluteBlock = getAbsoluteBlock(timelock, swap.cancel_timelock, swap.punish_timelock);

  // This calculates the duration of a segment
  // by getting the the difference to the next segment
  function durationOfSegment(index: number): number | null {
    const nextSegment = timelineSegments[index + 1];
    if (nextSegment == null) {
      return null;
    }
    return nextSegment.startBlock - timelineSegments[index].startBlock;
  }

  // This function returns the index of the active segment based on the current block
  // We iterate in reverse to find the first segment that has a start block less than the current block
  function getActiveSegmentIndex() {
    return Array.from(timelineSegments
      .slice()
      // We use .entries() to keep the indexes despite reversing
      .entries())
      .reverse()
      .find(([_, segment]) => absoluteBlock >= segment.startBlock)?.[0] ?? 0;
  }

  return (
    <Box sx={{ 
      width: '100%', 
      minWidth: '100%',
      flexGrow: 1
    }}>
      <Paper style={{ 
        position: 'relative',
        height: '4rem',
        overflow: 'hidden',
      }} elevation={3} variant="outlined">
        <Box sx={{ 
          position: 'relative',
          height: '100%',
          display: 'flex'
        }}>
          {timelineSegments.map((segment, index) => (
            <TimelineSegment
              key={index}
              segment={segment}
              isActive={getActiveSegmentIndex() === index}
              absoluteBlock={absoluteBlock}
              durationOfSegment={durationOfSegment(index)}
              totalBlocks={totalBlocks}
            />
          ))}
        </Box>
      </Paper>
    </Box>
  );
}

/**
 * Main component for displaying the swap status alert.
 * @param swap - The swap information.
 * @returns JSX.Element | null
 */
export default function SwapStatusAlert({
  swap,
}: {
  swap: GetSwapInfoResponseExt;
}): JSX.Element | null {
  const classes = useStyles();

  // If the swap is completed, there is no need to display the alert
  // TODO: Here we should also check if the swap is in a state where any funds can be lost
  // TODO: If the no Bitcoin have been locked yet, we can safely ignore the swap
  if (!isGetSwapInfoResponseRunningSwap(swap)) {
    return null;
  }

  return (
    <Alert
      key={swap.swap_id}
      severity="warning"
      action={<SwapResumeButton swap={swap}>Resume Swap</SwapResumeButton>}
      variant="filled"
      classes={{ message: classes.alertMessage }}
    >
      <AlertTitle>
        Swap <TruncatedText>{swap.swap_id}</TruncatedText> is unfinished
      </AlertTitle>
      <SwapAlertStatusText swap={swap} />
      <TimelockTimeline timelock={swap.timelock} swap={swap} />
    </Alert>
  );
}
