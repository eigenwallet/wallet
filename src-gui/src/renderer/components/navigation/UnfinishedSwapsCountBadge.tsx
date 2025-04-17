import { Badge } from "@material-ui/core";
import { useResumeableSwapsCountExcludingPunishedAndSetup } from "store/hooks";

export default function UnfinishedSwapsBadge({
  children,
}: {
  children: JSX.Element;
}) {
  const resumableSwapsCount = useResumeableSwapsCountExcludingPunishedAndSetup();

  if (resumableSwapsCount > 0) {
    return (
      <Badge badgeContent={resumableSwapsCount} color="primary">
        {children}
      </Badge>
    );
  }
  return children;
}
