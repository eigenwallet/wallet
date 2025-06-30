import { Box } from "@mui/material";
import CancelButton from "renderer/components/pages/swap/swap/CancelButton";

export default function SwapBasePage({ children,
  showCancelButton = true,
}: {
  children: React.ReactNode;
  showCancelButton?: boolean;
}) {
  return (
    <Box>
      {children}
      {showCancelButton && <CancelButton />}
    </Box>
  );
}