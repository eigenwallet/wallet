import { Box, DialogContentText } from "@material-ui/core";
import { TauriSwapProgressEventContent } from "models/tauriModelExt";
import SwapMightBeCancelledAlert from "../../../../alert/SwapMightBeCancelledAlert";
import BitcoinTransactionInfoBox from "../../BitcoinTransactionInfoBox";
import { Alert } from "@material-ui/lab";

export default function BitcoinLockTxInMempoolPage({
  btc_lock_confirmations,
  btc_lock_txid,
}: TauriSwapProgressEventContent<"BtcLockTxInMempool">) {
  return (
    <Box style={{
      display: "flex",
      flexDirection: "column",
      gap: "1rem",
    }}>
      <Alert severity="info">
        The Bitcoin lock transaction has been published. The swap will proceed
        once the transaction is confirmed and the swap provider locks their
        Monero.
      </Alert>
      <SwapMightBeCancelledAlert />
      <BitcoinTransactionInfoBox
        title="Bitcoin Lock Transaction"
        txId={btc_lock_txid}
        loading
        additionalContent={
          <>
            Most swap providers require one confirmation before locking their
            Monero
            <br />
            Confirmations: {btc_lock_confirmations}
          </>
        }
      />
    </Box>
  );
}
