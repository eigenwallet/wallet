import {
  Box,
  Typography,
  makeStyles,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  IconButton,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Paper,
  List,
  ListItem,
  ListItemText,
  Chip,
  Badge,
  TextField,
  CircularProgress,
} from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { useState, useEffect, useMemo, useCallback } from "react";
import { useAppSelector, useUnreadMessagesCount, useAppDispatch } from "store/hooks";
import { PrimitiveDateTimeString } from "renderer/api";
import logger from "utils/logger";
import { useSnackbar } from "notistack";
import TruncatedText from "renderer/components/other/TruncatedText";
import { Message } from "renderer/api";
import { markMessagesAsSeen } from "store/features/conversationsSlice";
import ChatIcon from '@material-ui/icons/Chat';
import SendIcon from '@material-ui/icons/Send';
import { appendFeedbackMessageViaHttp, fetchAllConversations } from "renderer/api";

const useStyles = makeStyles((theme) => ({
  content: {
    display: "flex",
    flexDirection: "column",
    alignItems: "flex-start",
    gap: theme.spacing(2),
  },
  tableContainer: {
    maxHeight: 300, // Limit height and make scrollable
  },
  messageBox: {
    marginBottom: theme.spacing(1),
    padding: theme.spacing(1),
    borderRadius: theme.shape.borderRadius,
    maxWidth: '80%',
  },
  userMessage: {
    backgroundColor: theme.palette.primary.light,
    marginLeft: 'auto', // Align user messages to the right
    textAlign: 'right',
  },
  staffMessage: {
    backgroundColor: theme.palette.primary.light,
    marginRight: 'auto', // Align staff messages to the left
  },
  inputArea: {
    display: 'flex',
    alignItems: 'center',
    marginTop: theme.spacing(2),
    padding: theme.spacing(1),
  },
  inputField: {
    flexGrow: 1,
    marginRight: theme.spacing(1),
  },
}));

// Define break tokens at the file root
const breakTokens = ["\n\nDaemon Logs:"];

// Helper function to format PrimitiveDateTimeString
function formatDateTime(dateTime: PrimitiveDateTimeString): string {
  try {
    // Assuming the tuple represents [YYYY, MM, DD, HH, MM, NANOSEC]
    // Nanoseconds need careful handling for Date object
    const [year, month, day, hour, minute, nanoseconds] = dateTime;
    const seconds = Math.floor(nanoseconds / 1_000_000_000);
    const milliseconds = Math.floor((nanoseconds % 1_000_000_000) / 1_000_000);

    // Date constructor uses 0-indexed month
    const date = new Date(Date.UTC(year, month - 1, day, hour, minute, seconds, milliseconds));

    if (isNaN(date.getTime())) {
      return "Invalid Date";
    }

    // Format to a readable string (e.g., "YYYY-MM-DD HH:MM:SS UTC")
    return date.toISOString().replace('T', ' ').replace(/\.\d+Z$/, ' UTC');

  } catch (error) {
    logger.error(error, "Error formatting datetime", { dateTime });
    return "Invalid Date";
  }
}

