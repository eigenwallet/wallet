import { IconButton } from "@mui/material";
import FeedbackIcon from "@mui/icons-material/Feedback";
import FeedbackDialog from "../../feedback/FeedbackDialog";
import { useState } from "react";

export default function FeedbackSubmitBadge() {
  const [showFeedbackDialog, setShowFeedbackDialog] = useState(false);

  return <>
    {showFeedbackDialog && (
      <FeedbackDialog
        open={showFeedbackDialog}
        onClose={() => setShowFeedbackDialog(false)}
      />
    )}
    <IconButton onClick={() => setShowFeedbackDialog(true)} size="large">
      <FeedbackIcon />
    </IconButton>
  </>;
}
