# Contributing

## Development environment

Clone the repository and enter the Nix development shell on Apple Silicon
macOS:

```bash
git clone https://github.com/furedea/apple-mail-cli
cd apple-mail-cli
nix develop
pnpm install --frozen-lockfile
lefthook install
```

## Running local changes

An `apple-mail` command on `PATH` may refer to a previously installed release
build. Run the current checkout through Cargo when developing or reviewing local
changes:

```bash
cargo run -- accounts
cargo run -- unread --limit 5
```

Use the local flake when the Nix package itself must be verified:

```bash
nix run . -- accounts
```

## Quality gates

Run the Rust and Nix gates:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
nix flake check
```

Run the embedded JXA format, lint, and type checks:

```bash
pnpm run check
```

CI additionally runs Nix linting, dependency audits, GitHub Actions linting,
dependency review, and CodeQL.

## Release

Releases are managed by Release Please. Merging the release PR updates
`Cargo.toml`, `Cargo.lock`, `.release-please-manifest.json`, and `CHANGELOG.md`,
then publishes a GitHub Release with the `aarch64-apple-darwin` cargo-dist
archive, its checksum, a shell installer, and build provenance attestations.

If the release assets need to be rebuilt for an existing tag, run the
`Release Please` workflow manually with the tag name, for example:

```text
apple-mail-cli-v0.1.1
```
