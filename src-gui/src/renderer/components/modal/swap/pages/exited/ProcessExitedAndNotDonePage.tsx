import { Box, DialogContentText } from "@material-ui/core";
import { useActiveSwapInfo, useAppSelector } from "store/hooks";
import CliLogsBox from "../../../../other/RenderedCliLog";

export default function ProcessExitedAndNotDonePage() {
  const swap = useActiveSwapInfo();
  const logs = useAppSelector((s) => s.swap.logs);

  function getText() {
    const hasSwap = swap != null;

    const messages = [];

    messages.push("The swap exited unexpectedly without completing.");

    if (!hasSwap) {
      messages.push("No funds were locked.");
    }

    messages.push("Check the logs below for more information.");

    if (hasSwap) {
      messages.push(`The swap is in the "${swap.state_name}" state.`);
    }

    return messages.join(" ");
  }

  return (
    <Box>
      <DialogContentText>{getText()}</DialogContentText>
      <Box
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "0.5rem",
        }}
      >
        {state.rpcError && (
          <CliLogsBox
            logs={[state.rpcError]}
            label="Error returned by the Swap Daemon"
          />
        )}
        <CliLogsBox logs={logs} label="Logs relevant to the swap" />
      </Box>
    </Box>
  );
}
