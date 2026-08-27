# ADR-0002: Use an embedded JXA bridge for Apple Mail

- Status: Accepted
- Date: 2026-08-28

In the context of automating multiple provider accounts already configured in
Apple Mail, facing credential duplication and the instability of Mail's private
database, we decided for a Rust CLI invoking a fixed embedded JXA program through
Apple's `osascript` and Apple Events and against direct database access, provider
APIs, MCP servers, and third-party Mail wrappers, to reuse the system account and
permission boundary without new credentials, accepting Automation permission,
Mail.app availability, Apple Event performance, and scripting-dictionary changes.
