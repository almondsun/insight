# Application Architecture

This document describes the current `main` branch. insIGht is a Tauri 2 desktop application: React renders the interface, Rust owns native and trust-sensitive operations, and SQLite stores normalized local history. Published v0.1.1 predates the current native path boundary and stricter importer; see [version differences](GETTING_STARTED.md#version-differences).

```text
Official Instagram JSON ZIP or folder
                  |
                  v
       Native file dialog and Rust parser
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

Rust owns import selection, export destinations, archive parsing, validation, SQLite access, summary calculations, comparisons, report serialization, and deletion. Import parsing and native file dialogs run outside the WebView boundary.

The parser follows neither nested directory symlinks nor a symlink selected as the traversal root. It accepts only enclosed ZIP entry paths, preflights raw central-directory metadata before constructing the archive reader, rejects canonical duplicate entry names, reads recognized files, and applies entry-count, file-count, relationship-count, individual-size, aggregate-size, and traversal-depth limits. Usernames and profile URLs are normalized before persistence.

## Persistence

At startup, Tauri resolves the operating system's application-data directory and opens `insight.db`. Database migrations use SQLite's `user_version`. Accounts contain immutable snapshots; relationships belong to a snapshot and are not stored as redundant derived categories.

Mutuals and non-reciprocal categories are derived from follower and following membership. Summary metrics are database aggregates; relationship and change lists use stable cursor pagination instead of loading a snapshot into the WebView at once. Snapshot hashes are based on normalized membership so equivalent imports are detected as duplicates.

The separate Fame persistence adapter installs run, membership, and observation schema scaffolding with an authentication-state field. No production command populates it, and no cryptographic observation verifier or retrieval orchestration is implemented. Account and snapshot deletion still applies the defined cascades and unreferenced-observation cleanup.

## Tauri interface

The native command surface exposes account and snapshot queries, account rename/delete, import preview/commit/cancel, summaries, paged relationship lists, paged adjacent-snapshot comparisons, snapshot deletion, native relationship/change report export, and a read-only Fame foundation status. CSV and JSON exports stream a transaction-consistent database snapshot through a dedicated connection on a blocking worker, sync a private sibling temporary file, and atomically replace the selected destination only after success.

TypeScript models mirror serialized Rust responses. Network access is not exposed to the WebView. The current content security policy permits local application resources and Tauri IPC.

## Frontend

React and TanStack Query manage server-state caching, mutations, debounced search, and incremental page loading. A dedicated results component owns list/change row rendering; native code remains authoritative for filesystem, archive, persistence, filtering, and validation behavior.

The interface compares a selected snapshot with its immediately prior import. Export behavior follows the current list or change view.

## Validation

The repository's CI-equivalent command is:

```bash
npm run check
```

It runs the TypeScript/Vite production build, frontend tests, Rust formatting check, strict Clippy analysis, and locked Rust tests. CI separately audits production npm dependencies and runs dependency review and CodeQL workflows.

Two ignored Rust integration tests can be enabled locally by setting `INSIGHT_REAL_EXPORT` to a private export. Their input must never be committed or attached to a report.

## Security-sensitive contracts

- Archives are untrusted input.
- Native code mediates every import and export path.
- The SQLite mutex is not held across native dialog waits.
- Errors cross the Tauri boundary as explicit results rather than being silently ignored.
- Real account data must not enter fixtures, logs, screenshots, issues, or pull requests.
- Fame production networking remains fail closed until its external feasibility and audit gates pass.
