# Changelog

All notable user-visible changes to insIGht are documented here. The project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0-preview.3] - 2026-08-08

### Changed

- Consolidated all Rust packages under one pinned workspace toolchain and lockfile
- Added cursor-based relationship and change loading with debounced native search
- Changed summaries and exports to bounded-memory database queries; exports now stream from a transaction-consistent dedicated SQLite connection and atomically replace completed reports
- Separated the Fame persistence adapter from the archive database module

### Security

- Added raw ZIP/ZIP64 metadata, compressed-work, and central-directory preflight limits plus canonical duplicate-path rejection before archive extraction
- Added private Unix permissions for application data, SQLite files, and exported reports
- Replaced native command lock panics with explicit errors and enabled SQLite foreign keys, secure deletion, WAL, and a busy timeout
- Made release publication depend on the complete CI-equivalent validation suite

### Tests

- Added regressions for stable paging, literal wildcard search, adjacent comparisons, ZIP metadata limits, duplicate archive names, multi-page streaming export, and scheduler integer boundaries

### Documentation

- Added task-based installation, user, privacy, troubleshooting, architecture, support, and roadmap documentation
- Added a versioned source of truth for GitHub About metadata and repository topics
- Clarified current platform artifacts, unsigned-install limitations, supported ZIP/folder imports, and the unavailable Fame roadmap boundary

## [0.1.1] - 2026-07-11

### Fixed

- Registered platform-specific PNG, ICNS, and ICO assets for cross-platform installer packaging

## [0.1.0] - 2026-07-11

### Added

- Local import of official Instagram JSON ZIP archives, individual files, and extracted folders
- Followers, following, mutuals, and non-reciprocal relationship dashboards
- Searchable account lists and snapshot-to-snapshot change detection
- Local multi-account history backed by SQLite
- Duplicate snapshot detection and snapshot deletion
- CSV and JSON relationship exports
- Defensive archive path, file-count, and decompressed-size validation
- Windows, macOS, and Linux release builds through GitHub Actions

### Security

- Imports only the relationship files and owner metadata required for analytics
- Keeps source archives outside application storage
- Performs no Instagram login, scraping, telemetry, or automatic network synchronization

[Unreleased]: https://github.com/almondsun/insight/compare/v0.2.0-preview.3...HEAD
[0.2.0-preview.3]: https://github.com/almondsun/insight/compare/v0.2.0-preview.2...v0.2.0-preview.3
[0.1.1]: https://github.com/almondsun/insight/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/almondsun/insight/releases/tag/v0.1.0
