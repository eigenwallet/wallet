import { Button, Typography, Box, Paper, Divider } from "@mui/material";
import BitcoinQrCode from "renderer/components/pages/swap/swap/components/BitcoinQrCode";
import ActionableMonospaceTextBox from "renderer/components/other/ActionableMonospaceTextBox";
import MakerOfferItem from "./MakerOfferItem";
import { useAppDispatch } from "store/hooks";
import { usePendingSelectMakerApproval } from "store/hooks";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import { resolveApproval } from "renderer/rpc";
import MakerDiscoveryStatus from "./MakerDiscoveryStatus";
import { swapReset } from "store/features/swapSlice";
import { TauriSwapProgressEventContent } from "models/tauriModelExt";
import { useState } from "react";
import { SatsAmount } from "renderer/components/other/Units";

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
          sx={{ 
            padding: 2, 
            display: "flex", 
            flexDirection: { xs: "column", md: "row" }, 
            gap: 2 
          }}
        >
          <Box sx={{ flexGrow: 1, flexShrink: 0, minWidth: "12em" }}>
            <Typography variant="body1">Bitcoin Balance</Typography>
            <Typography variant="h5">
              <SatsAmount amount={max_giveable} />
            </Typography>
          </Box>

          <Divider 
            orientation="vertical" 
            flexItem 
            sx={{ 
              marginX: { xs: 0, md: 1 }, 
              marginY: { xs: 1, md: 0 },
              display: { xs: "none", md: "block" }
            }} 
          />
          <Divider 
            orientation="horizontal" 
            flexItem 
            sx={{ 
              marginX: { xs: 0, md: 1 }, 
              marginY: { xs: 1, md: 0 },
              display: { xs: "block", md: "none" }
            }} 
          />

          <Box
            sx={{
              flexShrink: 1,
              display: "flex",
              flexDirection: { xs: "row", md: "column", lg: "row" },
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
              <Typography variant="h5">Select an offer</Typography>
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
      </Box>
    </>
  );
}
