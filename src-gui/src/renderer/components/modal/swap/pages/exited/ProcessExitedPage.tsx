import { TauriSwapProgressEvent } from "models/tauriModel";
import SwapStatePage from "../SwapStatePage";

export default function ProcessExitedPage({
  prevState,
  swapId,
}: {
  prevState: TauriSwapProgressEvent | null;
  swapId: string;
}) {
  // If we have a swap state, for a "done" state we should use it to display additional information that can't be extracted from the database
  if (
    prevState.type === "XmrRedeemInMempool" ||
    prevState.type === "BtcRefunded" ||
    prevState.type === "BtcPunished"
  ) {
    return <SwapStatePage curr={prevState} prev={null} swapId={swapId} />;
  }

  // TODO: Display something useful here
  return (
    <>
      If the swap is not a "done" state (or we don't have a db state because the
      swap did complete the SwapSetup yet) we should tell the user and show logs
      Not implemented yet
    </>
  );

  // If the swap is not a "done" state (or we don't have a db state because the swap did complete the SwapSetup yet) we should tell the user and show logs
  // return <ProcessExitedAndNotDonePage state={state} />;
}
