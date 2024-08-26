import { IconButton } from "@mui/material";
import { ReactNode } from "react";

export default function LinkIconButton({
  url,
  children,
}: {
  url: string;
  children: ReactNode;
}) {
  return (
    <IconButton
      component="span"
      onClick={() => window.open(url, "_blank")}
      size="large"
    >
      {children}
    </IconButton>
  );
}
