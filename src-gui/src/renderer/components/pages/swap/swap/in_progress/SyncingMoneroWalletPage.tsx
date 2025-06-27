import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import CancelButton from "../CancelButton";

export function SyncingMoneroWalletPage() {
  return (
    <>
      <CircularProgressWithSubtitle description="Syncing Monero wallet with blockchain, this might take a while..." />
      <CancelButton />
    </>
  );
}
