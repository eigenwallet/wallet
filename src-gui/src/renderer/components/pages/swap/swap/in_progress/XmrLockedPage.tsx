import CircularProgressWithSubtitle from "../components/CircularProgressWithSubtitle";
import CancelButton from "../CancelButton";

export default function XmrLockedPage() {
  return (
    <>
      <CircularProgressWithSubtitle description="Revealing encrypted signature to the other party" />
      <CancelButton />
    </>
  );
}
