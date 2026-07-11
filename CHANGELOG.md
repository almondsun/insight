# Changelog

All notable user-visible changes to insIGht are documented here. The project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/almondsun/insight/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/almondsun/insight/releases/tag/v0.1.0
