import { invoke as invokeUnsafe } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  BalanceArgs,
  BalanceResponse,
  BuyXmrArgs,
  BuyXmrResponse,
  GetSwapInfoResponse,
  ResumeArgs,
  ResumeSwapResponse,
  SuspendCurrentSwapResponse,
  TauriSwapProgressEventWrapper,
  WithdrawBtcArgs,
  WithdrawBtcResponse,
} from "models/tauriModel";
import { rpcSetBalance, rpcSetSwapInfo } from "store/features/rpcSlice";
import { swapTauriEventReceived } from "store/features/swapSlice";
import { store } from "./store/storeRenderer";

listen<TauriSwapProgressEventWrapper>("swap-progress-update", (event) => {
  store.dispatch(swapTauriEventReceived(event.payload));
});

async function invoke<ARGS, RESPONSE>(
  command: string,
  args: ARGS,
): Promise<RESPONSE> {
  return invokeUnsafe(command, {
    args: args as Record<string, unknown>,
  }) as Promise<RESPONSE>;
}

async function invokeNoArgs<RESPONSE>(command: string): Promise<RESPONSE> {
  return invokeUnsafe(command, {}) as Promise<RESPONSE>;
}

export async function checkBitcoinBalance() {
  const response = await invoke<BalanceArgs, BalanceResponse>("get_balance", {
    force_refresh: true,
  });

  store.dispatch(rpcSetBalance(response.balance));
}

export async function getRawSwapInfos() {
  const response =
    await invokeNoArgs<GetSwapInfoResponse[]>("get_swap_infos_all");

  response.forEach((swapInfo) => {
    store.dispatch(rpcSetSwapInfo(swapInfo));
  });
}

export async function withdrawBtc(address: string): Promise<string> {
  const response = await invoke<WithdrawBtcArgs, WithdrawBtcResponse>(
    "withdraw_btc",
    {
      address,
      amount: null,
    },
  );

  return response.txid;
}

export async function buyXmr(
  seller: string,
  bitcoin_change_address: string,
  monero_receive_address: string,
) {
  await invoke<BuyXmrArgs, BuyXmrResponse>("buy_xmr", {
    seller,
    bitcoin_change_address,
    monero_receive_address,
  });
}

export async function resumeSwap(swapId: string) {
  await invoke<ResumeArgs, ResumeSwapResponse>("resume_swap", {
    swap_id: swapId,
  });
}

export async function suspendCurrentSwap() {
  await invokeNoArgs<SuspendCurrentSwapResponse>("suspend_current_swap");
}
