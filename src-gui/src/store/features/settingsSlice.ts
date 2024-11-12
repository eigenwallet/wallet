import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { TauriSettings } from "models/tauriModel";

export interface SettingsState {
  /// This is an ordered list of node urls for each network and blockchain
  nodes: Record<Network, Record<Blockchain, string[]>>;
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
  nodes: {
    [Network.Testnet]: {
      [Blockchain.Bitcoin]: [],
      [Blockchain.Monero]: []
    },
    [Network.Mainnet]: {
      [Blockchain.Bitcoin]: [],
      [Blockchain.Monero]: []
    }
  },
  theme: Theme.Dark
};

const alertsSlice = createSlice({
  name: "settings",
  initialState,
  reducers: {
    moveUpNode(slice, action: PayloadAction<{ network: Network, type: Blockchain, node: string }>) {
      const index = slice.nodes[action.payload.network][action.payload.type].indexOf(action.payload.node);
      if (index > 0) {
        const temp = slice.nodes[action.payload.network][action.payload.type][index];
        slice.nodes[action.payload.network][action.payload.type][index] = slice.nodes[action.payload.network][action.payload.type][index - 1];
        slice.nodes[action.payload.network][action.payload.type][index - 1] = temp;
      }
    },
    setTheme(slice, action: PayloadAction<Theme>) {
      slice.theme = action.payload;
    },
    addNode(slice, action: PayloadAction<{ network: Network, type: Blockchain, node: string }>) {
      // Make sure the node is not already in the list
      if (slice.nodes[action.payload.network][action.payload.type].includes(action.payload.node)) {
        return;
      }
      // Add the node to the list
      slice.nodes[action.payload.network][action.payload.type].push(action.payload.node);
    },
    removeNode(slice, action: PayloadAction<{ network: Network, type: Blockchain, node: string }>) {
      slice.nodes[action.payload.network][action.payload.type] = slice.nodes[action.payload.network][action.payload.type].filter(node => node !== action.payload.node);
    },
    resetSettings(_) {
      return initialState;
    }
  },
});

export const {
  moveUpNode,
  setTheme,
  addNode,
  removeNode,
  resetSettings
} = alertsSlice.actions;
export default alertsSlice.reducer;
