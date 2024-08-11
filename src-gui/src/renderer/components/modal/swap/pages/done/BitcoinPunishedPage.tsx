import { Box, DialogContentText } from "@mui/material";
import FeedbackInfoBox from "../../../../pages/help/FeedbackInfoBox";

export default function BitcoinPunishedPage() {
  return (
    <Box>
      <DialogContentText>
        Unfortunately, the swap was not successful, and you&apos;ve incurred a
        penalty because the swap was not refunded in time. Both the Bitcoin and
        Monero are irretrievable.
      </DialogContentText>
      <FeedbackInfoBox />
    </Box>
  );
}
