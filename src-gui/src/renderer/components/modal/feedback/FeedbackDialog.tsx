import {
    Box,
    Button,
    Checkbox,
    Dialog,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControlLabel,
    IconButton,
    Paper,
    TextField,
    Tooltip,
    Typography,
} from '@material-ui/core'
import { Visibility } from '@material-ui/icons'
import LoadingButton from '../../other/LoadingButton'
import ExternalLink from 'renderer/components/other/ExternalLink'
import SwapSelectDropDown from './SwapSelectDropDown'
import LogViewer from './LogViewer'
import { useFeedback, MAX_FEEDBACK_LENGTH } from './useFeedback'
import { useState } from 'react'
import PromiseInvokeButton from 'renderer/components/PromiseInvokeButton'

export default function FeedbackDialog({
    open,
    onClose,
}: {
    open: boolean
    onClose: () => void
}) {
    const [swapLogsEditorOpen, setSwapLogsEditorOpen] = useState(false)
    const [daemonLogsEditorOpen, setDaemonLogsEditorOpen] = useState(false)

    const { input, setInputState, logs, error, clearState, submitFeedback } =
        useFeedback()

    const handleClose = () => {
        clearState()
        onClose()
    }

    const bodyTooLong = input.bodyText.length > MAX_FEEDBACK_LENGTH

    return (
        <Dialog open={open} onClose={handleClose}>
            <DialogTitle>Submit Feedback</DialogTitle>
            <DialogContent>
                <Typography>
                    Have a question or need assistance? Message us below or{' '}
                    <ExternalLink href="#">email us</ExternalLink>!
                </Typography>
                <Box
                    style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: '1rem',
                    }}
                >
                    <TextField
                        variant="outlined"
                        value={input.bodyText}
                        onChange={(e) =>
                            setInputState((prev) => ({
                                ...prev,
                                bodyText: e.target.value,
                            }))
                        }
                        label={
                            bodyTooLong
                                ? `Text is too long (${input.bodyText.length}/${MAX_FEEDBACK_LENGTH})`
                                : 'Message'
                        }
                        multiline
                        minRows={4}
                        maxRows={4}
                        fullWidth
                        error={bodyTooLong}
                    />
                    <Typography>
                        Attach logs with your feedback for better support.
                    </Typography>
                    <Box
                        style={{
                            display: 'flex',
                            flexDirection: 'row',
                            justifyContent: 'space-between',
                            gap: '1rem',
                        }}
                    >
                        <SwapSelectDropDown
                            selectedSwap={input.selectedSwap}
                            setSelectedSwap={(swapId) =>
                                setInputState((prev) => ({
                                    ...prev,
                                    selectedSwap: swapId,
                                }))
                            }
                        />
                        <Tooltip title="View the logs">
                            <Box
                                style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                }}
                            >
                                <IconButton
                                    onClick={() => setSwapLogsEditorOpen(true)}
                                    disabled={input.selectedSwap === null}
                                >
                                    <Visibility />
                                </IconButton>
                            </Box>
                        </Tooltip>
                    </Box>
                    <LogViewer
                        open={swapLogsEditorOpen}
                        setOpen={setSwapLogsEditorOpen}
                        logs={logs.swapLogs}
                        setIsRedacted={(redact) =>
                            setInputState((prev) => ({
                                ...prev,
                                isSwapLogsRedacted: redact,
                            }))
                        }
                        isRedacted={input.isSwapLogsRedacted}
                    />
                    <Box
                        style={{
                            display: 'flex',
                            flexDirection: 'row',
                            justifyContent: 'space-between',
                            gap: '1rem',
                        }}
                    >
                        <Paper
                            variant="outlined"
                            style={{ padding: '0.5rem', width: '100%' }}
                        >
                            <FormControlLabel
                                control={
                                    <Checkbox
                                        color="primary"
                                        checked={input.attachDaemonLogs}
                                        onChange={(e) =>
                                            setInputState((prev) => ({
                                                ...prev,
                                                attachDaemonLogs:
                                                    e.target.checked,
                                            }))
                                        }
                                    />
                                }
                                label="Attach logs from the current session"
                            />
                        </Paper>
                        <Tooltip title="View the logs">
                            <Box
                                style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                }}
                            >
                                <IconButton
                                    onClick={() =>
                                        setDaemonLogsEditorOpen(true)
                                    }
                                    disabled={input.attachDaemonLogs === false}
                                >
                                    <Visibility />
                                </IconButton>
                            </Box>
                        </Tooltip>
                    </Box>
                    <Typography>
                        Your feedback will be answered in the app and can be
                        found in the Feedback tab
                    </Typography>
                    <LogViewer
                        open={daemonLogsEditorOpen}
                        setOpen={setDaemonLogsEditorOpen}
                        logs={logs.daemonLogs}
                        setIsRedacted={(redact) =>
                            setInputState((prev) => ({
                                ...prev,
                                isDaemonLogsRedacted: redact,
                            }))
                        }
                        isRedacted={input.isDaemonLogsRedacted}
                    />
                </Box>
            </DialogContent>
            <DialogActions>
                <Button onClick={handleClose}>Cancel</Button>
                <PromiseInvokeButton
                    color="primary"
                    variant="contained"
                    onInvoke={submitFeedback}
                    onSuccess={handleClose}
                >
                    Submit
                </PromiseInvokeButton>
            </DialogActions>
        </Dialog>
    )
}
