import { Box, DialogContentText } from "@material-ui/core";
import { TauriSwapProgressEventContent } from "models/tauriModelExt";
import { useActiveSwapInfo } from "store/hooks";
import FeedbackInfoBox from "../../../../pages/help/FeedbackInfoBox";
import BitcoinTransactionInfoBox from "../../BitcoinTransactionInfoBox";

export default function BitcoinRefundedPage({
  btc_refund_txid,
  btc_refund_finalized,
}: TauriSwapProgressEventContent<"BtcRefunded">) {
  const swap = useActiveSwapInfo();
  const additionalContent = swap
    ? <>
        {!btc_refund_finalized && "Waiting for refund transaction to be confirmed"}
        {!btc_refund_finalized && <br />}
        Refund address: ${swap.btc_refund_address}
      </>
    : null;

  return (
    <Box>
      <DialogContentText>
        Unfortunately, the swap was not successful. However, rest assured that
        all your Bitcoin has been refunded to the specified address. The swap
        process is now complete, and you are free to exit the application.
      </DialogContentText>
      <Box
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "0.5rem",
        }}
      >
        <BitcoinTransactionInfoBox
          title="Bitcoin Refund Transaction"
          txId={btc_refund_txid}
          loading={!btc_refund_finalized}
          additionalContent={additionalContent}
        />
        <FeedbackInfoBox />
      </Box>
    </Box>
  );
}
