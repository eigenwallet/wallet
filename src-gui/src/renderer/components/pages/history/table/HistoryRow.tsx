import { Box, Collapse, IconButton, TableCell, TableRow } from "@mui/material";
import makeStyles from '@mui/styles/makeStyles';
import { useState } from "react";
import ArrowForwardIcon from "@mui/icons-material/ArrowForward";
import KeyboardArrowDownIcon from "@mui/icons-material/KeyboardArrowDown";
import KeyboardArrowUpIcon from "@mui/icons-material/KeyboardArrowUp";
import {
  getHumanReadableDbStateType,
  getSwapBtcAmount,
  getSwapXmrAmount,
  GetSwapInfoResponse,
} from "../../../../../models/rpcModel";
import HistoryRowActions from "./HistoryRowActions";
import HistoryRowExpanded from "./HistoryRowExpanded";
import { BitcoinAmount, MoneroAmount } from "../../../other/Units";

type HistoryRowProps = {
  swap: GetSwapInfoResponse;
};

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
      <BitcoinAmount amount={btcAmount} />
      <ArrowForwardIcon />
      <MoneroAmount amount={xmrAmount} />
    </Box>
  );
}

export default function HistoryRow({ swap }: HistoryRowProps) {
  const btcAmount = getSwapBtcAmount(swap);
  const xmrAmount = getSwapXmrAmount(swap);

  const [expanded, setExpanded] = useState(false);

  return (
    <>
      <TableRow>
        <TableCell>
          <IconButton size="small" onClick={() => setExpanded(!expanded)}>
            {expanded ? <KeyboardArrowUpIcon /> : <KeyboardArrowDownIcon />}
          </IconButton>
        </TableCell>
        <TableCell>{swap.swap_id.substring(0, 5)}...</TableCell>
        <TableCell>
          <AmountTransfer xmrAmount={xmrAmount} btcAmount={btcAmount} />
        </TableCell>
        <TableCell>{getHumanReadableDbStateType(swap.state_name)}</TableCell>
        <TableCell>
          <HistoryRowActions swap={swap} />
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
