import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import CancelButton from "../CancelButton";

export default function BitcoinCancelledPage() {
  return (
    <>
      <CircularProgressWithSubtitle description="Refunding your Bitcoin" />
      <CancelButton />
    </>
  );
}
