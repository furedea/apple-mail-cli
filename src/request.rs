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
        max_body_bytes: u32,
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
                max_body_bytes: args.max_body_bytes(),
            },
            Command::MarkRead(locator) => Self::MarkRead {
                account: locator.account(),
                mailbox: locator.mailbox(),
                id: locator.id(),
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
