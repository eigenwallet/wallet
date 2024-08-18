import { TauriSwapProgressEvent } from "./tauriModel";

export type TauriSwapProgressEventContent<
  T extends TauriSwapProgressEvent["type"],
> = Extract<TauriSwapProgressEvent, { type: T }>["content"];

type TauriSwapStateName = TauriSwapProgressEvent["type"];
