import { Button, Typography, Box, Paper, Divider, Chip } from "@mui/material";
import BitcoinQrCode from "renderer/components/pages/swap/swap/components/BitcoinQrCode";
import ActionableMonospaceTextBox from "renderer/components/other/ActionableMonospaceTextBox";
import MakerOfferItem from "./MakerOfferItem";
import FiatPriceLabel from "../../components/FiatPriceLabel";
import { Currency } from "../../components/FiatPriceLabel";
import { useAppDispatch } from "store/hooks";
import { usePendingSelectMakerApproval } from "store/hooks";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import { resolveApproval } from "renderer/rpc";
import { satsToBtc } from "utils/conversionUtils";
import MakerDiscoveryStatus from "./MakerDiscoveryStatus";
import { swapReset } from "store/features/swapSlice";
import { TauriSwapProgressEventContent } from "models/tauriModelExt";
import { useState } from "react";

export default function WalletAndMakers({
  deposit_address,
  min_bitcoin_lock_tx_fee,
  max_giveable,
}: TauriSwapProgressEventContent<"WaitingForBtcDeposit">) {
  const dispatch = useAppDispatch();
  const pendingSelectMakerApprovals = usePendingSelectMakerApproval();
  const [selectedMakerRequestId, setSelectedMakerRequestId] = useState<
    string | null
  >(null);

  return (
    <>
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          gap: 3,
        }}
      >
        <Paper
          elevation={8}
          sx={{ padding: 2, display: "flex", flexDirection: "row", gap: 2 }}
        >
          <Box sx={{ flexGrow: 1, flexShrink: 0, minWidth: "12em" }}>
            <Typography variant="body1">Bitcoin Balance</Typography>

            <Box sx={{ padding: 4, paddingLeft: 0 }}>
              <Typography variant="h4">
                {satsToBtc(max_giveable)} BTC
              </Typography>

              <FiatPriceLabel
                amount={satsToBtc(max_giveable)}
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
              <ActionableMonospaceTextBox content={deposit_address} />
            </Box>
            <Box
              sx={{
                display: "flex",
                justifyContent: "center",
                width: "100%",
                maxWidth: "8em",
              }}
            >
              <BitcoinQrCode address={deposit_address} />
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
                Current network fee: {min_bitcoin_lock_tx_fee} sats/vbyte
              </Typography>
            </Box>
            <Chip
              label={`${pendingSelectMakerApprovals.length} online`}
              color="success"
              size="small"
            />
          </Box>

          {/* Maker Discovery Status */}
          <MakerDiscoveryStatus />

          {/* Real Maker Offers */}
          <Box>
            {pendingSelectMakerApprovals.length > 0 && (
              <Box sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
                {pendingSelectMakerApprovals.map((makerApproval, index) => {
                  return (
                    <MakerOfferItem
                      key={index}
                      makerApproval={makerApproval}
                      selectedMakerRequestId={selectedMakerRequestId}
                      onSelect={setSelectedMakerRequestId}
                    />
                  );
                })}
              </Box>
            )}

            {pendingSelectMakerApprovals.length === 0 && (
              <Paper variant="outlined" sx={{ p: 3, textAlign: "center" }}>
                <Typography variant="body1" color="textSecondary">
                  Searching for available makers...
                </Typography>
                <Typography
                  variant="body2"
                  color="textSecondary"
                  sx={{ mt: 1 }}
                >
                  Please wait while we find the best offers for your swap.
                </Typography>
              </Paper>
            )}
          </Box>
        </Box>
      </Box>

      {/* Navigation Buttons */}
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          mt: 4,
          pt: 2,
          borderTop: 1,
          borderColor: "divider",
        }}
      >
        <Button
          variant="outlined"
          onClick={() => dispatch(swapReset())}
          sx={{ minWidth: 120 }}
        >
          Cancel
        </Button>
        <PromiseInvokeButton
          variant="contained"
          disabled={!selectedMakerRequestId}
          onInvoke={() => resolveApproval(selectedMakerRequestId, true)}
          displayErrorSnackbar
          sx={{ minWidth: 120 }}
        >
          Continue
        </PromiseInvokeButton>
      </Box>
    </>
  );
}
