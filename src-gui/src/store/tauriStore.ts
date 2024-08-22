import { Store } from "@tauri-apps/plugin-store";

const store = new Store("store.bin");

export async function getSumittedFeedbackIds(): Promise<string[]> {
  return (await store.get<string[]>("submitted-feedback")) ?? [];
}

export async function addSubmittedFeedbackId(id: string) {
  const ids = (await getSumittedFeedbackIds()).concat(id);
  await store.set("submitted-feedback", ids);
  console.log(ids);
}
