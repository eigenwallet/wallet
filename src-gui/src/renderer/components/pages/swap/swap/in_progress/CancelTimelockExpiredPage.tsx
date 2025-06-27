import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import CancelButton from "../CancelButton";

export default function CancelTimelockExpiredPage() {
  return (
    <>
      <CircularProgressWithSubtitle description="Cancelling the swap" />
      <CancelButton />
    </>
  );
}
