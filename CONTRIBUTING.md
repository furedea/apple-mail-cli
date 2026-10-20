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
tags the merge commit, and creates a draft GitHub Release. The `Release Please`
workflow then starts the `Release Artifacts` workflow on that tag, which tests
and builds the `aarch64-apple-darwin` cargo-dist archive, attests its
provenance, and uploads the archive, its checksum, and a shell installer. The
release is published only after every step succeeds; if one fails, it stays a
draft.

To resume a draft release whose artifact run failed, start the same workflow on
the tag so the attestation records the tagged commit:

```bash
tag=apple-mail-cli-v0.1.1
gh workflow run release_artifacts.yml --ref "$tag" -f tag_name="$tag"
```

The workflow fails before building when it is started from any other ref or
when the release is already published. A published release is never rebuilt in
place; ship a fix as a new release.
