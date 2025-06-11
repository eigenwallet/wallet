import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { ConnectionProgress, ConnectionProgressUpdate } from "models/tauriModel";

export interface ConnectionSlice {
  // Map of peer_id to connection progress
  connections: Record<string, ConnectionProgress>;
  // Last update timestamp for each connection
  lastUpdated: Record<string, number>;
  // Global connection stats
  stats: {
    totalConnections: number;
    activeConnections: number;
    failedConnections: number;
    totalRetries: number;
  };
}

const initialState: ConnectionSlice = {
  connections: {},
  lastUpdated: {},
  stats: {
    totalConnections: 0,
    activeConnections: 0,
    failedConnections: 0,
    totalRetries: 0,
  },
};

export const connectionSlice = createSlice({
  name: "connections",
  initialState,
  reducers: {
    connectionProgressUpdated(slice, action: PayloadAction<ConnectionProgressUpdate>) {
      const { peer_id, progress } = action.payload;
      const previousProgress = slice.connections[peer_id];
      
      // Update the connection progress
      slice.connections[peer_id] = progress;
      slice.lastUpdated[peer_id] = Date.now();

      // Update global stats
      if (!previousProgress) {
        // New connection being tracked
        slice.stats.totalConnections += 1;
      }

      // Update active connections count
      slice.stats.activeConnections = Object.values(slice.connections).filter(
        (conn: ConnectionProgress) => conn.state === "Connecting" || conn.state === "WaitingToRetry"
      ).length;

      // Update failed connections count
      slice.stats.failedConnections = Object.values(slice.connections).filter(
        (conn: ConnectionProgress) => conn.state === "Failed"
      ).length;

      // Update total retries
      if (previousProgress && progress.total_attempts > previousProgress.total_attempts) {
        slice.stats.totalRetries += progress.total_attempts - previousProgress.total_attempts;
      } else if (!previousProgress && progress.total_attempts > 0) {
        slice.stats.totalRetries += progress.total_attempts;
      }
    },

    connectionRemoved(slice, action: PayloadAction<string>) {
      const peerId = action.payload;
      delete slice.connections[peerId];
      delete slice.lastUpdated[peerId];
      
      // Recalculate stats
      slice.stats.activeConnections = Object.values(slice.connections).filter(
        (conn: ConnectionProgress) => conn.state === "Connecting" || conn.state === "WaitingToRetry"
      ).length;

      slice.stats.failedConnections = Object.values(slice.connections).filter(
        (conn: ConnectionProgress) => conn.state === "Failed"
      ).length;
    },

    connectionStatsReset(slice) {
      slice.stats = {
        totalConnections: 0,
        activeConnections: 0,
        failedConnections: 0,
        totalRetries: 0,
      };
    },

    // Clear old connection records (older than 1 hour)
    cleanupOldConnections(slice) {
      const oneHourAgo = Date.now() - 60 * 60 * 1000;
      const toRemove: string[] = [];

      Object.entries(slice.lastUpdated).forEach(([peerId, timestamp]: [string, number]) => {
        if (timestamp < oneHourAgo && slice.connections[peerId]?.state === "Connected") {
          toRemove.push(peerId);
        }
      });

      toRemove.forEach((peerId) => {
        delete slice.connections[peerId];
        delete slice.lastUpdated[peerId];
      });
    },
  },
});

export const {
  connectionProgressUpdated,
  connectionRemoved,
  connectionStatsReset,
  cleanupOldConnections,
} = connectionSlice.actions;

// Selectors
export const selectConnectionProgress = (state: { connections: ConnectionSlice }, peerId: string) =>
  state.connections.connections[peerId];

export const selectAllConnections = (state: { connections: ConnectionSlice }) =>
  state.connections.connections;

export const selectConnectionStats = (state: { connections: ConnectionSlice }) =>
  state.connections.stats;

export const selectActiveConnections = (state: { connections: ConnectionSlice }) =>
  Object.entries(state.connections.connections).filter(
    ([_, progress]: [string, ConnectionProgress]) => progress.state === "Connecting" || progress.state === "WaitingToRetry"
  );

export const selectFailedConnections = (state: { connections: ConnectionSlice }) =>
  Object.entries(state.connections.connections).filter(
    ([_, progress]: [string, ConnectionProgress]) => progress.state === "Failed"
  );

export const selectConnectionsWithManyRetries = (state: { connections: ConnectionSlice }, threshold = 10) =>
  Object.entries(state.connections.connections).filter(
    ([_, progress]: [string, ConnectionProgress]) => progress.total_attempts >= threshold
  );

export const selectMostRecentConnectionUpdate = (state: { connections: ConnectionSlice }) => {
  const connections = Object.entries(state.connections.connections);
  if (connections.length === 0) return null;

  return connections.reduce((latest, [peerId, progress]) => {
    const timestamp = state.connections.lastUpdated[peerId] || 0;
    if (!latest || timestamp > (state.connections.lastUpdated[latest[0]] || 0)) {
      return [peerId, progress] as [string, ConnectionProgress];
    }
    return latest;
  }, null as [string, ConnectionProgress] | null);
};

export default connectionSlice.reducer;