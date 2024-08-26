import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
} from "@mui/material";
import { ButtonProps } from "@mui/material/Button/Button";
import { CliLog } from "models/cliModel";
import { useState } from "react";
import PromiseInvokeButton from "renderer/components/PromiseInvokeButton";
import CliLogsBox from "../../../other/RenderedCliLog";

export default function SwapLogFileOpenButton({
  swapId,
  ...props
}: { swapId: string } & ButtonProps) {
  const [logs, setLogs] = useState<CliLog[] | null>(null);

  return (
    <>
      <PromiseInvokeButton
        onSuccess={(data) => {
          setLogs(data as CliLog[]);
        }}
        onClick={async () => {
          throw new Error("Not implemented");
        }}
        {...props}
      >
        View log
      </PromiseInvokeButton>
      {logs && (
        <Dialog open onClose={() => setLogs(null)} fullWidth maxWidth="lg">
          <DialogTitle>Logs of swap {swapId}</DialogTitle>
          <DialogContent>
            <CliLogsBox logs={logs} label="Logs relevant to the swap" />
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setLogs(null)}>Close</Button>
          </DialogActions>
        </Dialog>
      )}
    </>
  );
}
