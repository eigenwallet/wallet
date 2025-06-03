import { Box } from "@mui/material";
import makeStyles from '@mui/styles/makeStyles';
import FeedbackInfoBox from "../help/FeedbackInfoBox";
import ConversationsBox from "../help/ConversationsBox";
import ContactInfoBox from "../help/ContactInfoBox";

const useStyles = makeStyles((theme) => ({
  outer: {
    display: "flex",
    gap: theme.spacing(2),
    flexDirection: "column",
    paddingBottom: theme.spacing(2),
  },
}));

export default function FeedbackPage() {
  const classes = useStyles();

  return (
    <Box className={classes.outer}>
      <FeedbackInfoBox />
      <ConversationsBox />
      <ContactInfoBox />
    </Box>
  );
} 