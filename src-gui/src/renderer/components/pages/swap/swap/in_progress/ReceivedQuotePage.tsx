import {
  useConservativeBitcoinSyncProgress,
  usePendingBackgroundProcesses,
} from "store/hooks";
import CircularProgressWithSubtitle, {
  LinearProgressWithSubtitle,
} from "../components/CircularProgressWithSubtitle";
import SwapBasePage from "../components/SwapBasePage";

export default function ReceivedQuotePage() {
  const syncProgress = useConservativeBitcoinSyncProgress();
  let progress = null;

  if (syncProgress?.type === "Known") {
    const percentage = Math.round(
      (syncProgress.content.consumed / syncProgress.content.total) * 100,
    );

    progress = (
      <LinearProgressWithSubtitle
        description={`Syncing Bitcoin wallet`}
        value={percentage}
      />
    );
  }

  if (syncProgress?.type === "Unknown") {
    progress = (
      <CircularProgressWithSubtitle description="Syncing Bitcoin wallet" />
    );
  }

  progress = <CircularProgressWithSubtitle description="Processing offer" />;

  return (
    <SwapBasePage>
      {progress}
    </SwapBasePage>
  );
}
