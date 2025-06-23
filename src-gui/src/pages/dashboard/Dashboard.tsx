import { Box } from "@mui/material";
import ApiAlertsBox from "renderer/components/pages/swap/ApiAlertsBox";
import SwapWidget from "./swap/SwapWidget";

export default function Dashboard() {
  return (
    <Box
      sx={{
        display: "flex",
        width: "100%",
        flexDirection: "column",
        alignItems: "center",
        paddingBottom: 1,
        gap: 1,
      }}
    >
      <ApiAlertsBox />
      <SwapWidget />
    </Box>
  );
}
