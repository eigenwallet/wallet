import { createListenerMiddleware } from "@reduxjs/toolkit";
import { getAllSwapInfos, checkBitcoinBalance } from "renderer/rpc";
import logger from "utils/logger";
import { contextStatusEventReceived } from "store/features/rpcSlice";

export function createMainListeners() {
  const listener = createListenerMiddleware();

  // Listener for when the Context becomes available
  // When the context becomes available, we check the bitcoin balance and fetch all swap infos
  listener.startListening({
    predicate: (action) => {
      // Check if the action is the contextStatusEventReceived action
      return action.type === "rpc/contextStatusEventReceived";
    },
    effect: async (action) => {
      const status = action.payload as Parameters<
        typeof contextStatusEventReceived
      >[0];

      if (status.type === "Available") {
        logger.debug(
          "Context is available, checking bitcoin balance and history",
        );
        await checkBitcoinBalance();
        await getAllSwapInfos();
      }
    },
  });

  return listener;
}