// New component for rendering a single row
function ConversationRow({ feedbackId, onOpenModal }: { feedbackId: string; onOpenModal: (id: string) => void }) {
  const unreadCount = useUnreadMessagesCount(feedbackId);
  // Fetch messages for this specific conversation
  const messages = useAppSelector((state) => 
    state.conversations.conversations[feedbackId] || []
  );

  // Sort messages to find the oldest one (first message)
  const sortedMessages = useMemo(() => [...messages].sort((a, b) => { 
    try {
      const dateA = new Date(formatDateTime(a.created_at)).getTime();
      const dateB = new Date(formatDateTime(b.created_at)).getTime();
      if (isNaN(dateA)) return 1;
      if (isNaN(dateB)) return -1;
      return dateA - dateB; // Ascending order
    } catch (e) {
      return 0;
    }
  }), [messages]);

  const firstMessageContent = sortedMessages.length > 0 ? sortedMessages[0].content : "No messages yet";

  // Find the earliest break token index
  let earliestBreakIndex = firstMessageContent.length; // Default to full length
  for (const token of breakTokens) {
    const index = firstMessageContent.indexOf(token);
    if (index !== -1 && index < earliestBreakIndex) {
      earliestBreakIndex = index;
    }
  }

  // Slice the content if a break token was found
  const previewContent = firstMessageContent.substring(0, earliestBreakIndex);

  return (
    <TableRow key={feedbackId}>
      <TableCell>
        <TruncatedText limit={7}>{feedbackId}</TruncatedText>
      </TableCell>
      <TableCell> {/* New cell for preview */}
        <TruncatedText limit={30}>{previewContent}</TruncatedText> {/* Use pre-sliced content */}
      </TableCell>
      <TableCell align="right">
        <Badge
          badgeContent={unreadCount}
          color="primary"
          overlap="rectangular" // Use "rectangular" as per deprecation warning
          invisible={unreadCount === 0}
        >
          <IconButton size="small" onClick={() => onOpenModal(feedbackId)} title="Open Conversation">
            <ChatIcon />
          </IconButton>
        </Badge>
      </TableCell>
    </TableRow>
  );
}

