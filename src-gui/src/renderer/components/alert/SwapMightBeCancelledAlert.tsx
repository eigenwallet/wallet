import { useActiveSwapInfo } from "store/hooks";
import SwapStatusAlert from "./SwapStatusAlert/SwapStatusAlert";
import { isGetSwapInfoResponseWithTimelock } from "models/tauriModelExt";

export default function SwapMightBeCancelledAlert() {
  const swapInfo = useActiveSwapInfo();

  // If the swap does not have a timelock, we cannot display the alert
  if (!isGetSwapInfoResponseWithTimelock(swapInfo)) {
    return null;
  }

  return (
    <SwapStatusAlert swap={swapInfo} isRunning={true} />
  );
}
