import { createSlice, PayloadAction } from "@reduxjs/toolkit";

export interface SettingsSlice {
  bitcoinConfirmationTarget: number;
}

const initialState: SettingsSlice = {
  bitcoinConfirmationTarget: 1,
};

const alertsSlice = createSlice({
  name: "settings",
  initialState,
  reducers: {
    setBitcoinConfirmationTarget(slice, action: PayloadAction<number>) {
      slice.bitcoinConfirmationTarget = action.payload;
    },
  },
});

export const { setBitcoinConfirmationTarget } = alertsSlice.actions;
export default alertsSlice.reducer;
