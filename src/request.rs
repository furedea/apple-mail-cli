use serde_derive::Serialize;

use crate::cli::Command;

#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum MailRequest<'a> {
    Accounts,
    Mailboxes {
        account: &'a str,
    },
    Unread {
        #[serde(skip_serializing_if = "Option::is_none")]
        account: Option<&'a str>,
        limit: u16,
    },
    Search {
        query: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        account: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mailbox: Option<&'a str>,
        limit: u16,
    },
    Get {
        account: &'a str,
        mailbox: &'a str,
        id: u64,
        include_body: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_body_bytes: Option<u32>,
    },
    MarkRead {
        account: &'a str,
        mailbox: &'a str,
        id: u64,
    },
    Move {
        account: &'a str,
        mailbox: &'a str,
        id: u64,
        destination: &'a str,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum MutationPreview<'a> {
    MarkRead {
        account: &'a str,
        mailbox: &'a str,
        id: u64,
    },
    Move {
        account: &'a str,
        mailbox: &'a str,
        id: u64,
        destination: &'a str,
    },
}

impl<'a> MutationPreview<'a> {
    #[must_use]
    pub fn from_command(command: &'a Command) -> Option<Self> {
        match command {
            Command::MarkRead(args) if !args.execute() => Some(Self::MarkRead {
                account: args.locator().account(),
                mailbox: args.locator().mailbox(),
                id: args.locator().id(),
            }),
            Command::Move(args) if !args.execute() => Some(Self::Move {
                account: args.locator().account(),
                mailbox: args.locator().mailbox(),
                id: args.locator().id(),
                destination: args.destination(),
            }),
            _ => None,
        }
    }
}

impl<'a> From<&'a Command> for MailRequest<'a> {
    fn from(command: &'a Command) -> Self {
        match command {
            Command::Accounts => Self::Accounts,
            Command::Mailboxes(args) => Self::Mailboxes {
                account: args.account(),
            },
            Command::Unread(args) => Self::Unread {
                account: args.account(),
                limit: args.limit(),
            },
            Command::Search(args) => Self::Search {
                query: args.query(),
                account: args.account(),
                mailbox: args.mailbox(),
                limit: args.limit(),
            },
            Command::Get(args) => Self::Get {
                account: args.locator().account(),
                mailbox: args.locator().mailbox(),
                id: args.locator().id(),
                include_body: args.include_body(),
                max_body_bytes: args.max_body_bytes(),
            },
            Command::MarkRead(args) => Self::MarkRead {
                account: args.locator().account(),
                mailbox: args.locator().mailbox(),
                id: args.locator().id(),
            },
            Command::Move(args) => Self::Move {
                account: args.locator().account(),
                mailbox: args.locator().mailbox(),
                id: args.locator().id(),
                destination: args.destination(),
            },
        }
    }
}
