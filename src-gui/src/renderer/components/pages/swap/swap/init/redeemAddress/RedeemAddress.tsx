import { Button, Typography, Box, Paper } from "@mui/material";
import ReceiveAddressSelector from "./ReceiveAddressSelector";
import { isXmrAddressValid } from "utils/conversionUtils";
import { isTestnet } from "store/config";
import { buyXmr } from "renderer/rpc";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import { useState } from "react";

export default function RedeemAddress() {
  const [redeemAddress, setRedeemAddress] = useState<string>("");

  const isValidAddress =
    redeemAddress && isXmrAddressValid(redeemAddress, isTestnet());

  async function init() {
    if (!isValidAddress) {
      return;
    }
    await buyXmr(null, redeemAddress);
  }

  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        alignItems: "flex-start",
        gap: 1,
      }}
    >
      <Typography variant="h4" sx={{ textAlign: "center", mb: 1 }}>
        Begin Swap
      </Typography>

      <Typography
        variant="body1"
        color="text.secondary"
        sx={{ textAlign: "center" }}
      >
        Enter your Monero redeem address to start swapping
      </Typography>

      <ReceiveAddressSelector
        onChange={setRedeemAddress}
        value={redeemAddress}
        isValidAddress={isValidAddress}
      />

      {isValidAddress && (
        <Typography variant="body2" color="success.dark">
          ✓ Valid Monero address
        </Typography>
      )}

      <PromiseInvokeButton
        variant="contained"
        color="primary"
        onInvoke={init}
        disabled={!isValidAddress}
        size="large"
        sx={{ mt: 2, minWidth: 200 }}
        displayErrorSnackbar
      >
        Continue
      </PromiseInvokeButton>
    </Box>
  );
}
