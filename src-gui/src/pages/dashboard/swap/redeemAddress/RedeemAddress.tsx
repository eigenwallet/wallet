import { Button, Typography, Box, Paper } from "@mui/material";
import ReceiveAddressSelector from "../components/ReceiveAddressSelector";
import { useAppSelector } from "store/hooks";
import { isXmrAddressValid } from "utils/conversionUtils";
import { isTestnet } from "store/config";

export default function RedeemAddress({ onNext }: { onNext: () => void }) {
  const redeemAddress = useAppSelector(
    (state) => state.startSwap.redeemAddress,
  );
  const isValidAddress =
    redeemAddress && isXmrAddressValid(redeemAddress, isTestnet());

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

      <ReceiveAddressSelector />

      {isValidAddress && (
        <Typography variant="body2" color="success.dark">
          ✓ Valid Monero address
        </Typography>
      )}

      <Button
        variant="contained"
        color="primary"
        onClick={onNext}
        disabled={!isValidAddress}
        size="large"
        sx={{ mt: 2, minWidth: 200 }}
      >
        Continue
      </Button>
    </Box>
  );
}
