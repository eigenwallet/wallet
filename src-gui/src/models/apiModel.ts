export interface ExtendedMakerStatus extends MakerStatus {
  uptime?: number;
  age?: number;
  relevancy?: number;
  version?: string;
  recommended?: boolean;
}

export interface MakerStatus extends MakerQuote, Maker { }

export interface MakerQuote {
  price: number;
  minSwapAmount: number;
  maxSwapAmount: number;
}

export interface Maker {
  multiAddr: string;
  testnet: boolean;
  peerId: string;
}

export interface Alert {
  id: number;
  title: string;
  body: string;
  severity: "info" | "warning" | "error";
}

// Corresponds to Rust's PrimitiveDateTime
export type PrimitiveDateTimeString = [number, number, number, number, number, number]; 

// Corresponds to Rust's Uuid
export type UuidString = string;

export interface Feedback {
  id: UuidString;
  created_at: PrimitiveDateTimeString;
}

export interface Attachment {
  id: number; 
  message_id: number;
  content: string;
  created_at: PrimitiveDateTimeString;
}

export interface Message {
  id: number;
  feedback_id: UuidString;
  is_from_staff: boolean;
  content: string;
  created_at: PrimitiveDateTimeString;
}

export interface MessageWithAttachments {
  message: Message;
  attachments: Attachment[];
}
