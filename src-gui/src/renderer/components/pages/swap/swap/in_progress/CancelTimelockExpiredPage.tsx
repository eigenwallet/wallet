import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import SwapBasePage from "../components/SwapBasePage";

export default function CancelTimelockExpiredPage() {
  return (
    <SwapBasePage>
      <CircularProgressWithSubtitle description="Cancelling the swap" />
    </SwapBasePage>
  );
}
