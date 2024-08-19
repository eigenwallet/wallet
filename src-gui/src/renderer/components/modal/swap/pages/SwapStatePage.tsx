import { Box } from "@material-ui/core";
import { SwapSlice } from "models/storeModel";
import BitcoinPunishedPage from "./done/BitcoinPunishedPage";
import BitcoinRefundedPage from "./done/BitcoinRefundedPage";
import XmrRedeemInMempoolPage from "./done/XmrRedeemInMempoolPage";
import ProcessExitedPage from "./exited/ProcessExitedPage";
import BitcoinCancelledPage from "./in_progress/BitcoinCancelledPage";
import BitcoinLockTxInMempoolPage from "./in_progress/BitcoinLockTxInMempoolPage";
import BitcoinRedeemedPage from "./in_progress/BitcoinRedeemedPage";
import ReceivedQuotePage from "./in_progress/ReceivedQuotePage";
import StartedPage from "./in_progress/StartedPage";
import XmrLockedPage from "./in_progress/XmrLockedPage";
import XmrLockTxInMempoolPage from "./in_progress/XmrLockInMempoolPage";
import InitiatedPage from "./init/InitiatedPage";
import InitPage from "./init/InitPage";
import WaitingForBitcoinDepositPage from "./init/WaitingForBitcoinDepositPage";

export default function SwapStatePage({
  state,
}: {
  state: SwapSlice["state"];
}) {
  // TODO: Reimplement this using tauri events
  /*
  const isSyncingMoneroWallet = useAppSelector(
    (state) => state.rpc.state.moneroWallet.isSyncing,
  );

  if (isSyncingMoneroWallet) {
    return <SyncingMoneroWalletPage />;
  }
  */

  if (state === null) {
    return <InitPage />;
  }
  if (state.curr.type === "Initiated") {
    return <InitiatedPage />;
  }
  if (state.curr.type === "ReceivedQuote") {
    return <ReceivedQuotePage />;
  }
  if (state.curr.type === "WaitingForBtcDeposit") {
    return <WaitingForBitcoinDepositPage {...state.curr.content} />;
  }
  if (state.curr.type === "Started") {
    return <StartedPage {...state.curr.content} />;
  }
  if (state.curr.type === "BtcLockTxInMempool") {
    return <BitcoinLockTxInMempoolPage {...state.curr.content} />;
  }
  if (state.curr.type === "XmrLockTxInMempool") {
    return <XmrLockTxInMempoolPage {...state.curr.content} />;
  }
  if (state.curr.type === "XmrLocked") {
    return <XmrLockedPage />;
  }
  if (state.curr.type === "BtcRedeemed") {
    return <BitcoinRedeemedPage />;
  }
  if (state.curr.type === "XmrRedeemInMempool") {
    return <XmrRedeemInMempoolPage {...state.curr.content} />;
  }
  if (state.curr.type === "BtcCancelled") {
    return <BitcoinCancelledPage />;
  }
  if (state.curr.type === "BtcRefunded") {
    return <BitcoinRefundedPage {...state.curr.content} />;
  }
  if (state.curr.type === "BtcPunished") {
    return <BitcoinPunishedPage />;
  }
  if (state.curr.type === "Released") {
    return <ProcessExitedPage prevState={state.prev} swapId={state.swapId} />;
  }

  // TODO: Implement cooperative redeem attempt/reject page here

  return (
    <Box>
      No information to display
      <br />
      State: ${JSON.stringify(state, null, 4)}
    </Box>
  );
}
