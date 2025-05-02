import { listen } from "@tauri-apps/api/event";
import { TauriSwapProgressEventWrapper, TauriContextStatusEvent, TauriLogEvent, BalanceResponse, TauriDatabaseStateEvent, TauriTimelockChangeEvent, TauriBackgroundRefundEvent, ApprovalRequest, TauriBackgroundProgressWrapper, TauriEvent } from "models/tauriModel";
import { contextStatusEventReceived, receivedCliLog, rpcSetBalance, timelockChangeEventReceived, rpcSetBackgroundRefundState, approvalEventReceived, backgroundProgressEventReceived } from "store/features/rpcSlice";
import { swapProgressEventReceived } from "store/features/swapSlice";
import logger from "utils/logger";
import { updatePublicRegistry, updateRates } from "./api";
import { checkContextAvailability, getSwapInfo, initializeContext, updateAllNodeStatuses } from "./rpc";
import { store } from "./store/storeRenderer";

// Update the public registry every 5 minutes
const PROVIDER_UPDATE_INTERVAL = 5 * 60 * 1_000;

// Update node statuses every 2 minutes
const STATUS_UPDATE_INTERVAL = 2 * 60 * 1_000;

// Update the exchange rate every 5 minutes
const UPDATE_RATE_INTERVAL = 5 * 60 * 1_000;

function setIntervalImmediate(callback: () => void, interval: number): void {
    callback();
    setInterval(callback, interval);
}

export async function setupBackgroundTasks(): Promise<void> {
    // Setup periodic fetch tasks
    setIntervalImmediate(updatePublicRegistry, PROVIDER_UPDATE_INTERVAL);
    setIntervalImmediate(updateAllNodeStatuses, STATUS_UPDATE_INTERVAL);
    setIntervalImmediate(updateRates, UPDATE_RATE_INTERVAL);

    // Check if the context is already available. This is to prevent unnecessary re-initialization
    if (await checkContextAvailability()) {
        store.dispatch(contextStatusEventReceived({ type: "Available" }));
    } else {
        // Warning: If we reload the page while the Context is being initialized, this function will throw an error
        initializeContext().catch((e) => {
            logger.error(e, "Failed to initialize context on page load. This might be because we reloaded the page while the context was being initialized");
            // Wait a short time before retrying
            setTimeout(() => {
                initializeContext().catch((e) => {
                    logger.error(e, "Failed to initialize context even after retry");
                });
            }, 2000);
        });
    }

    // Listen for the unified event
    listen<TauriEvent>("tauri-unified-event", (event) => {
        const { channelName, event: eventData } = event.payload;
        
        switch (channelName) {
            case "SwapProgress":
                store.dispatch(swapProgressEventReceived(eventData));
                break;
            
            case "ContextInitProgress":
                store.dispatch(contextStatusEventReceived(eventData));
                break;
            
            case "CliLog":
                store.dispatch(receivedCliLog(eventData));
                break;
            
            case "BalanceChange":
                store.dispatch(rpcSetBalance((eventData).balance));
                break;
            
            case "SwapDatabaseStateUpdate":
                getSwapInfo(eventData.swap_id);
                
                // This is ugly but it's the best we can do for now
                // Sometimes we are too quick to fetch the swap info and the new state is not yet reflected
                // in the database. So we wait a bit before fetching the new state
                setTimeout(() => getSwapInfo(eventData.swap_id), 3000);
                break;
            
            case "TimelockChange":
                store.dispatch(timelockChangeEventReceived(eventData));
                break;
            
            case "BackgroundRefund":
                store.dispatch(rpcSetBackgroundRefundState(eventData));
                break;
            
            case "Approval":
                store.dispatch(approvalEventReceived(eventData));
                break;
            
            case "BackgroundProgress":
                store.dispatch(backgroundProgressEventReceived(eventData));
                break;
            
            default:
                logger.warn(`Received unknown event type: ${channelName}`, eventData);
        }
    });
}