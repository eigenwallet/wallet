import { Box } from "@material-ui/core";
import { useAppSelector } from "store/hooks";
import BitcoinLockTxInMempoolPage from "./in_progress/BitcoinLockTxInMempoolPage";
import StartedPage from "./in_progress/StartedPage";
import XmrLockTxInMempoolPage from "./in_progress/XmrLockInMempoolPage";
import InitiatedPage from "./init/InitiatedPage";
import WaitingForBitcoinDepositPage from "./init/WaitingForBitcoinDepositPage";
// eslint-disable-next-line import/no-cycle
import { TauriSwapProgressEvent } from "models/tauriModel";
import BitcoinPunishedPage from "./done/BitcoinPunishedPage";
import BitcoinRefundedPage from "./done/BitcoinRefundedPage";
import XmrRedeemInMempoolPage from "./done/XmrRedeemInMempoolPage";
import BitcoinCancelledPage from "./in_progress/BitcoinCancelledPage";
import BitcoinRedeemedPage from "./in_progress/BitcoinRedeemedPage";
import ReceivedQuotePage from "./in_progress/ReceivedQuotePage";
import { SyncingMoneroWalletPage } from "./in_progress/SyncingMoneroWalletPage";
import XmrLockedPage from "./in_progress/XmrLockedPage";
import InitPage from "./init/InitPage";

export default function SwapStatePage({
  swapState,
}: {
  swapState: TauriSwapProgressEvent;
}) {
  const isSyncingMoneroWallet = useAppSelector(
    (state) => state.rpc.state.moneroWallet.isSyncing,
  );

  if (isSyncingMoneroWallet) {
    return <SyncingMoneroWalletPage />;
  }

  if (swapState === null) {
    return <InitPage />;
  }
  if (swapState.type === "Initiated") {
    return <InitiatedPage />;
  }
  if (swapState.type === "ReceivedQuote") {
    return <ReceivedQuotePage />;
  }
  if (swapState.type === "WaitingForBtcDeposit") {
    return <WaitingForBitcoinDepositPage {...swapState.content} />;
  }

  if (swapState.type === "Started") {
    return <StartedPage {...swapState.content} />;
  }
  if (swapState.type === "BtcLockTxInMempool") {
    return <BitcoinLockTxInMempoolPage {...swapState.content} />;
  }
  if (swapState.type === "XmrLockTxInMempool") {
    return <XmrLockTxInMempoolPage {...swapState.content} />;
  }
  if (swapState.type === "XmrLocked") {
    return <XmrLockedPage />;
  }
  if (swapState.type === "BtcRedeemed") {
    return <BitcoinRedeemedPage />;
  }
  if (swapState.type === "XmrRedeemInMempool") {
    return <XmrRedeemInMempoolPage {...swapState.content} />;
  }
  if (swapState.type === "BtcCancelled") {
    return <BitcoinCancelledPage />;
  }
  if (swapState.type === "BtcRefunded") {
    return <BitcoinRefundedPage {...swapState.content} />;
  }
  if (swapState.type === "BtcPunished") {
    return <BitcoinPunishedPage />;
  }

  /*
  TODO: Implement this page
  if (isSwapStateProcessExited(swapState)) {
    return <ProcessExitedPage state={swapState} />;
  }
  */

  // TODO: Implement cooperative redeem attempt/reject page here

  console.error(
    `No swap state page found for swap state State: ${JSON.stringify(
      swapState,
      null,
      4,
    )}`,
  );
  return (
    <Box>
      No information to display
      <br />
      State: ${JSON.stringify(swapState, null, 4)}
    </Box>
  );
}
