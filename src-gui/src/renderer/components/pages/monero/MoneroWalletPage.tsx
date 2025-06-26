import React, { useEffect, useState } from "react";
import { Box, Typography, CircularProgress, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, Alert } from "@mui/material";
import { GetMoneroHistoryResponse, TransactionInfo, GetMoneroMainAddressResponse, GetMoneroBalanceResponse } from "models/tauriModel";
import { getMoneroHistory, getMoneroMainAddress, getMoneroBalance } from "../../../rpc";
import ActionableMonospaceTextBox from "../../other/ActionableMonospaceTextBox";
import { PiconeroAmount } from "../../other/Units";

export default function MoneroWalletPage() {
  const [history, setHistory] = useState<TransactionInfo[] | null>(null);
  const [mainAddress, setMainAddress] = useState<string | null>(null);
  const [balance, setBalance] = useState<GetMoneroBalanceResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        // throw new Error("test");
        setIsLoading(true);
        setError(null);
        /*const [historyResponse, addressResponse]: [GetMoneroHistoryResponse, GetMoneroMainAddressResponse] = await Promise.all([
          getMoneroHistory(),
          getMoneroMainAddress(),
        ]);
        setHistory(historyResponse.transactions);*/
        // Fetch all data in parallel if desired, or sequentially
        const mainAddressResponse = await getMoneroMainAddress();
        setMainAddress(mainAddressResponse.address);

        const balanceResponse = await getMoneroBalance();
        setBalance(balanceResponse);

        // History fetching can be re-enabled when stable
        // const historyResponse = await getMoneroHistory(); 
        // setHistory(historyResponse.transactions);

      } catch (err) {
        console.error("Failed to fetch Monero wallet data:", err);
        setError("Failed to fetch Monero wallet data.");
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();
  }, []);

  return (
    <Box>
      <Typography variant="h3">Monero Wallet</Typography>

      {isLoading && <CircularProgress />}
      {error && <Alert severity="error">{error}</Alert>}

      {mainAddress && (
        <Box mb={2}>
          <Typography variant="h6">Main Address:</Typography>
          <ActionableMonospaceTextBox content={mainAddress} />
        </Box>
      )}

      {balance && (
        <Box mb={2}>
          <Typography variant="h6">Balance:</Typography>
          <Typography>Total: <PiconeroAmount amount={parseFloat(balance.total_balance)} /></Typography>
          <Typography>Unlocked: <PiconeroAmount amount={parseFloat(balance.unlocked_balance)} /></Typography>
        </Box>
      )}

      {history && history.length > 0 && (
        <TableContainer component={Paper}>
          <Table aria-label="monero transaction history">
            <TableHead>
              <TableRow>
                <TableCell>Amount</TableCell>
                <TableCell>Fee</TableCell>
                <TableCell>Confirmations</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {history.map((tx, index) => (
                <TableRow key={index}>
                  <TableCell component="th" scope="row">
                    <PiconeroAmount amount={tx.amount} />
                  </TableCell>
                  <TableCell>
                    <PiconeroAmount amount={tx.fee} />
                  </TableCell>
                  <TableCell>{tx.block_height}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}

      {history && history.length === 0 && !isLoading && !error && (
        <Typography>No transactions found.</Typography>
      )}
    </Box>
  );
} 