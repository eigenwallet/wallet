import {
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Box,
  Divider,
  IconButton,
} from "@mui/material";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import ExpandLessIcon from "@mui/icons-material/ExpandLess";
import MakerOfferItem from "../components/MakerOfferItem";
import { useState } from "react";

export default function Offer({
  onBack,
  onNext,
}: {
  onBack: () => void;
  onNext: () => void;
}) {
  const [feeExpanded, setFeeExpanded] = useState(false);
  return (
    <>
      <DialogContent>
        <Typography variant="body1">
          Review and confirm your swap offer
        </Typography>

        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Selected Maker:
        </Typography>
        <MakerOfferItem />

        <Box
          sx={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: 2,
            mt: 3,
          }}
        >
          <Typography variant="body1" sx={{ fontWeight: "bold" }}>
            You send
          </Typography>
          <Typography
            sx={{ textAlign: "right", fontWeight: "bold" }}
            variant="body1"
          >
            0.00250 BTC
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
            0.00003 BTC
          </Typography>
          {feeExpanded && (
            <>
              <Typography variant="body2">Bitcoin Transaction Fee</Typography>
              <Typography sx={{ textAlign: "right" }} variant="body2">
                0.00001 BTC
              </Typography>

              <Typography variant="body2">Exchange Fee</Typography>
              <Typography sx={{ textAlign: "right" }} variant="body2">
                0.00001 BTC
              </Typography>

              <Typography variant="body2">Monero Transaction Fee</Typography>
              <Typography sx={{ textAlign: "right" }} variant="body2">
                0.0001 XMR
              </Typography>

              <Typography variant="body2">Maker Fee</Typography>
              <Typography sx={{ textAlign: "right" }} variant="body2">
                0.0003 XMR
              </Typography>
            </>
          )}
          <Divider sx={{ gridColumn: "span 2" }} />

          <Typography variant="body1" sx={{ fontWeight: "bold" }}>
            You receive
          </Typography>
          <Typography
            sx={{ textAlign: "right", fontWeight: "bold" }}
            variant="body1"
          >
            0.1045 XMR
          </Typography>
        </Box>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onBack}>
          Back
        </Button>
        <Button variant="contained" color="primary" onClick={onNext}>
          Confirm Swap
        </Button>
      </DialogActions>
    </>
  );
}
