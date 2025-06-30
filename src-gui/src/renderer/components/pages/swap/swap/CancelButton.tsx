import { Box, Button } from "@mui/material";
import { haveFundsBeenLocked } from "models/tauriModelExt";
import { getCurrentSwapId, suspendCurrentSwap } from "renderer/rpc";
import { swapReset } from "store/features/swapSlice";
import { useAppDispatch, useAppSelector, useIsSwapRunning } from "store/hooks";
import { useState } from "react";
import SwapSuspendAlert from "renderer/components/modal/SwapSuspendAlert";

export default function CancelButton() {
  const swap = useAppSelector((state) => state.swap);
  const [openSuspendAlert, setOpenSuspendAlert] = useState(false);
  const dispatch = useAppDispatch();
  const isSwapRunning = useIsSwapRunning();

  async function onCancel() {
    const swapId = await getCurrentSwapId();

    if (swapId !== null) {
      if (haveFundsBeenLocked(swap.state.curr) && isSwapRunning) {
        setOpenSuspendAlert(true);
      } else {
        await suspendCurrentSwap();
        dispatch(swapReset());
      }
    } else {
      dispatch(swapReset());
    }
  }

  return (
    <>
      <SwapSuspendAlert
        open={openSuspendAlert}
        onClose={() => setOpenSuspendAlert(false)}
      />
      <Box
        sx={{ display: "flex", justifyContent: "flex-start", width: "100%" }}
      >
        <Button variant="outlined" onClick={onCancel}>
          Cancel
        </Button>
      </Box>
    </>
  );
}
