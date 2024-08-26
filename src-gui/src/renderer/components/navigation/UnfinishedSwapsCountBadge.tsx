import { Badge } from "@mui/material";
import { useResumeableSwapsCount } from "store/hooks";

export default function UnfinishedSwapsBadge({
  children,
}: {
  children: JSX.Element;
}) {
  const resumableSwapsCount = useResumeableSwapsCount();

  if (resumableSwapsCount > 0) {
    return (
      <Badge badgeContent={resumableSwapsCount} color="primary">
        {children}
      </Badge>
    );
  }
  return children;
}
