import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { TauriSettings } from "models/tauriModel";

export interface SettingsState {
  /// Settings needed to start the tauri context
  tauriSettings: TauriSettings;
  /// This is an ordered list of node urls for each network and blockchain
  nodes: Record<Blockchain, string[]>;
  /// Which theme to use
  theme: Theme;
}

export enum Network {
  Testnet = "testnet",
  Mainnet = "mainnet"
}

export enum Blockchain {
  Bitcoin = "bitcoin",
  Monero = "monero"
}

const initialState: SettingsState = {
  tauriSettings: {
    electrum_rpc_url: null,
    monero_node_url: null,
  },
  nodes: {
    [Blockchain.Bitcoin]: [],
    [Blockchain.Monero]: []
  },
  theme: Theme.Dark
};

const alertsSlice = createSlice({
  name: "settings",
  initialState,
  reducers: {
    setElectrumRpcUrl(slice, action: PayloadAction<string | null>) {
      if (action.payload === null || action.payload === "") {
        slice.electrum_rpc_url = null;
      } else {
        slice.electrum_rpc_url = action.payload;
      }
    },
    setMoneroNodeUrl(slice, action: PayloadAction<string | null>) {
      if (action.payload === null || action.payload === "") {
        slice.monero_node_url = null;
      } else {
        slice.monero_node_url = action.payload;
      }
    },
    setTheme(slice, action: PayloadAction<Theme>) {
      slice.theme = action.payload;
    },
    addNode(slice, action: PayloadAction<{ type: Blockchain, node: string }>) {
      // Check if the node is already in the list
      if (slice.nodes[action.payload.type].includes(action.payload.node)) {
        return;
      }
      // Add the node to the list
      slice.nodes[action.payload.type].push(action.payload.node);
    },
    removeNode(slice, action: PayloadAction<{ type: Blockchain, node: string }>) {
      slice.nodes[action.payload.type] = slice.nodes[action.payload.type].filter(node => node !== action.payload.node);
    },
    resetSettings(_) {
      return initialState;
    }
  },
});

export const {
  setElectrumRpcUrl,
  setMoneroNodeUrl,
  resetSettings,
} = alertsSlice.actions;
export default alertsSlice.reducer;
