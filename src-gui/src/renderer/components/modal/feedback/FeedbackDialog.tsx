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
  IconButton,
  MenuItem,
  Paper,
  Select,
  TextField,
  Typography,
} from "@material-ui/core";
import { useSnackbar } from "notistack";
import { useEffect, useState } from "react";
import TruncatedText from "renderer/components/other/TruncatedText";
import { store } from "renderer/store/storeRenderer";
import { useActiveSwapInfo, useAppSelector } from "store/hooks";
import { parseDateString } from "utils/parseUtils";
import { submitFeedbackViaHttp } from "../../../api";
import LoadingButton from "../../other/LoadingButton";
import { PiconeroAmount } from "../../other/Units";
import { getLogsOfSwap } from "renderer/rpc";
import logger from "utils/logger";
import { Edit } from "@material-ui/icons";
import PaperTextBox from "../PaperTextBox";

async function submitFeedback(body: string, swapId: string | null, swapLogs: string | null, daemonLogs: string | null) {
  let attachedBody = "";

  if (swapId !== null) {
    const swapInfo = store.getState().rpc.state.swapInfos[swapId];

    if (swapInfo === undefined) {
      throw new Error(`Swap with id ${swapId} not found`);
    }

    attachedBody = `${JSON.stringify(swapInfo, null, 4)}\n\nLogs: ${swapLogs ?? ""}`;
  }

  if (daemonLogs !== null) {
    attachedBody += `\n\nDaemon Logs: ${daemonLogs ?? ""}`;
  }

  console.log(`Sending feedback with attachement: \`\n${attachedBody}\``)
  await submitFeedbackViaHttp(body, attachedBody);
}

/*
 * This component is a dialog that allows the user to submit feedback to the
 * developers. The user can enter a message and optionally attach logs from a
 * specific swap.
 * selectedSwap = null means no swap is attached
 */
function SwapSelectDropDown({
  selectedSwap,
  setSelectedSwap,
}: {
  selectedSwap: string | null;
  setSelectedSwap: (swapId: string | null) => void;
}) {
  const swaps = useAppSelector((state) =>
    Object.values(state.rpc.state.swapInfos),
  );

  return (
    <Select
      value={selectedSwap ?? ""}
      variant="outlined"
      onChange={(e) => setSelectedSwap(e.target.value as string || null)}
      style={{ width: "100%" }}
    >
      <MenuItem value="">Do not attach a swap</MenuItem>
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

  const [selectedSwap, setSelectedSwap] = useState<
    string | null
  >(currentSwapId?.swap_id || null);
  const [swapLogs, setSwapLogs] = useState<string | null>(null);
  const [attachDaemonLogs, setAttachDaemonLogs] = useState(true);

  const [daemonLogs, setDaemonLogs] = useState<string | null>(null);

  useEffect(() => {
    // Reset logs if no swap is selected
    if (selectedSwap === null) {
      setSwapLogs(null);
      return;
    }

    // Fetch the logs from the rust backend and update the state
    getLogsOfSwap(selectedSwap, false).then((response) => setSwapLogs(response.logs.join("\n")))
  }, [selectedSwap]);

  useEffect(() => {
    if (attachDaemonLogs === false) {
      setDaemonLogs(null);
      return;
    }

    setDaemonLogs(store.getState().rpc?.logs.map((log) => {
      if (typeof log === "string")
        return log;
      else
        return JSON.stringify(log)
    }).join("\n"))
  }, [attachDaemonLogs]);

  // Whether to display the log editor
  const [swapLogsEditorOpen, setSwapLogsEditorOpen] = useState(false);
  const [daemonLogsEditorOpen, setDaemonLogsEditorOpen] = useState(false);

  const bodyTooLong = bodyText.length > MAX_FEEDBACK_LENGTH;

  const sendFeedback = async () => {
    if (pending) {
      return;
    }

    try {
      setPending(true);
      await submitFeedback(bodyText, selectedSwap, swapLogs, daemonLogs);
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
  }

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>Submit Feedback</DialogTitle>
      <DialogContent>
        <DialogContentText>
          Got something to say? Drop us a message below. If you had an issue
          with a specific swap, select it from the dropdown to attach the logs.
          It will help us figure out what went wrong.
          <br />
          We appreciate you taking the time to share your thoughts! Every message is read by a core developer!
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
          <Box style={{
            display: "flex",
            flexDirection: "row",
          }}>

            <SwapSelectDropDown
              selectedSwap={selectedSwap}
              setSelectedSwap={setSelectedSwap}
            />
            {selectedSwap !== null ? <IconButton onClick={() => setSwapLogsEditorOpen(true)}>
              <Edit />
            </IconButton> : <></>
            }
          </Box>
          <LogEditor open={swapLogsEditorOpen} setOpen={setSwapLogsEditorOpen} logs={swapLogs} setLogs={setSwapLogs} />
          <Box style={{
            display: "flex",
            flexDirection: "row",
          }}>
            <Paper variant="outlined" style={{ padding: "0.5rem", width: "100%" }} >
              <FormControlLabel
                control={
                  <Checkbox
                    color="primary"
                    checked={attachDaemonLogs}
                    onChange={(e) => setAttachDaemonLogs(e.target.checked)}
                  />
                }
                label="Attach logs from the current session"
              />
            </Paper>
            {attachDaemonLogs ? <IconButton onClick={() => setDaemonLogsEditorOpen(true)}>
              <Edit />
            </IconButton> : <></>
            }
          </Box>
          <LogEditor open={daemonLogsEditorOpen} setOpen={setDaemonLogsEditorOpen} logs={daemonLogs} setLogs={setDaemonLogs} />
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <LoadingButton
          color="primary"
          variant="contained"
          onClick={sendFeedback}
          loading={pending}
        >
          Submit
        </LoadingButton>
      </DialogActions>
    </Dialog>
  );
}

function LogEditor(
  { open,
    setOpen,
    logs,
    setLogs
  }: {
    open: boolean,
    setOpen: (boolean) => void,
    logs: string | null,
    setLogs: (_: string | null) => void
  }) {
  const onChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setLogs(event.target.value.length === 0 ? null : event.target.value);
  }

  return (
    <Dialog open={open} onClose={() => setOpen(false)} fullWidth>
      <DialogContent>
        <Typography>
          These are the logs that would be attached to your feedback message.
          For long logs, it might be advisable to edit them in a real editor
          and copy-paste them here after that.
        </Typography>
        <TextField defaultValue={logs} onChange={onChange} multiline
          minRows={8}
          maxRows={8}
          fullWidth
          variant="outlined">
        </TextField>
      </DialogContent>
      <DialogActions>
        <Button variant="contained" color="primary" onClick={() => setOpen(false)}>
          Close
        </Button>
      </DialogActions>
    </Dialog>
  )
}