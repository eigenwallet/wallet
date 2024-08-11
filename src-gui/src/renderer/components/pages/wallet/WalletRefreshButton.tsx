import { Button, CircularProgress, IconButton } from "@mui/material";
import RefreshIcon from "@mui/icons-material/Refresh";
import IpcInvokeButton from "../../IpcInvokeButton";
import { checkBitcoinBalance } from "renderer/rpc";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";

export default function WalletRefreshButton() {
  return (
    <PromiseInvokeButton
      endIcon={<RefreshIcon />}
      isIconButton
      onClick={() => checkBitcoinBalance()}
      size="small"
    />
  );
}
