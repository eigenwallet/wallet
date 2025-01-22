import { Box } from "@mui/material";
import ContactInfoBox from "./ContactInfoBox";
import DonateInfoBox from "./DonateInfoBox";
import DaemonControlBox from "./DaemonControlBox";
import SettingsBox from "./SettingsBox";
import ExportDataBox from "./ExportDataBox";
import DiscoveryBox from "./DiscoveryBox";
import { useLocation } from "react-router-dom";
import { useEffect } from "react";
import { makeStyles } from "@mui/styles";

const useStyles = makeStyles((theme) => ({
  outer: {
    display: "flex",
    gap: theme.spacing(2),
    flexDirection: "column",
    paddingBottom: theme.spacing(2),
  },
}));

export default function SettingsPage() {
  const location = useLocation();

  useEffect(() => {
    if (location.hash) {
      const element = document.getElementById(location.hash.slice(1));
      element?.scrollIntoView({ behavior: "smooth" });
    }
  }, [location]);

  return (
    <Box
      sx={{
        display: "flex",
        gap: 2,
        flexDirection: "column",
        paddingBottom: 2,
      }}
    >
      <SettingsBox />
      <DiscoveryBox />
      <ExportDataBox />
      <DaemonControlBox />
      <DonateInfoBox />
    </Box>
  );
}
