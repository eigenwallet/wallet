import {
  Box,
  Button,
  Checkbox,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  FormControl,
  FormControlLabel,
  MenuItem,
  Paper,
  Select,
  TextField,
} from "@material-ui/core";
import { useSnackbar } from "notistack";
import { useState } from "react";
import TruncatedText from "renderer/components/other/TruncatedText";
import { store } from "renderer/store/storeRenderer";
import { useActiveSwapInfo, useAppSelector } from "store/hooks";
import { parseDateString } from "utils/parseUtils";
import { submitFeedbackViaHttp, Feedback, AttachmentInput } from "../../../api";
import LoadingButton from "../../other/LoadingButton";
import { PiconeroAmount } from "../../other/Units";
import { getLogsOfSwap } from "renderer/rpc";
import logger from "utils/logger";
import { addFeedbackId } from "store/features/conversationsSlice";

async function submitFeedback(body: string, swapId: string | number, submitDaemonLogs: boolean) {
  const attachments: AttachmentInput[] = [];

  // Handle swap logs and info
  if (swapId !== 0 && typeof swapId === "string") {
    const swapInfo = store.getState().rpc.state.swapInfos[swapId];
    if (swapInfo) {
      // Add swap info as an attachment
      attachments.push({
        key: `swap_info_${swapId}.json`,
        content: JSON.stringify(swapInfo, null, 2), // Pretty print JSON
      });
      // Retrieve and add logs for the specific swap
      try {
          const logs = await getLogsOfSwap(swapId, false);
          attachments.push({
            key: `swap_logs_${swapId}.txt`,
            content: logs.logs.map((l) => JSON.stringify(l)).join("\n"),
          });
      } catch (logError) {
          logger.error(logError, "Failed to get logs for swap", { swapId });
          // Optionally add an attachment indicating log retrieval failure
          attachments.push({ key: `swap_logs_${swapId}.error`, content: "Failed to retrieve swap logs." });
      }
    } else {
      logger.warn("Selected swap info not found in state", { swapId });
      attachments.push({ key: `swap_info_${swapId}.error`, content: "Swap info not found." });
    }
  }

  // Handle daemon logs
  if (submitDaemonLogs) {
    const logs = store.getState().rpc?.logs ?? [];
    if (logs.length > 0) {
        attachments.push({
            key: "daemon_logs.txt",
            content: logs.map((l) => JSON.stringify(l)).join("\n"),
        });
    } else {
        logger.info("Daemon logs requested but none found in state.");
    }
  }

  // Call the updated API function
  const feedbackId = await submitFeedbackViaHttp(body, attachments);
  
  // Dispatch only the ID
  store.dispatch(addFeedbackId(feedbackId)); 
}

/*
 * This component is a dialog that allows the user to submit feedback to the
 * developers. The user can enter a message and optionally attach logs from a
 * specific swap.
 * selectedSwap = 0 means no swap is attached
 */
function SwapSelectDropDown({
  selectedSwap,
  setSelectedSwap,
}: {
  selectedSwap: string | number;
  setSelectedSwap: (swapId: string | number) => void;
}) {
  const swaps = useAppSelector((state) =>
    Object.values(state.rpc.state.swapInfos),
  );

  return (
    <Select
      value={selectedSwap}
      label="Attach logs"
      variant="outlined"
      onChange={(e) => setSelectedSwap(e.target.value as string)}
    >
      <MenuItem value={0}>Do not attach a swap</MenuItem>
      {swaps.map((swap) => (
        <MenuItem value={swap.swap_id} key={swap.swap_id}>
          Swap <TruncatedText>{swap.swap_id}</TruncatedText> from{" "}
          {new Date(parseDateString(swap.start_date)).toDateString()} (
          <PiconeroAmount amount={swap.xmr_amount} />)
        </MenuItem>
      ))}
    </Select>
  );
}

const MAX_FEEDBACK_LENGTH = 4000;

export default function FeedbackDialog({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const [pending, setPending] = useState(false);
  const [bodyText, setBodyText] = useState("");
  const currentSwapId = useActiveSwapInfo();

  const { enqueueSnackbar } = useSnackbar();

  const [selectedAttachedSwap, setSelectedAttachedSwap] = useState<
    string | number
  >(currentSwapId?.swap_id || 0);
  const [attachDaemonLogs, setAttachDaemonLogs] = useState(true);

  const bodyTooLong = bodyText.length > MAX_FEEDBACK_LENGTH;

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>Submit Feedback</DialogTitle>
      <DialogContent>
        <DialogContentText>
          Got something to say? Drop us a message below. If you had an issue
          with a specific swap, select it from the dropdown to attach the logs.
          It will help us figure out what went wrong.
          <br />
          We appreciate you taking the time to share your thoughts! Every feedback is read by a core developer!
        </DialogContentText>
        <Box
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "1rem",
          }}
        >
          <TextField
            variant="outlined"
            value={bodyText}
            onChange={(e) => setBodyText(e.target.value)}
            label={
              bodyTooLong
                ? `Text is too long (${bodyText.length}/${MAX_FEEDBACK_LENGTH})`
                : "Message"
            }
            multiline
            minRows={4}
            maxRows={4}
            fullWidth
            error={bodyTooLong}
          />
          <SwapSelectDropDown
            selectedSwap={selectedAttachedSwap}
            setSelectedSwap={setSelectedAttachedSwap}
          />
          <Paper variant="outlined" style={{ padding: "0.5rem" }}>
            <FormControlLabel
              control={
                <Checkbox
                  color="primary"
                  checked={attachDaemonLogs}
                  onChange={(e) => setAttachDaemonLogs(e.target.checked)}
                />
              }
              label="Attach daemon logs"
            />
          </Paper>
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <LoadingButton
          color="primary"
          variant="contained"
          onClick={async () => {
            if (pending) {
              return;
            }

            try {
              setPending(true);
              await submitFeedback(bodyText, selectedAttachedSwap, attachDaemonLogs);
              enqueueSnackbar("Feedback submitted successfully!", {
                variant: "success",
              });
            } catch (e) {
              logger.error(`Failed to submit feedback: ${e}`);
              enqueueSnackbar(`Failed to submit feedback (${e})`, {
                variant: "error",
              });
            } finally {
              setPending(false);
            }
            onClose();
          }}
          loading={pending}
        >
          Submit
        </LoadingButton>
      </DialogActions>
    </Dialog>
  );
}
