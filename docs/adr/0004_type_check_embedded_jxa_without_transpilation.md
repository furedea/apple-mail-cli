# ADR-0004: Type-check embedded JXA without transpilation

- Status: Accepted
- Date: 2026-08-29

In the context of maintaining a fixed JXA bridge that Apple executes as
JavaScript, facing defects that Rust cannot detect inside the embedded source,
we decided for TypeScript `checkJs` with local Mail boundary declarations and
against transpilation, generated JavaScript, and third-party JXA type packages,
to add strict static checks while preserving one auditable runtime artifact and
a Cargo-only build, accepting JSDoc annotations and locally maintained types for
Mail's dynamic scripting terminology.
