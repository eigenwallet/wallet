import { combineReducers, configureStore, StoreEnhancer } from "@reduxjs/toolkit";
import { persistReducer, persistStore, FLUSH, REHYDRATE, PAUSE, PERSIST, PURGE, REGISTER } from "redux-persist";
import sessionStorage from "redux-persist/lib/storage/session";
import { reducers } from "store/combinedReducer";
import { createMainListeners } from "store/middleware/storeListener";
import { LazyStore } from "@tauri-apps/plugin-store";

// Goal: Maintain application state across page reloads while allowing a clean slate on application restart
// Settings are persisted across application restarts, while the rest of the state is cleared

// Persist user settings across application restarts
// We use Tauri's storage for settings to ensure they're retained even when the application is closed
const rootPersistConfig = {
  key: "gui-global-state-store",
  storage: sessionStorage,
  blacklist: ["settings"],
};

// Use Tauri's store plugin for persistent settings
const tauriStore = new LazyStore("settings.bin");

// Helper to adapt Tauri storage to redux-persist (expects stringified JSON)
const createTauriStorage = () => ({
  getItem: async (key: string): Promise<string | null> => {
    const value = await tauriStore.get<unknown>(key);
    return value == null ? null : JSON.stringify(value);
  },
  setItem: async (key: string, value: string): Promise<void> => {
    try {
      await tauriStore.set(key, JSON.parse(value));
      await tauriStore.save();
    } catch (err) {
      console.error(`Error parsing or setting item "${key}" in Tauri store:`, err);
    }
  },
  removeItem: async (key: string): Promise<void> => {
    await tauriStore.delete(key);
    await tauriStore.save();
  },
});

// Configure how settings are stored and retrieved using Tauri's storage
const settingsPersistConfig = {
  key: "settings",
  storage: createTauriStorage(),
};

// Create a persisted version of the settings reducer
const persistedSettingsReducer = persistReducer(
  settingsPersistConfig,
  reducers.settings,
);

// Combine all reducers, using the persisted settings reducer
const rootReducer = combineReducers({
  ...reducers,
  settings: persistedSettingsReducer,
});

// Enable persistence for the entire application state
const persistedReducer = persistReducer(rootPersistConfig, rootReducer);

let remoteDevToolsEnhancer: StoreEnhancer | undefined;

if (import.meta.env.DEV) {
  console.log('Development mode detected, attempting to enable Redux DevTools Remote...');
  try {
    const { devToolsEnhancer } = await import('@redux-devtools/remote');
    remoteDevToolsEnhancer = devToolsEnhancer({
      name: 'UnstoppableSwap_RemoteInstance',
      realtime: true,
      hostname: 'localhost',
      port: 8098,
    });
    console.log('Redux DevTools Remote enhancer is ready.');
  } catch (e) {
    console.warn('Could not enable Redux DevTools Remote.', e);
    remoteDevToolsEnhancer = undefined;
  }
}

// Set up the Redux store with persistence, middleware, and remote DevTools
export const store = configureStore({
  reducer: persistedReducer,
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: [FLUSH, REHYDRATE, PAUSE, PERSIST, PURGE, REGISTER],
      },
    }).prepend(createMainListeners().middleware),
  devTools: false,
  enhancers: (getDefaultEnhancers) => {
    const defaultEnhancers = getDefaultEnhancers();
    return remoteDevToolsEnhancer
      ? defaultEnhancers.concat(remoteDevToolsEnhancer)
      : defaultEnhancers;
  },
});

// Create a persistor to manage the persisted store
export const persistor = persistStore(store);

// TypeScript type definitions for easier use of the store in the application
export type AppDispatch = typeof store.dispatch;
export type RootState = ReturnType<typeof store.getState>;