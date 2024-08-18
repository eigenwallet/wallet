import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Provider } from "models/apiModel";
import { TauriSwapProgressEventWrapper } from "models/tauriModel";
import { SwapSpawnType } from "../../models/cliModel";
import { SwapSlice } from "../../models/storeModel";
import logger from "../../utils/logger";

const initialState: SwapSlice = {
  state: null,
  processRunning: false,
  swapId: null,
  logs: [],
  provider: null,
  spawnType: null,
};

export const swapSlice = createSlice({
  name: "swap",
  initialState,
  reducers: {
    swapTauriEventReceived(
      swap,
      action: PayloadAction<TauriSwapProgressEventWrapper>,
    ) {
      swap.state = action.payload.event;
      swap.swapId = action.payload.swap_id;
    },
    swapReset() {
      return initialState;
    },
    swapInitiate(
      swap,
      action: PayloadAction<{
        provider: Provider | null;
        spawnType: SwapSpawnType;
        swapId: string | null;
      }>,
    ) {
      // TOOD: Replace this functionality with tauri events
      //const nextState: SwapStateInitiated = {
      //  type: SwapStateType.INITIATED,
      //};
      //swap.state = nextState;

      swap.processRunning = true;
      swap.state = null;
      swap.logs = [];
      swap.provider = action.payload.provider;
      swap.spawnType = action.payload.spawnType;
      swap.swapId = action.payload.swapId;
    },
    swapProcessExited(swap, action: PayloadAction<string | null>) {
      if (!swap.processRunning) {
        logger.warn(`swapProcessExited called on a swap that is not running`);
        return;
      }

      /*
      const nextState: SwapStateProcessExited = {
        type: SwapStateType.PROCESS_EXITED,
        prevState: swap.state,
        rpcError: action.payload,
      };

      swap.state = nextState;*/
      swap.processRunning = false;
    },
  },
});

export const {
  swapInitiate,
  swapProcessExited,
  swapReset,
  swapTauriEventReceived,
} = swapSlice.actions;

export default swapSlice.reducer;
