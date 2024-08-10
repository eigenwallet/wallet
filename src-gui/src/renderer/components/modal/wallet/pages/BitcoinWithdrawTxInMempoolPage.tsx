import { Button, DialogActions, DialogContentText } from "@mui/material";
import BitcoinTransactionInfoBox from "../../swap/BitcoinTransactionInfoBox";
import WithdrawDialogContent from "../WithdrawDialogContent";

export default function BtcTxInMempoolPageContent({
  withdrawTxId,
  onCancel,
}: {
  withdrawTxId: string;
  onCancel: () => void;
}) {
  return (
    <>
      <DialogContentText>
        All funds of the internal Bitcoin wallet have been transferred to your
        withdraw address.
      </DialogContentText>
      <BitcoinTransactionInfoBox
        txId={withdrawTxId}
        loading={false}
        title="Bitcoin Withdraw Transaction"
        additionalContent={null}
      />
    </>
  );
}
