import { Typography } from "@mui/material";
import { useAppSelector } from "store/hooks";
import SwapTxLockAlertsBox from "../../alert/SwapTxLockAlertsBox";
import SwapDialog from "../../modal/swap/SwapDialog";
import HistoryTable from "./table/HistoryTable";

export default function HistoryPage() {
  return (
    <>
      <Typography variant="h3">History</Typography>
      <SwapTxLockAlertsBox />
      <HistoryTable />
    </>
  );
}
