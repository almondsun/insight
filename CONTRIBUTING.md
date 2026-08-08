# Contributing

Start with the [architecture overview](docs/ARCHITECTURE.md), then read the user-facing documentation for any workflow you plan to change. Public behavior, supported inputs, privacy boundaries, and roadmap status must remain consistent across code and documentation.

## Development Workflow

1. Create a focused branch from `main`.
2. Install dependencies with `npm ci`.
3. Keep filesystem, archive, SQLite, and export behavior in Rust; keep presentation and transient UI state in React.
4. Add or update tests for behavior changes.
5. Run `npm run check`.
6. Open a pull request using the repository template.

## Pull Requests

- Keep changes scoped and explain user-visible or compatibility effects.
- Do not commit Instagram exports, SQLite databases, login secrets, or other personal account data.
- Do not weaken archive validation, filesystem boundaries, or error reporting to make a test pass.
- Update documentation when workflows, supported formats, or privacy behavior change.
- Keep [GitHub repository metadata](docs/GITHUB_METADATA.md) aligned with the live About panel when product positioning changes.
- Include screenshots for visible interface changes when practical.

## Commit Messages

Use short imperative subjects, such as `Add snapshot selector` or `Reject oversized archives`. Separate unrelated changes into separate commits.

## Reporting Bugs

Use the bug report form and remove all personal Instagram data from logs, screenshots, fixtures, and example exports.
