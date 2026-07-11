# insIGht

insIGht is a local-first desktop application for understanding Instagram account relationships. It imports the official JSON account export and presents followers, following, mutuals, non-reciprocal relationships, and changes between snapshots without sending account data to a server.

## Features

- Import official Instagram JSON ZIP archives, individual JSON files, or extracted folders
- Track multiple Instagram accounts and import snapshots over time
- Browse followers, following, mutuals, and non-reciprocal relationships
- Search relationship lists and compare consecutive snapshots
- Detect duplicate imports
- Export relationship reports as CSV or JSON
- Delete individual snapshots while keeping the remaining history
- Store all normalized data locally in SQLite

## Privacy

insIGht does not connect to Instagram, request login secrets, or include telemetry. Imports are parsed locally, and original archives are not retained. The SQLite database is stored in the operating system's application-data directory and relies on normal OS account and disk protections.

## Supported Input

Request **Download Your Information** from Instagram Accounts Center and select **JSON**. HTML exports are not supported. Instagram does not reliably include stable numeric IDs, so a username change may appear as one removed account and one added account.

## Development

### Prerequisites

- Node.js 22 or newer
- Rust stable
- [Tauri 2 platform prerequisites](https://v2.tauri.app/start/prerequisites/)

Install dependencies and launch the desktop application:

```bash
npm ci
npm run tauri dev
```

The standalone Vite server is useful for layout work, but native imports, dialogs, persistence, and exports only work inside Tauri.

### Validation

```bash
npm run build
npm audit --omit=dev
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Architecture

- **React + TypeScript** renders the desktop interface and manages query state.
- **Tauri + Rust** owns filesystem access, archive validation, imports, reports, and native dialogs.
- **SQLite** stores normalized accounts, immutable snapshots, and relationships.
- Imported archives are treated as untrusted input and constrained by path, file-count, and uncompressed-size checks.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Security reports should follow [SECURITY.md](SECURITY.md) rather than a public issue.

## License

No license has been selected yet. All rights are reserved until a license file is added.
