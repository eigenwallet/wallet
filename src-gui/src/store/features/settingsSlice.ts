import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { TauriSettings } from "models/tauriModel";
import { updateRates } from "renderer/api";
import { Theme } from "renderer/components/theme";

export interface SettingsState {
  /// This is an ordered list of node urls for each network and blockchain
  nodes: Record<Network, Record<Blockchain, string[]>>;
  /// Which theme to use
  theme: Theme;
  /// Whether to fetch fiat prices from the internet
  fetchFiatPrices: boolean;
  fiatCurrency: FiatCurrency;
}

export enum FiatCurrency {
  Usd = "USD",
  Eur = "EUR",
  Gbp = "GBP",
  Chf = "CHF",
  Jpy = "JPY",
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
  theme: Theme.Dark,
  fetchFiatPrices: false,
  fiatCurrency: FiatCurrency.Usd,
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
    setFetchFiatPrices(slice, action: PayloadAction<boolean>) {
      if (action.payload === true) 
        try { updateRates() } catch (_) {}
      slice.fetchFiatPrices = action.payload;
    },
    setFiatCurrency(slice, action: PayloadAction<FiatCurrency>) {
      console.log("setFiatCurrency", action.payload);
      slice.fiatCurrency = action.payload;
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
  resetSettings,
  setFetchFiatPrices,
  setFiatCurrency,
} = alertsSlice.actions;
export default alertsSlice.reducer;
