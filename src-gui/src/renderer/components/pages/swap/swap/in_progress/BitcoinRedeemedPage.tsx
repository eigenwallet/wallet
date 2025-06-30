import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import SwapBasePage from "../components/SwapBasePage";

export default function BitcoinRedeemedPage() {
  return (
    <SwapBasePage>
      <CircularProgressWithSubtitle description="Redeeming your Monero" />
    </SwapBasePage>
  );
}
