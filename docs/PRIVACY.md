# Privacy and Data Handling

This document describes the behavior implemented on the current `main` branch and explicitly identifies where published v0.1.1 differs. It separates shipped guarantees from proposed Fame research so users can make informed decisions.

## Version scope

Both current `main` and published v0.1.1 perform relationship analytics locally without Instagram credentials, scraping, archive upload, or telemetry.

Current `main` adds native-only import and export path mediation, required archive-owner confirmation, rejection of partial or standalone relationship inputs, and folder entry, depth, and aggregate relevant-byte limits. These protections are not present in the v0.1.1 binaries. See the [complete version comparison](GETTING_STARTED.md#version-differences).

## Current main application

The released relationship-analytics workflow is local-first:

- You select an official Instagram JSON ZIP or extracted folder through a native file dialog.
- Rust reads only recognized relationship files and optional owner metadata.
- Parsed snapshots are stored in a local SQLite database.
- Searches, comparisons, summaries, and report generation run on the device.
- insIGht does not ask for Instagram credentials, connect to an Instagram account, scrape profiles, upload archives, or send usage telemetry.
- The source ZIP or folder is read in place and is not copied into application storage.

The current `main` WebView content security policy permits application resources and Tauri IPC. Network retrieval is not exposed to the interface. In v0.1.1, file paths selected through the Tauri dialog plugin pass through the WebView command boundary.

## Data that is stored

The local database can contain:

- An account label and confirmed owner username.
- Snapshot import times, source names, and membership hashes.
- Normalized related-account usernames.
- Sanitized canonical Instagram profile URLs when supplied by the export.
- Relationship timestamps when supplied by the export.
- Relationship kind and snapshot membership.

The database schema reserves tables for the Fame persistence foundation, but the released interface has no retrieval or run command that populates network observations.

The database is stored as `insight.db` inside the operating system application-data directory resolved for the `app.insight.local` application identifier.

## Data that is not stored

insIGht does not retain the raw archive body, passwords, login sessions, cookies, biographies, posts, messages, photographs, or unrecognized export files. Import previews are held only in memory until confirmed, cancelled, replaced, or the application exits.

## Filesystem and parser protections

The native application, not WebView JavaScript, chooses import and export paths. Imported archives are treated as untrusted input.

Current `main` limits include:

- At most 10,000 archive or folder entries inspected.
- At most 2,000 relevant files.
- At most 16 MiB per relevant JSON file.
- At most 128 MiB of relevant JSON data in total.
- Nested directory symlinks are not followed. A symlink explicitly selected as the import root may be resolved by the directory walker.
- Enclosed ZIP paths only, preventing archive path traversal.
- A maximum folder traversal depth of 20.

On current `main`, malformed relationship JSON, invalid usernames, partial relationship exports, unsafe paths, and size-limit violations fail the import. Published v0.1.1 does not enforce the complete-export requirement or the current folder entry, depth, and aggregate-byte limits.

## What still depends on you

The SQLite database is not encrypted by insIGht. Its confidentiality depends on operating-system account permissions, full-disk encryption, backups, and physical device security.

CSV and JSON reports are ordinary files. Other applications, cloud-sync services, backups, or people with access to their destination may read them. Choose the destination deliberately and delete reports you no longer need.

Instagram exports may contain much more information than insIGht reads. Store or delete the original archive according to your own security needs.

## Fame roadmap boundary

Fame is not a current network feature. The repository contains its versioned score, persistence foundation, synthetic corpus tooling, scheduler scaffolding, and frozen privacy specifications, but no production retrieval or user-facing run workflow.

The proposed architecture would introduce explicit optional networking only after feasibility measurements and independent audits. It forbids direct Instagram access, credential use, fallback transport, miss-driven collection, and claims of absolute anonymity. Until those gates pass, the application remains offline and reports Fame network retrieval as unavailable.

See the [roadmap](ROADMAP.md), [implementation status](fame/IMPLEMENTATION_STATUS.md), and [formal threat model](fame/THREAT_MODEL.md).

## Reporting a privacy or security issue

Follow [SECURITY.md](../SECURITY.md). Do not open a public issue containing a real export, database, username list, local path, or credential.
