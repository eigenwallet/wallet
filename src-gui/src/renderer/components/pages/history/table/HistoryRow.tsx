import { Box, Collapse, IconButton, TableCell, TableRow } from "@mui/material";
import makeStyles from "@mui/styles/makeStyles";
import ArrowForwardIcon from "@mui/icons-material/ArrowForward";
import KeyboardArrowDownIcon from "@mui/icons-material/KeyboardArrowDown";
import KeyboardArrowUpIcon from "@mui/icons-material/KeyboardArrowUp";
import { GetSwapInfoResponse } from "models/tauriModel";
import { useState } from "react";
import { PiconeroAmount, SatsAmount } from "../../../other/Units";
import HistoryRowActions from "./HistoryRowActions";
import HistoryRowExpanded from "./HistoryRowExpanded";

const useStyles = makeStyles((theme) => ({
  amountTransferContainer: {
    display: "flex",
    alignItems: "center",
    gap: theme.spacing(1),
  },
}));

function AmountTransfer({
  btcAmount,
  xmrAmount,
}: {
  xmrAmount: number;
  btcAmount: number;
}) {
  const classes = useStyles();

  return (
    <Box className={classes.amountTransferContainer}>
      <SatsAmount amount={btcAmount} />
      <ArrowForwardIcon />
      <PiconeroAmount amount={xmrAmount} />
    </Box>
  );
}

export default function HistoryRow(swap: GetSwapInfoResponse) {
  const [expanded, setExpanded] = useState(false);

  return (
    <>
      <TableRow>
        <TableCell>
          <IconButton size="small" onClick={() => setExpanded(!expanded)}>
            {expanded ? <KeyboardArrowUpIcon /> : <KeyboardArrowDownIcon />}
          </IconButton>
        </TableCell>
        <TableCell>{swap.swap_id}</TableCell>
        <TableCell>
          <AmountTransfer
            xmrAmount={swap.xmr_amount}
            btcAmount={swap.btc_amount}
          />
        </TableCell>
        <TableCell>{swap.state_name.toString()}</TableCell>
        <TableCell>
          <HistoryRowActions {...swap} />
        </TableCell>
      </TableRow>

      <TableRow>
        <TableCell style={{ padding: 0 }} colSpan={6}>
          <Collapse in={expanded} timeout="auto">
            {expanded && <HistoryRowExpanded swap={swap} />}
          </Collapse>
        </TableCell>
      </TableRow>
    </>
  );
}
