import { Box, makeStyles } from "@material-ui/core";
import { Alert, AlertTitle } from "@material-ui/lab";
import { useActiveSwapInfo } from "store/hooks";
import { StateAlert } from "./SwapStatusAlert/SwapStatusAlert";
import { TimelockTimeline } from "./SwapStatusAlert/TimelockTimeline";
import { isGetSwapInfoResponseRunningSwap, isGetSwapInfoResponseWithTimelock } from "models/tauriModelExt";

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
  box: {
    display: "flex",
    flexDirection: "column",
    gap: theme.spacing(1),
  },
}));

export default function SwapMightBeCancelledAlert() {
  const classes = useStyles();
  const swapInfo = useActiveSwapInfo();

  if (!isGetSwapInfoResponseWithTimelock(swapInfo)) {
    return <></>;
  }

  if(!isGetSwapInfoResponseRunningSwap(swapInfo)) {
    return <></>;
  }

  return (
    <Alert severity="warning" variant="filled" classes={{
      message: classes.message,
    }}>
      <AlertTitle>
        The swap has been running for a while
      </AlertTitle>
      <Box className={classes.box}>
        <StateAlert swap={swapInfo} />
        <TimelockTimeline swap={swapInfo} />
      </Box>
    </Alert>
  );
}
