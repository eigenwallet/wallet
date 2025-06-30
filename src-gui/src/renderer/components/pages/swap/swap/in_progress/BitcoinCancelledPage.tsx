import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import SwapBasePage from "../components/SwapBasePage";

export default function BitcoinCancelledPage() {
  return (
    <SwapBasePage>
      <CircularProgressWithSubtitle description="Refunding your Bitcoin" />
    </SwapBasePage>
  );
}
