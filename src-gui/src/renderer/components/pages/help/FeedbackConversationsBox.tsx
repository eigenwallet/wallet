import {
  Button,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableRow,
  Typography,
} from "@material-ui/core";
import { useEffect, useState } from "react";
import InfoBox from "renderer/components/modal/swap/InfoBox";
import { getSumittedFeedbackIds } from "store/tauriStore";

function ConversationsTable() {
  const [conversations, setConversations] = useState<string[]>([]);

  useEffect(() => {
    getSumittedFeedbackIds().then((ids) => {
      setConversations(ids);
    });
  }, []);

  return (
    <Table>
      <TableHead>
        <TableRow>
          <TableCell>ID</TableCell>
          <TableCell>Response</TableCell>
        </TableRow>
      </TableHead>
      <TableBody>
        {conversations.map((conversation) => (
          <TableRow key={conversation}>
            <TableCell>{conversation}</TableCell>
            <TableCell>
              <Button variant="contained" size="small">
                View reply
              </Button>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}

export default function FeedbackConversationsBox() {
  return (
    <InfoBox
      title="Feedback"
      icon={null}
      mainContent={
        <Typography variant="subtitle2">
          If the core developers have responded to your feedback, you can view
          the conversation here.
        </Typography>
      }
      additionalContent={<ConversationsTable />}
      loading={false}
    />
  );
}
