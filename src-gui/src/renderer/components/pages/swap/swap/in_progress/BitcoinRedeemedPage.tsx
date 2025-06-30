import CircularProgressWithSubtitle from "renderer/components/pages/swap/swap/components/CircularProgressWithSubtitle";
import SwapBasePage from "renderer/components/pages/swap/swap/components/SwapBasePage";

export default function BitcoinRedeemedPage() {
  return (
    <SwapBasePage>
      <CircularProgressWithSubtitle description="Redeeming your Monero" />
    </SwapBasePage>
  );
}
