import {
  Button,
  Typography,
  Box,
  Paper,
  Divider,
  Chip,
  List,
  ListItem,
  ListItemButton,
} from "@mui/material";
import BitcoinQrCode from "renderer/components/modal/swap/BitcoinQrCode";
import ActionableMonospaceTextBox from "renderer/components/other/ActionableMonospaceTextBox";
import MakerOfferItem from "../components/MakerOfferItem";
import { useState } from "react";
import FiatPriceLabel from "../components/FiatPriceLabel";
import { Currency } from "../components/FiatPriceLabel";
import ReceiveAddressSelector from "../components/ReceiveAddressSelector";

// Dummy data for demonstration
const DUMMY_WALLET_BALANCE = 0.0025; // BTC
const DUMMY_DEPOSIT_ADDRESS = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";
const DUMMY_MAKERS = [
  {
    id: "1",
    name: "Maria Mitchell",
    multiAddr: "fjklsdfjlfdk",
    activeFor: "10 minutes",
    fee: "0.12%",
    minMax: "0.00003 – 0.00500",
    rate: "0.04 XMR",
  },
  {
    id: "2",
    name: "Alice Johnson",
    multiAddr: "abc123defg456",
    activeFor: "5 minutes",
    fee: "0.15%",
    minMax: "0.00005 – 0.01000",
    rate: "0.045 XMR",
  },
  {
    id: "3",
    name: "Bob Smith",
    multiAddr: "xyz789uvw012",
    activeFor: "15 minutes",
    fee: "0.10%",
    minMax: "0.00002 – 0.00300",
    rate: "0.042 XMR",
  },
];

export default function WalletAndMakers({
  onNext,
  onBack,
}: {
  onNext: () => void;
  onBack: () => void;
}) {
  return (
    <>
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          gap: 3,
        }}
      >
        <ReceiveAddressSelector />
        {/* Wallet Balance Section */}
        <Paper
          elevation={8}
          sx={{ padding: 2, display: "flex", flexDirection: "row", gap: 2 }}
        >
          <Box sx={{ flexGrow: 1, flexShrink: 0, minWidth: "12em" }}>
            <Typography variant="body1">Bitcoin Balance</Typography>

            <Box sx={{ padding: 4, paddingLeft: 0 }}>
              <Typography variant="h4">{DUMMY_WALLET_BALANCE} BTC</Typography>

              <FiatPriceLabel
                amount={DUMMY_WALLET_BALANCE}
                originalCurrency={Currency.BTC}
              />
            </Box>
          </Box>

          <Divider orientation="vertical" flexItem sx={{ marginX: 1 }} />

          <Box
            sx={{
              flexShrink: 1,
              display: "flex",
              flexDirection: "row",
              gap: 2,
            }}
          >
            <Box
              sx={{
                display: "flex",
                flexDirection: "column",
                gap: 1,
              }}
            >
              <Typography variant="body1">Deposit</Typography>
              <Typography variant="body2" color="text.secondary">
                Send Bitcoin to your internal wallet to swap your desired amount
                of Monero
              </Typography>
              <ActionableMonospaceTextBox content={DUMMY_DEPOSIT_ADDRESS} />
            </Box>
            <Box
              sx={{
                display: "flex",
                justifyContent: "center",
                width: "100%",
                maxWidth: "8em",
              }}
            >
              <BitcoinQrCode address={DUMMY_DEPOSIT_ADDRESS} />
            </Box>
          </Box>
        </Paper>

        {/* Available Makers Section */}
        <Box>
          <Box
            sx={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              mb: 2,
            }}
          >
            <Box>
              <Typography variant="h4">Available Makers</Typography>
              <Typography variant="body2" color="text.secondary">
                Current network fee: 0.12%
              </Typography>
            </Box>
            <Chip
              label={`${DUMMY_MAKERS.length} online`}
              color="success"
              size="small"
            />
          </Box>
          <Box
            sx={{
              display: "flex",
              flexDirection: "column",
              gap: 2,
              overflowY: "scroll",
              maxHeight: "18em",
            }}
          >
            {DUMMY_MAKERS.map((maker) => (
              <MakerOfferItem />
            ))}
          </Box>
        </Box>
      </Box>
    </>
  );
}
