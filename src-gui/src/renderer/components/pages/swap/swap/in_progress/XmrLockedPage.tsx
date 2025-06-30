import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import SwapBasePage from "../components/SwapBasePage";

export default function XmrLockedPage() {
  return (
    <SwapBasePage>
      <CircularProgressWithSubtitle description="Revealing encrypted signature to the other party" />
    </SwapBasePage>
  );
}
