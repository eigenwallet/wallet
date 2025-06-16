import {
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Box,
  Divider,
  IconButton,
  CircularProgress,
} from "@mui/material";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import ExpandLessIcon from "@mui/icons-material/ExpandLess";
import MakerOfferItem from "../components/MakerOfferItem";
import { useState } from "react";
import { usePendingLockBitcoinApproval } from "store/hooks";
import { useActiveSwapId } from "store/hooks";
import { LockBitcoinDetails } from "models/tauriModel";

export default function Offer({
  onBack,
  onNext,
}: {
  onBack: () => void;
  onNext: () => void;
}) {
  const [feeExpanded, setFeeExpanded] = useState(false);

  const approvals = usePendingLockBitcoinApproval();
  const activeSwapId = useActiveSwapId();

  const request = approvals?.find(
    (r) => r.content.details.content.swap_id === activeSwapId,
  );

  const { btc_lock_amount, xmr_receive_amount, btc_network_fee } = (request
    ?.content.details.content as LockBitcoinDetails) || {
    btc_lock_amount: 0,
    xmr_receive_amount: 0,
    btc_network_fee: 0,
  };

  return (
    <>
      <DialogContent>
        <Typography variant="body1">
          Confirm your offer to start the swap
        </Typography>
        {request ? (
          <Box
            sx={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: 2,
              mt: 2,
            }}
          >
            <Typography variant="body1">You send</Typography>
            <Typography sx={{ textAlign: "right" }} variant="body1">
              {btc_lock_amount} BTC
            </Typography>
            <Divider sx={{ gridColumn: "span 2" }} />

            <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
              <Typography variant="body1">Fee</Typography>
              <IconButton
                onClick={() => setFeeExpanded(!feeExpanded)}
                disableFocusRipple
              >
                {feeExpanded ? <ExpandLessIcon /> : <ExpandMoreIcon />}
              </IconButton>
            </Box>
            <Typography sx={{ textAlign: "right" }} variant="body1">
              {btc_network_fee} BTC
            </Typography>
            {feeExpanded && (
              <>
                <Typography variant="body2">Network Fee</Typography>
                <Typography sx={{ textAlign: "right" }} variant="body2">
                  {request.content.details.content.btc_network_fee} BTC
                </Typography>

                {/* <Typography variant="body2">Developer Tax</Typography>
              <Typography sx={{ textAlign: "right" }} variant="body2">
                {request.content.details.content.developer_tax} BTC
              </Typography> */}
              </>
            )}
            <Divider sx={{ gridColumn: "span 2" }} />

            <Typography variant="body1">You receive</Typography>
            <Typography sx={{ textAlign: "right" }} variant="body1">
              {xmr_receive_amount} XMR
            </Typography>
          </Box>
        ) : (
          <CircularProgress />
        )}
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onBack}>
          Back
        </Button>
        <Button variant="contained" color="primary" onClick={onNext}>
          Get Offer
        </Button>
      </DialogActions>
    </>
  );
}
