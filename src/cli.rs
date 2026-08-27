use std::str::FromStr;

use clap::{Parser, Subcommand};

const MAX_ACCOUNT_BYTES: usize = 1_024;
const MAX_MAILBOX_BYTES: usize = 4_096;
const MAX_QUERY_BYTES: usize = 1_024;

#[derive(Debug, Parser)]
#[command(
    name = "apple-mail",
    version,
    about = "Read and organize accounts configured in Apple Mail"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List every account configured in Apple Mail.
    Accounts,
    /// List mailbox paths for one account.
    Mailboxes(MailboxesArgs),
    /// List unread messages in the aggregate inbox.
    Unread(UnreadArgs),
    /// Search sender and subject in an inbox or mailbox.
    Search(SearchArgs),
    /// Read one message body from a locator.
    Get(GetArgs),
    /// Mark one located message as read.
    MarkRead(MessageLocatorArgs),
    /// Move one located message to an explicit mailbox in the same account.
    Move(MoveArgs),
}

#[derive(Debug, clap::Args)]
pub struct MailboxesArgs {
    /// Account identifier returned by `accounts`.
    #[arg(long)]
    account: BoundedText<MAX_ACCOUNT_BYTES>,
}

impl MailboxesArgs {
    #[must_use]
    pub fn account(&self) -> &str {
        self.account.as_str()
    }
}

#[derive(Debug, clap::Args)]
pub struct UnreadArgs {
    /// Restrict results to one account identifier.
    #[arg(long)]
    account: Option<BoundedText<MAX_ACCOUNT_BYTES>>,

    /// Maximum number of messages to return.
    #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u16).range(1..=200))]
    limit: u16,
}

impl UnreadArgs {
    #[must_use]
    pub fn account(&self) -> Option<&str> {
        self.account.as_ref().map(BoundedText::as_str)
    }

    #[must_use]
    pub const fn limit(&self) -> u16 {
        self.limit
    }
}

#[derive(Debug, clap::Args)]
pub struct SearchArgs {
    /// Text to find in a message sender or subject.
    query: BoundedText<MAX_QUERY_BYTES>,

    /// Restrict results to one account identifier.
    #[arg(long)]
    account: Option<BoundedText<MAX_ACCOUNT_BYTES>>,

    /// Restrict results to a mailbox path returned by `mailboxes`.
    #[arg(long, requires = "account")]
    mailbox: Option<BoundedText<MAX_MAILBOX_BYTES>>,

    /// Maximum number of messages to return.
    #[arg(long, default_value_t = 25, value_parser = clap::value_parser!(u16).range(1..=200))]
    limit: u16,
}

impl SearchArgs {
    #[must_use]
    pub fn query(&self) -> &str {
        self.query.as_str()
    }

    #[must_use]
    pub fn account(&self) -> Option<&str> {
        self.account.as_ref().map(BoundedText::as_str)
    }

    #[must_use]
    pub fn mailbox(&self) -> Option<&str> {
        self.mailbox.as_ref().map(BoundedText::as_str)
    }

    #[must_use]
    pub const fn limit(&self) -> u16 {
        self.limit
    }
}

#[derive(Debug, clap::Args)]
pub struct GetArgs {
    #[command(flatten)]
    locator: MessageLocatorArgs,

    /// Maximum UTF-8 bytes of message body to return.
    #[arg(long, default_value_t = 65_536, value_parser = clap::value_parser!(u32).range(1..=1_000_000))]
    max_body_bytes: u32,
}

impl GetArgs {
    #[must_use]
    pub const fn locator(&self) -> &MessageLocatorArgs {
        &self.locator
    }

    #[must_use]
    pub const fn max_body_bytes(&self) -> u32 {
        self.max_body_bytes
    }
}

#[derive(Debug, clap::Args)]
pub struct MessageLocatorArgs {
    /// Account identifier copied from message output.
    #[arg(long)]
    account: BoundedText<MAX_ACCOUNT_BYTES>,

    /// Mailbox path copied from message output.
    #[arg(long)]
    mailbox: BoundedText<MAX_MAILBOX_BYTES>,

    /// Numeric Mail message identifier copied from message output.
    #[arg(long)]
    id: u64,
}

impl MessageLocatorArgs {
    #[must_use]
    pub fn account(&self) -> &str {
        self.account.as_str()
    }

    #[must_use]
    pub fn mailbox(&self) -> &str {
        self.mailbox.as_str()
    }

    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug, clap::Args)]
pub struct MoveArgs {
    #[command(flatten)]
    locator: MessageLocatorArgs,

    /// Destination mailbox path returned by `mailboxes`.
    #[arg(long = "to")]
    destination: BoundedText<MAX_MAILBOX_BYTES>,
}

impl MoveArgs {
    #[must_use]
    pub const fn locator(&self) -> &MessageLocatorArgs {
        &self.locator
    }

    #[must_use]
    pub fn destination(&self) -> &str {
        self.destination.as_str()
    }
}

#[derive(Debug, Clone)]
struct BoundedText<const MAX_BYTES: usize>(String);

impl<const MAX_BYTES: usize> BoundedText<MAX_BYTES> {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl<const MAX_BYTES: usize> FromStr for BoundedText<MAX_BYTES> {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err("value must not be empty".to_owned());
        }
        if value.len() > MAX_BYTES {
            return Err(format!("value must not exceed {MAX_BYTES} bytes"));
        }
        Ok(Self(value.to_owned()))
    }
}