export default function ConversationsBox() {
  const classes = useStyles();
  // Select the list of known feedback IDs
  const knownFeedbackIds = useAppSelector((state) => state.conversations.knownFeedbackIds || []); 
  // Select the conversations map (used by modal)
  // const conversationsMap = useAppSelector((state) => state.conversations.conversations);

  // State to hold the ID of the feedback for the modal
  const [selectedFeedbackId, setSelectedFeedbackId] = useState<string | null>(null);

  const handleOpenModal = (feedbackId: string) => {
    setSelectedFeedbackId(feedbackId);
  };

  const handleCloseModal = () => {
    setSelectedFeedbackId(null);
  };

  // Sort IDs if needed (e.g., based on data fetched elsewhere or just keep order)
  // For now, we don't have creation dates here, so we use the order from the store.
  const sortedFeedbackIds = knownFeedbackIds; // Or apply sorting if possible later

  return (
    <InfoBox
      title="Your Feedback Conversations"
      icon={null}
      loading={false} // Loading is handled by the fetchAllConversations in background
      mainContent={
        <Box className={classes.content}>
          <Typography variant="subtitle2">
            View your past feedback submissions and any replies from the development team.
          </Typography>
          {sortedFeedbackIds.length === 0 ? (
            <Typography variant="body2">No feedback submitted yet.</Typography>
          ) : (
            <TableContainer component={Paper} className={classes.tableContainer}>
              <Table stickyHeader size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>Feedback ID</TableCell>
                    <TableCell>Preview</TableCell>
                    <TableCell align="right">Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {sortedFeedbackIds.map((feedbackId) => (
                    <ConversationRow
                      key={feedbackId}
                      feedbackId={feedbackId}
                      onOpenModal={handleOpenModal}
                    />
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </Box>
      }
      additionalContent={
        selectedFeedbackId && (
          <ConversationModal
            open={selectedFeedbackId !== null}
            onClose={handleCloseModal}
            feedbackId={selectedFeedbackId} // Pass ID instead of Feedback object
          />
        )
      }
    />
  );
}


function ConversationModal({
  open,
  onClose,
  feedbackId,
}: {
  open: boolean;
  onClose: () => void;
  feedbackId: string;
}) {
  const classes = useStyles();
  const dispatch = useAppDispatch(); // Get dispatch function
  const [newMessage, setNewMessage] = useState(""); // State for the new message input
  const [isSending, setIsSending] = useState(false); // State for loading indicator
  // Select messages directly from the Redux store
  const messages = useAppSelector((state) => 
    state.conversations.conversations[feedbackId] || [] // Default to empty array if undefined
  );
  const seenMessagesSet = useAppSelector((state) => 
    new Set(state.conversations.seenMessages)
  );
  const { enqueueSnackbar } = useSnackbar(); // Keep for potential future use

  // Effect to mark messages as seen when modal opens
  useEffect(() => {
    if (open && messages.length > 0) {
      const unseenMessages = messages.filter(
        (msg) => !seenMessagesSet.has(msg.id.toString())
      );

      if (unseenMessages.length > 0) {
        dispatch(markMessagesAsSeen(unseenMessages));
      }
    }
    // Dependency array includes open and messages array reference
  }, [open, messages, dispatch, seenMessagesSet]); 

  // Sort messages from the store (descending for display)
  const sortedMessagesForDisplay = useMemo(() => [...messages].sort((a, b) => { 
    try {
      const dateA = new Date(formatDateTime(a.created_at)).getTime();
      const dateB = new Date(formatDateTime(b.created_at)).getTime();
      if (isNaN(dateA)) return 1;
      if (isNaN(dateB)) return -1;
      return dateB - dateA;
    } catch (e) {
      return 0;
    }
  }), [messages]);

  // Handle sending the message
  const handleSendMessage = useCallback(async () => {
    if (!newMessage.trim()) return; // Don't send empty messages

    setIsSending(true);
    try {
      await appendFeedbackMessageViaHttp(feedbackId, newMessage);
      setNewMessage(""); // Clear input field
      enqueueSnackbar("Message sent successfully!", { variant: "success" });
      // Refresh conversations to show the new message
      // Using fetchAllConversations for simplicity, could optimize later
      dispatch(fetchAllConversations as any); // Dispatch the thunk
    } catch (error) {
      logger.error(error, "Failed to send message", { feedbackId });
      enqueueSnackbar("Failed to send message. Please try again.", { variant: "error" });
    } finally {
      setIsSending(false);
    }
  }, [feedbackId, newMessage, dispatch, enqueueSnackbar]);

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth scroll="paper">
      <DialogTitle>Conversation <TruncatedText limit={8} truncateMiddle={true}>{feedbackId}</TruncatedText></DialogTitle> {/* Use feedbackId */}
      <DialogContent dividers>
        {sortedMessagesForDisplay.length === 0 && (
          <Typography variant="body2">No messages loaded for this conversation.</Typography>
        )}
        {sortedMessagesForDisplay.length > 0 && (
           <List>
             {sortedMessagesForDisplay.map((msg) => { // Use sortedMessages
                // Find the index of "Daemon Logs:"
                const daemonLogsIndex = msg.content.indexOf("\n\nDaemon Logs:");
                // Truncate the content if "Daemon Logs:" is found
                const displayContent = daemonLogsIndex !== -1 
                    ? msg.content.substring(0, daemonLogsIndex)
                    : msg.content;

                const isNew = msg.id > 1_000_000; // Check if id is potentially temporary client-side id

                return (
                  <ListItem key={msg.id} disableGutters>
                    <Paper
                      className={`${classes.messageBox} ${msg.is_from_staff ? classes.staffMessage : classes.userMessage}`}
                      elevation={1}
                    >
                      <ListItemText
                        primary={displayContent} // Use the truncated content
                        secondary={`${msg.is_from_staff ? 'Staff' : 'You'} - ${formatDateTime(msg.created_at)}`}
                        style={{ whiteSpace: 'pre-wrap' }} // Preserve line breaks in message content
                      />
                    </Paper>
                  </ListItem>
                );
              })}
           </List>
        )}
        {/* Add input field and button to send a new message */}
        <Box className={classes.inputArea}>
          <TextField
            className={classes.inputField}
            variant="outlined"
            size="small"
            multiline
            rowsMax={4}
            placeholder="Type your message..."
            value={newMessage}
            onChange={(e) => setNewMessage(e.target.value)}
            disabled={isSending}
          />
          <IconButton 
            color="primary"
            onClick={handleSendMessage} 
            disabled={!newMessage.trim() || isSending}
          >
            {isSending ? <CircularProgress size={24} /> : <SendIcon />} {/* Show spinner when sending */}
          </IconButton>
        </Box>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} color="primary" variant="contained">
          Close
        </Button>
      </DialogActions>
    </Dialog>
  );
}
