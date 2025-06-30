import CircularProgressWithSubtitle from "renderer/components/pages/swap/swap/components/CircularProgressWithSubtitle";
import SwapBasePage from "renderer/components/pages/swap/swap/components/SwapBasePage";

export default function BitcoinCancelledPage() {
  return (
    <SwapBasePage>
      <CircularProgressWithSubtitle description="Refunding your Bitcoin" />
    </SwapBasePage>
  );
}
