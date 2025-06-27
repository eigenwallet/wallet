import { Paper } from "@mui/material";
import { useAppSelector } from "store/hooks";
import SwapStatePage from "renderer/components/pages/swap/swap/SwapStatePage";
import CancelButton from "./CancelButton";

export default function SwapWidget() {
  const swap = useAppSelector((state) => state.swap);

  return (
    <Paper
      elevation={3}
      sx={{
        width: "100%",
        maxWidth: 800,
        margin: "0 auto",
        borderRadius: 2,
        padding: 2,
      }}
    >
      <SwapStatePage state={swap.state} />
    </Paper>
  );
}
