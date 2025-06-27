import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import CancelButton from "../CancelButton";

export default function BitcoinRedeemedPage() {
  return (
    <>
      <CircularProgressWithSubtitle description="Redeeming your Monero" />
      <CancelButton />
    </>
  );
}
