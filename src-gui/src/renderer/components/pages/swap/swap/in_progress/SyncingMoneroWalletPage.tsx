import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";

export function SyncingMoneroWalletPage() {
  return (
    <CircularProgressWithSubtitle description="Syncing Monero wallet with blockchain, this might take a while..." />
  );
}
