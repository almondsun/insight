# Nivune Documentation

Nivune is a local-first desktop application for exploring follower and following relationships from an official Instagram JSON export. Start with the guide that matches what you want to do.

## Use Nivune

- [Install Nivune and complete your first import](GETTING_STARTED.md)
- [Explore relationships, snapshots, changes, and reports](USER_GUIDE.md)
- [Understand what stays local and what is stored](PRIVACY.md)
- [Solve import, comparison, export, or installation problems](TROUBLESHOOTING.md)
- [Get support without exposing personal information](../SUPPORT.md)

## Understand the project

- [Application architecture](ARCHITECTURE.md)
- [Product roadmap](ROADMAP.md)
- [Fame privacy-engineering research](fame/README.md)
- [Release process](RELEASING.md)
- [GitHub repository metadata](GITHUB_METADATA.md)
- [Contributing](../CONTRIBUTING.md)
- [Security policy](../SECURITY.md)

## Current product boundary

Current `main` imports complete ZIP archives or extracted folders containing both follower and following JSON files. Its analytics, history, database, and reports are local. It does not log in to Instagram, scrape profiles, upload relationship data, or send telemetry. Published v0.1.1 has a weaker import boundary; review the [version differences](GETTING_STARTED.md#version-differences).

Fame is roadmap research, not a released feature. Its network retrieval and user workflow remain unavailable.
