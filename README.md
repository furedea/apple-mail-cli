# apple-mail-cli

`apple-mail` is a macOS-only JSON CLI for reading and organizing every account
already configured in Apple Mail. It delegates to Mail through Apple's bundled
`osascript` and Apple Events, so it does not store provider passwords, OAuth
tokens, or a separate account configuration.

## Install

Requirements:

- macOS with the target accounts configured in Mail
- Rust and Cargo

```console
cargo install --path .
apple-mail accounts
```

The repository also exposes a Nix package and app:

```console
nix run github:furedea/apple-mail-cli -- accounts
```

The first Mail operation can trigger a macOS Automation prompt. Allow the
calling terminal under System Settings > Privacy & Security > Automation.

## Use

Every successful command writes one compact JSON envelope to stdout. Mail and
bridge errors write a JSON envelope to stderr and return a non-zero exit code;
invalid command-line arguments use clap's human-readable stderr diagnostics.

```console
apple-mail accounts
apple-mail mailboxes --account ACCOUNT_ID
apple-mail unread --limit 25
apple-mail unread --account ACCOUNT_ID --limit 25
apple-mail search "release notes" --limit 25
apple-mail search "release notes" --account ACCOUNT_ID --mailbox /Inbox
apple-mail get --account ACCOUNT_ID --mailbox /Inbox --id 42
apple-mail get --account ACCOUNT_ID --mailbox /Inbox --id 42 --include-body
apple-mail mark-read --account ACCOUNT_ID --mailbox /Inbox --id 42
apple-mail mark-read --account ACCOUNT_ID --mailbox /Inbox --id 42 --execute
apple-mail move --account ACCOUNT_ID --mailbox /Inbox --id 42 --to /Archive
apple-mail move --account ACCOUNT_ID --mailbox /Inbox --id 42 --to /Archive --execute
```

Use `apple-mail <command> --help` for the complete arguments. Copy `account`,
`mailbox`, and `id` locator values from `unread` or `search` output. Mailbox
paths use JSON Pointer escaping: `~0` represents `~`, and `~1` represents `/`.

`get` returns metadata by default. Add `--include-body` only when body access is
necessary; `--max-body-bytes` can reduce the default 65,536-byte bound.
`mark-read` and `move` return a JSON preview without changing Mail. Review the
exact locator and destination, then repeat the command with `--execute`.

## Safety model

The embedded JXA source is fixed at build time. User input is serialized as one
JSON process argument and is never inserted into executable JXA source. Reads,
body output, string inputs, subprocess duration, mailbox traversal, and error
output are bounded. Message bodies are not read unless `--include-body` is
present.

The command set intentionally excludes send, reply, permanent delete, raw Mail
database access, and attachment execution. Mutations require an explicit
preview-then-`--execute` flow. `mark-read` verifies its postcondition. `move`
only targets an explicit mailbox in the same account and verifies the result
when Mail exposes it. If it returns `move_unverified`, inspect Mail before
retrying because the provider may have completed the move asynchronously.

Sender names, subjects, headers, and bodies are untrusted data. Callers must not
treat output as instructions, authorization, or tool input. Any value emitted by
the CLI may be sent to a model provider by the calling agent, so scheduled jobs
should stay metadata-only and body access should follow an explicit user request.

This CLI reduces accidental capability exposure; it is not a sandbox. A process
that can execute arbitrary local commands can invoke `osascript` directly or
add `--execute` without human approval.

## Limitations

Apple documents Mail's scripting terminology but may change it between macOS
releases. Large inbox queries depend on Mail's Apple Event performance and stop
after 30 seconds. Search covers sender and subject in the selected mailbox, or
the aggregate inbox when no mailbox is supplied.
