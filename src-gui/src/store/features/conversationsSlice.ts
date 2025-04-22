import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Feedback, Message } from "../../renderer/api"; // Import Feedback type

export interface ConversationsSlice {
  // List of feedback IDs we know of
  knownFeedbackIds: string[];
  // Maps feedback IDs to conversations
  conversations: {
    [key: string]: Message[];
  };
}

const initialState: ConversationsSlice = {
  knownFeedbackIds: [],
  conversations: {},
};

const conversationsSlice = createSlice({
  name: "conversations",
  initialState,
  reducers: {
    addFeedbackId(slice, action: PayloadAction<string>) {
      slice.knownFeedbackIds.push(action.payload);
    },
    // Removes a feedback id from the list of known ones
    // Also removes the conversation from the store
    removeFeedback(slice, action: PayloadAction<string>) {
      slice.knownFeedbackIds = slice.knownFeedbackIds.filter(
        (id) => id !== action.payload,
      );
      delete slice.conversations[action.payload];
    },
    // Sets the conversations for a given feedback id
    setConversation(slice, action: PayloadAction<{feedbackId: string, messages: Message[]}>) {
      slice.conversations[action.payload.feedbackId] = action.payload.messages;
    },
  },
});

export const { addFeedbackId, removeFeedback, setConversation } = conversationsSlice.actions;
export default conversationsSlice.reducer;
