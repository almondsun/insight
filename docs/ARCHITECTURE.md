# Application Architecture

This document describes Preview 4 and current `main`. Nivune is a Tauri 2 desktop application: React renders the interface, Rust owns native and trust-sensitive operations, and SQLite stores normalized local history. Older former-name releases differ; see [version differences](GETTING_STARTED.md#version-differences).

```text
Official Instagram JSON ZIP or folder
                  |
                  v
       Native file dialog and source adapter
       - Instagram archive adapter (first implementation)
       - path and traversal validation
       - bounded decompression and reads
       - username and URL normalization
                  |
                  v
       Immutable snapshots in SQLite
       - accounts and snapshot history
       - normalized relationship membership
       - Fame foundation schema, offline only
                  |
                  v
       Typed Tauri commands and responses
                  |
                  v
       React desktop interface
       - import preview and confirmation
       - lists, search, history, changes
       - native CSV and JSON export
```

## Native boundary

Rust owns import selection, export and encrypted-backup destinations, archive parsing, validation, SQLite access, summary calculations, comparisons, report serialization, backup encryption/restore, and deletion. Import parsing and native file dialogs run outside the WebView boundary. A narrow `RelationshipSourceAdapter` keeps the platform-specific archive parser replaceable without changing persistence or presentation.

The parser follows neither nested directory symlinks nor a symlink selected as the traversal root. It accepts only enclosed ZIP entry paths, preflights raw central-directory metadata before constructing the archive reader, rejects canonical duplicate entry names, reads recognized files, and applies entry-count, file-count, relationship-count, individual-size, aggregate-size, and traversal-depth limits. Usernames and profile URLs are normalized before persistence.

## Persistence

At startup, Tauri resolves the `app.nivune.local` application-data directory and opens `nivune.db`. If it does not exist, an existing `app.insight.local/insight.db` is copied once through SQLite's online backup API; the legacy file is retained. Database migrations use SQLite's `user_version`, and an application identifier rejects foreign databases. Accounts contain immutable snapshots; relationships belong to a snapshot and are not stored as redundant derived categories. Observation dates are stored separately from import timestamps; migrated preview rows are marked as inferred from their legacy import time.

Mutuals and non-reciprocal categories are derived from follower and following membership. Summary metrics are database aggregates; relationship and change lists use stable cursor pagination instead of loading a snapshot into the WebView at once. Snapshot hashes are based on normalized membership so equivalent imports are detected as duplicates.

The separate Fame persistence adapter installs run, membership, and observation schema scaffolding with an authentication-state field. No production command populates it, and no cryptographic observation verifier or retrieval orchestration is implemented. Account and snapshot deletion still applies the defined cascades and unreferenced-observation cleanup.

## Tauri interface

The native command surface exposes account and snapshot queries, account rename/delete, import preview/commit/cancel, trends, relationship history, paged relationship lists, paged arbitrary-snapshot comparisons, snapshot deletion, native relationship/change report export, encrypted backup/restore, and a read-only Fame foundation status. CSV and JSON exports stream a transaction-consistent database snapshot through a dedicated connection on a blocking worker, sync a private sibling temporary file, and atomically replace the selected destination only after success.

Portable backups use SQLite's online backup API for a consistent snapshot and age passphrase encryption for authenticated confidentiality. Restore bounds input and decrypted sizes, verifies age authentication, SQLite integrity, schema compatibility, required tables, and the application identifier before replacing the live database through SQLite's backup API.

TypeScript models mirror serialized Rust responses. Network access is not exposed to the WebView. The current content security policy permits local application resources and Tauri IPC.

## Frontend

React and TanStack Query manage server-state caching, mutations, debounced search, and incremental page loading. A dedicated results component owns list/change row rendering; native code remains authoritative for filesystem, archive, persistence, filtering, and validation behavior.

The interface supports any two distinct snapshots in one account. Direction-filtered smart lists and exports share the same native query contract.

The root application accepts a typed `NivuneApi` client with the native Tauri client as its production default. The isolated documentation preview supplies a synthetic in-memory client to the same interface; it is not built into the desktop bundle and has no filesystem or network adapter. Playwright drives that preview to keep documentation screenshots aligned with the real React interface.

## Validation

The repository's CI-equivalent command is:

```bash
npm run check
```

It runs the TypeScript/Vite production build, frontend tests, Rust formatting check, strict Clippy analysis, and locked Rust tests. CI separately audits production npm dependencies and runs dependency review and CodeQL workflows.

Documentation changes use `npm run docs:check` to validate local Markdown targets and anchors and compare the synthetic Playwright tour with the checked-in screenshots. Intentional UI updates regenerate those images with `npm run docs:screenshots`.

Two ignored Rust integration tests can be enabled locally by setting `NIVUNE_REAL_EXPORT` to a private export. The former `INSIGHT_REAL_EXPORT` name remains accepted for developer compatibility. Their input must never be committed or attached to a report.

## Security-sensitive contracts

- Archives are untrusted input.
- Native code mediates every import and export path.
- The SQLite mutex is not held across native dialog waits.
- Errors cross the Tauri boundary as explicit results rather than being silently ignored.
- Real account data must not enter fixtures, logs, screenshots, issues, or pull requests.
- Fame production networking remains fail closed until its external feasibility and audit gates pass.
