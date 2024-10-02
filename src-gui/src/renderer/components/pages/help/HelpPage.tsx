import { Box, makeStyles } from "@material-ui/core";
import ContactInfoBox from "./ContactInfoBox";
import DonateInfoBox from "./DonateInfoBox";
import FeedbackInfoBox from "./FeedbackInfoBox";
import DaemonControlBox from "./RpcControlBox";
import SettingsBox from "./SettingsBox";

const useStyles = makeStyles((theme) => ({
  outer: {
    display: "flex",
    gap: theme.spacing(2),
    flexDirection: "column",
  },
}));

export default function HelpPage() {
  const classes = useStyles();

  return (
    <Box className={classes.outer}>
      <DaemonControlBox />
      <SettingsBox />
      <FeedbackInfoBox />
      <ContactInfoBox />
      <DonateInfoBox />
    </Box>
  );
}
