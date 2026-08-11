# Contributing

Start with the [architecture overview](docs/ARCHITECTURE.md), then read the user-facing documentation for any workflow you plan to change. Public behavior, supported inputs, privacy boundaries, and roadmap status must remain consistent across code and documentation.

## Development Workflow

1. Create a focused branch from `main`.
2. Install dependencies with `npm ci`.
3. Keep filesystem, archive, SQLite, and export behavior in Rust; keep presentation and transient UI state in React.
4. Add or update tests for behavior changes.
5. Run `npm run check`. For documentation or visible UI changes, also run `npm run docs:check`.
6. Open a pull request using the repository template.

The repository is one Cargo workspace with one root `Cargo.lock`. The checked-in
`rust-toolchain.toml` is authoritative; do not create package-local lockfiles or
silently substitute another Rust release in CI.

For changes that affect query or import scalability, also run the opt-in
250,000-account capacity regression:

```bash
cargo test --package nivune --lib handles_250k_unique_accounts_in_one_snapshot -- --ignored
```

## Pull Requests

- Keep changes scoped and explain user-visible or compatibility effects.
- Do not commit Instagram exports, SQLite databases, login secrets, or other personal account data.
- Do not weaken archive validation, filesystem boundaries, or error reporting to make a test pass.
- Update documentation when workflows, supported formats, or privacy behavior change.
- Keep [GitHub repository metadata](docs/GITHUB_METADATA.md) aligned with the live About panel when product positioning changes.
- Regenerate the synthetic product tour with `npm run docs:screenshots` for visible interface changes, then review every changed image and run `npm run docs:check`.

## Commit Messages

Use short imperative subjects, such as `Add snapshot selector` or `Reject oversized archives`. Separate unrelated changes into separate commits.

## Reporting Bugs

Use the bug report form and remove all personal Instagram data from logs, screenshots, fixtures, and example exports.
