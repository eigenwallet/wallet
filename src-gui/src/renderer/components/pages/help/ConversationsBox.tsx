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
} from "@material-ui/core";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { useState, useEffect } from "react";
import { useAppSelector, useUnreadMessagesCount, useAppDispatch } from "store/hooks";
import { PrimitiveDateTimeString } from "renderer/api";
import logger from "utils/logger";
import OpenInNewIcon from '@material-ui/icons/OpenInNew';
import { useSnackbar } from "notistack";
import TruncatedText from "renderer/components/other/TruncatedText";
import { Message } from "renderer/api";
import { markMessagesAsSeen } from "store/features/conversationsSlice";
import ChatIcon from '@material-ui/icons/Chat';
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
}));

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
                    {/* Removed Submitted column as created_at is not directly available */}
                    {/* <TableCell>Submitted</TableCell> */}
                    <TableCell>Feedback ID</TableCell>
                    <TableCell align="right">Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {sortedFeedbackIds.map((feedbackId) => {
                    // Get unread count for this conversation
                    const unreadCount = useUnreadMessagesCount(feedbackId);
                    
                    return (
                      <TableRow key={feedbackId}>
                        <TableCell>
                          {feedbackId}
                        </TableCell>
                        <TableCell align="right">
                          <Badge 
                            badgeContent={unreadCount} 
                            color="primary" 
                            invisible={unreadCount === 0}
                          >
                            <IconButton size="small" onClick={() => handleOpenModal(feedbackId)} title="Open Conversation">
                              <ChatIcon />
                            </IconButton>
                          </Badge>
                        </TableCell>
                      </TableRow>
                    );
                  })}
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

  // Sort messages from the store
  const sortedMessages = [...messages].sort((a, b) => { // Apply sort to selected messages
    try {
      const dateA = new Date(formatDateTime(a.created_at)).getTime();
      const dateB = new Date(formatDateTime(b.created_at)).getTime();
      if (isNaN(dateA)) return 1; // Treat invalid dates as older
      if (isNaN(dateB)) return -1;
      return dateB - dateA;
    } catch (e) {
      return 0;
    }
  });

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth scroll="paper">
      <DialogTitle>Conversation <TruncatedText children={feedbackId} /></DialogTitle> {/* Use feedbackId */}
      <DialogContent dividers>
        {sortedMessages.length === 0 && (
          <Typography variant="body2">No messages loaded for this conversation.</Typography>
        )}
        {sortedMessages.length > 0 && (
           <List>
             {sortedMessages.map((msg) => { // Use sortedMessages
                // Find the index of "Daemon Logs:"
                const daemonLogsIndex = msg.content.indexOf("\n\nDaemon Logs:");
                // Truncate the content if "Daemon Logs:" is found
                const displayContent = daemonLogsIndex !== -1 
                    ? msg.content.substring(0, daemonLogsIndex)
                    : msg.content;

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
        {/* TODO: Add input field and button to send a new message? */}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} color="primary" variant="contained">
          Close
        </Button>
      </DialogActions>
    </Dialog>
  );
}
