import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { TauriSettings } from "models/tauriModel";

const initialState: TauriSettings = {
  bitcoin_confirmation_target: 1,
  electrum_rpc_url: null,
  monero_node_url: null,
};

const alertsSlice = createSlice({
  name: "settings",
  initialState,
  reducers: {
    setBitcoinConfirmationTarget(slice, action: PayloadAction<number>) {
      slice.bitcoin_confirmation_target = action.payload;
    },
    setElectrumRpcUrl(slice, action: PayloadAction<string | null>) {
      if (action.payload.length === 0) {
        slice.electrum_rpc_url = null;
      } else {
        slice.electrum_rpc_url = action.payload;
      }
    },
    setMoneroNodeUrl(slice, action: PayloadAction<string | null>) {
      if (action.payload.length === 0) {
        slice.monero_node_url = null;
      } else {
        slice.monero_node_url = action.payload;
      }
    },
  },
});

export const {
  setBitcoinConfirmationTarget,
  setElectrumRpcUrl,
  setMoneroNodeUrl,
} = alertsSlice.actions;
export default alertsSlice.reducer;
