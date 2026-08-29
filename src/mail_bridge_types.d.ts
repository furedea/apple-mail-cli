type JxaValue = unknown;

interface BridgeRequest {
  account?: unknown;
  action: string;
  destination?: unknown;
  id?: unknown;
  include_body?: unknown;
  limit?: unknown;
  mailbox?: unknown;
  max_body_bytes?: unknown;
  query?: unknown;
}

interface MessageLocator {
  account: string;
  id: number;
  mailbox: string;
}

interface MessageSummary {
  account: string;
  body?: string;
  body_truncated?: boolean;
  date_received: string;
  id: number;
  mailbox: string;
  message_id: string;
  read: boolean;
  sender: string;
  size_bytes: number;
  subject: string;
}

interface MailboxSummary {
  name: string;
  path: string;
  unread_count: number;
}

interface TruncatedText {
  isTruncated: boolean;
  text: string;
}

interface BridgeErrorLike {
  bridgeCode?: unknown;
  errorNumber?: unknown;
  message?: unknown;
}

type BridgeHandler = (mail: MailApplication, request: BridgeRequest) => unknown;

interface MailNamedContainer {
  id(): JxaValue;
  name(): JxaValue;
}

type MailContainer = MailAccount | MailMailbox;

interface MailAccount extends MailNamedContainer {
  accountType(): JxaValue;
  emailAddresses(): JxaValue[];
  enabled(): JxaValue;
  mailboxes(): MailMailbox[];
}

interface MailMessageCollection {
  whose(query: Record<string, unknown>): () => MailMessage[];
}

interface MailMailbox extends MailNamedContainer {
  account(): MailAccount;
  container(): MailContainer;
  mailboxes(): MailMailbox[];
  messages: MailMessageCollection;
  unreadCount(): JxaValue;
}

interface MailMessage {
  content(): JxaValue;
  dateReceived(): JxaValue;
  id(): JxaValue;
  mailbox(): MailMailbox;
  messageId(): JxaValue;
  messageSize(): JxaValue;
  readStatus(): JxaValue;
  sender(): JxaValue;
  subject(): JxaValue;
}

interface MailApplication {
  accounts(): MailAccount[];
  inbox(): MailMailbox;
  move(message: MailMessage, options: { to: MailMailbox }): MailMessage | null;
}

declare function Application(name: "Mail"): MailApplication;
