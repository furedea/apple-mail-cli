# ADR-0003: Require explicit data and mutation access

- Status: Accepted
- Date: 2026-08-28

In the context of using Apple Mail through an LLM-driven secretary, facing
accidental disclosure of message bodies and unintended mailbox changes, we
decided for metadata-only reads by default and preview-before-execute mutations
and against implicit body reads and immediate mutations, to reduce unnecessary
data exposure and authority use, accepting extra user-visible flags and round
trips without treating them as a sandbox against arbitrary local processes.
