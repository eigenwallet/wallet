import { combineReducers, configureStore } from "@reduxjs/toolkit";
import { persistReducer, persistStore } from "redux-persist";
import sessionStorage from "redux-persist/lib/storage/session";
import { reducers } from "store/combinedReducer";
import { createMainListeners } from "store/middleware/storeListener";
import { createStore } from "@tauri-apps/plugin-store";

// We persist the redux store in sessionStorage
// The point of this is to preserve the store across reloads while not persisting it across GUI restarts
//
// If the user reloads the page, while a swap is running we want to
// continue displaying the correct state of the swap

// Configure persistence for the rest of the reducers using sessionStorage.
// This ensures that the application state is preserved across page reloads
// but not across application restarts. The 'settings' reducer is excluded
// since it's persisted separately using Tauri's storage.
const rootPersistConfig = {
  key: "gui-global-state-store", // Key under which the state will be stored in sessionStorage
  storage: sessionStorage, // Use sessionStorage for persistence
  blacklist: ["settings"], // Exclude 'settings' reducer from this persistence layer
};

// Initialize Tauri's store for persisting settings across application restarts.
// This utilizes Tauri's native file system capabilities for persistent storage.
const tauriStore = await createStore("settings.bin", {
  // Workaround for https://github.com/tauri-apps/plugins-workspace/issues/1865
  autoSave: 1000 as unknown as boolean,
});

// Configure persistence for the 'settings' reducer using Tauri's storage.
// This ensures that user settings are retained even after the application is closed.
const settingsPersistConfig = {
  key: "settings", // Key under which 'settings' will be stored in Tauri's storage
  storage: {
    getItem: (key: string) => tauriStore.get(key), // Retrieve item from Tauri's storage
    setItem: (key: string, value: unknown) => tauriStore.set(key, value), // Save item to Tauri's storage
    removeItem: (key: string) => tauriStore.delete(key), // Remove item from Tauri's storage
  },
};

// Wrap the 'settings' reducer with 'persistReducer' using Tauri's storage configuration.
// This creates a persisted version of the 'settings' reducer.
const persistedSettingsReducer = persistReducer(
  settingsPersistConfig,
  reducers.settings,
);

// Combine all reducers into a root reducer, replacing the original 'settings' reducer
// with the persisted version. This ensures the 'settings' state uses Tauri's storage.
const rootReducer = combineReducers({
  ...reducers, // Include all other reducers
  settings: persistedSettingsReducer, // Use the persisted 'settings' reducer
});

// Wrap the rootReducer with 'persistReducer' using the sessionStorage configuration.
// This enables persistence for the rest of the application state across page reloads.
const persistedReducer = persistReducer(rootPersistConfig, rootReducer);

// Configure the Redux store with the persisted reducer and custom middleware.
export const store = configureStore({
  reducer: persistedReducer, // Use the combined persisted reducer
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: false, // Disable checks for non-serializable data
    }).prepend(createMainListeners().middleware), // Add custom middleware
});

// Create a persistor instance, which is required by 'redux-persist' to control persistence.
// This allows for actions like purging or flushing the persisted store when needed.
export const persistor = persistStore(store);

// TypeScript type definitions for the Redux store.
// - 'AppDispatch' represents the dispatch function from the store.
// - 'RootState' represents the overall state structure managed by Redux.
export type AppDispatch = typeof store.dispatch;
export type RootState = ReturnType<typeof store.getState>;
