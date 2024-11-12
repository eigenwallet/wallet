import { makeStyles } from "@material-ui/core";
import { Alert, AlertTitle } from "@material-ui/lab";
import { useActiveSwapInfo } from "store/hooks";
import { StateAlert, TimelockTimeline } from "./SwapStatusAlert/SwapStatusAlert";

const useStyles = makeStyles((theme) => ({
  outer: {
    marginBottom: theme.spacing(1),
  },
  list: {
    margin: theme.spacing(0.25),
  },
  message: {
    flexGrow: 1,
  },
}));

export default function SwapMightBeCancelledAlert() {
  const classes = useStyles();
  const swapInfo = useActiveSwapInfo();

  // We don't have a running swap
  if(swapInfo === null) {
    return <></>;
  }

  // We don't have a timelock
  if(swapInfo.timelock === null) {
    return <></>;
  }

  return (
    <Alert severity="warning" variant="filled" classes={{
      message: classes.message,
    }}>
      <AlertTitle>
        The swap has been running for a while
      </AlertTitle>
      <StateAlert swap={swapInfo} />
      <TimelockTimeline timelock={swapInfo.timelock} swap={swapInfo} />
    </Alert>
  );
}
