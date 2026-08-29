# ADR-0005: Use pnpm quality tools for embedded JXA

- Status: Accepted
- Date: 2026-08-30

In the context of maintaining hand-written JXA beside a Rust CLI, facing
JavaScript correctness defects and formatting drift that type checking alone
does not detect, we decided for a minimal pnpm-locked TypeScript, Oxfmt, and
Oxlint development toolchain adapted from `template-typescript` and against
copying its Node runtime types, Vitest, Knip, and experimental type-aware Oxlint,
to obtain current reproducible checks without changing the Cargo build or CLI
runtime, accepting Node.js and pnpm as development-only dependencies.
