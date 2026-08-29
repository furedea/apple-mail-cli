use apple_mail_cli::cli::{Cli, Command};
use apple_mail_cli::request::MailRequest;
use clap::Parser;
use serde_json::json;

#[test]
fn accounts_command_is_accepted_without_options() {
    let cli =
        Cli::try_parse_from(["apple-mail", "accounts"]).expect("accounts command should parse");

    assert!(matches!(cli.command(), Command::Accounts));
}

#[test]
fn mailboxes_command_requires_an_account_identifier() {
    let cli = Cli::try_parse_from(["apple-mail", "mailboxes", "--account", "account-1"])
        .expect("mailboxes command should parse with an account identifier");

    assert!(matches!(cli.command(), Command::Mailboxes(_)));
}

#[test]
fn unread_command_defaults_to_twenty_five_messages() {
    let cli = Cli::try_parse_from(["apple-mail", "unread"])
        .expect("unread command should parse without options");
    let Command::Unread(args) = cli.command() else {
        panic!("expected unread command");
    };

    assert_eq!(args.limit(), 25);
}

#[test]
fn unread_command_rejects_a_limit_above_two_hundred() {
    let error = Cli::try_parse_from(["apple-mail", "unread", "--limit", "201"])
        .expect_err("an unbounded unread request must be rejected");

    assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
}

#[test]
fn search_command_accepts_a_query_and_optional_scope() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "search",
        "release notes",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox/Project",
        "--limit",
        "10",
    ])
    .expect("search command should parse with a bounded scope");

    assert!(matches!(cli.command(), Command::Search(_)));
}

#[test]
fn search_command_rejects_a_query_above_the_bridge_limit() {
    let query = "x".repeat(1_025);
    let error = Cli::try_parse_from(["apple-mail", "search", &query])
        .expect_err("an oversized query must be rejected before spawning the bridge");

    assert_eq!(error.kind(), clap::error::ErrorKind::ValueValidation);
}

#[test]
fn get_command_requires_a_complete_message_locator() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "get",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
    ])
    .expect("get command should parse with a complete message locator");

    assert!(matches!(cli.command(), Command::Get(_)));
}

#[test]
fn mark_read_command_requires_a_complete_message_locator() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "mark-read",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
    ])
    .expect("mark-read command should parse with a complete message locator");

    assert!(matches!(cli.command(), Command::MarkRead(_)));
}

#[test]
fn move_command_requires_an_explicit_destination_mailbox() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "move",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
        "--to",
        "Archive",
    ])
    .expect("move command should parse with an explicit destination");

    assert!(matches!(cli.command(), Command::Move(_)));
}

#[test]
fn accounts_command_maps_to_the_accounts_bridge_request() {
    let cli =
        Cli::try_parse_from(["apple-mail", "accounts"]).expect("accounts command should parse");

    let request = serde_json::to_value(MailRequest::from(cli.command()))
        .expect("bridge request should serialize");

    assert_eq!(request, json!({"action": "accounts"}));
}

#[test]
fn mailboxes_command_maps_its_account_to_the_bridge_request() {
    let cli = Cli::try_parse_from(["apple-mail", "mailboxes", "--account", "account-1"])
        .expect("mailboxes command should parse");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({"action": "mailboxes", "account": "account-1"}),
    );
}

#[test]
fn unread_command_omits_an_unspecified_account_from_the_bridge_request() {
    let cli = Cli::try_parse_from(["apple-mail", "unread"]).expect("unread command should parse");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({"action": "unread", "limit": 25}),
    );
}

#[test]
fn search_command_maps_query_scope_and_limit_to_the_bridge_request() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "search",
        "release notes",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox/Project",
        "--limit",
        "10",
    ])
    .expect("search command should parse");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({
            "action": "search",
            "query": "release notes",
            "account": "account-1",
            "mailbox": "Inbox/Project",
            "limit": 10,
        }),
    );
}

#[test]
fn get_command_omits_the_message_body_by_default() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "get",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
    ])
    .expect("get command should parse");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({
            "action": "get",
            "account": "account-1",
            "mailbox": "Inbox",
            "id": 42,
            "include_body": false,
        }),
    );
}

#[test]
fn get_command_includes_a_bounded_body_only_when_requested() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "get",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
        "--include-body",
        "--max-body-bytes",
        "4096",
    ])
    .expect("get command should accept explicit body access");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({
            "action": "get",
            "account": "account-1",
            "mailbox": "Inbox",
            "id": 42,
            "include_body": true,
            "max_body_bytes": 4096,
        }),
    );
}

#[test]
fn get_command_bounds_an_explicit_body_when_no_bound_is_supplied() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "get",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
        "--include-body",
    ])
    .expect("get command should accept body access with the default bound");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({
            "action": "get",
            "account": "account-1",
            "mailbox": "Inbox",
            "id": 42,
            "include_body": true,
            "max_body_bytes": 65_536,
        }),
    );
}

#[test]
fn get_command_rejects_a_body_bound_without_body_access() {
    let error = Cli::try_parse_from([
        "apple-mail",
        "get",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
        "--max-body-bytes",
        "4096",
    ])
    .expect_err("a body bound without body access must be rejected");

    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn executing_mark_read_maps_its_locator_to_the_bridge_request() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "mark-read",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
        "--execute",
    ])
    .expect("mark-read command should parse");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({
            "action": "mark-read",
            "account": "account-1",
            "mailbox": "Inbox",
            "id": 42,
        }),
    );
}

#[test]
fn executing_move_maps_locator_and_destination_to_the_bridge_request() {
    let cli = Cli::try_parse_from([
        "apple-mail",
        "move",
        "--account",
        "account-1",
        "--mailbox",
        "Inbox",
        "--id",
        "42",
        "--to",
        "Archive/2026",
        "--execute",
    ])
    .expect("move command should parse");
    let request = MailRequest::from(cli.command());

    assert_eq!(
        serde_json::to_value(request).expect("bridge request should serialize"),
        json!({
            "action": "move",
            "account": "account-1",
            "mailbox": "Inbox",
            "id": 42,
            "destination": "Archive/2026",
        }),
    );
}
