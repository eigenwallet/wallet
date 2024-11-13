import { Box, makeStyles } from "@material-ui/core";
import { Alert, AlertTitle } from "@material-ui/lab";
import { useActiveSwapInfo } from "store/hooks";
import SwapStatusAlert, { StateAlert } from "./SwapStatusAlert/SwapStatusAlert";
import { TimelockTimeline } from "./SwapStatusAlert/TimelockTimeline";
import { getAbsoluteBlock, isGetSwapInfoResponseRunningSwap, isGetSwapInfoResponseWithTimelock } from "models/tauriModelExt";

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

// This is the number of blocks after which we consider the swap to be at risk of being unsuccessful
const BITCOIN_CONFIRMATIONS_WARNING_THRESHOLD = 5;

export default function SwapMightBeCancelledAlert() {
  const classes = useStyles();
  const swapInfo = useActiveSwapInfo();

  // If the swap does not have a timelock, we cannot display the alert
  if (!isGetSwapInfoResponseWithTimelock(swapInfo)) {
    return null;
  }

  // If the swap has not been running for long enough, we do not need to display the alert
  // The swap is probably gonna be successful
  // No need to spook the user for no reason
  if(getAbsoluteBlock(swapInfo.timelock, swapInfo.cancel_timelock, swapInfo.punish_timelock) < BITCOIN_CONFIRMATIONS_WARNING_THRESHOLD) {
    return null;
  }

  return (
    <SwapStatusAlert swap={swapInfo} />
  );
}
