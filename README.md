<div align="center">
  <img src="src-tauri/icons/icon.png" alt="insIGht eye logo" width="128" height="128">
  <h1>insIGht</h1>
  <p><strong>See who follows you, who does not, and what changed—without connecting your Instagram account.</strong></p>
  <p>insIGht is a local-first desktop app for exploring official Instagram JSON exports.</p>

  [![Latest release](https://img.shields.io/github/v/release/almondsun/insight?display_name=tag&sort=semver)](https://github.com/almondsun/insight/releases/latest)
  [![CI](https://github.com/almondsun/insight/actions/workflows/ci.yml/badge.svg)](https://github.com/almondsun/insight/actions/workflows/ci.yml)
  [![CodeQL](https://github.com/almondsun/insight/actions/workflows/codeql.yml/badge.svg)](https://github.com/almondsun/insight/actions/workflows/codeql.yml)
  [![License: MIT](https://img.shields.io/badge/license-MIT-e1306c.svg)](LICENSE)

  [Download insIGht](https://github.com/almondsun/insight/releases/latest) · [Read the docs](docs/README.md) · [Get support](SUPPORT.md)
</div>

![The insIGht desktop import screen](docs/assets/insight-preview.png)

> [!IMPORTANT]
> These docs follow the hardened code on `main`. The latest v0.1.1 binaries predate native-only path mediation, required owner confirmation, complete-export enforcement, and the newest folder limits. See the [version differences](docs/GETTING_STARTED.md#version-differences) before installing v0.1.1.

## Your Instagram relationships, made understandable

Instagram gives you a copy of your information, but not an easy way to explore it. insIGht turns the follower and following files in that official export into a searchable desktop dashboard.

| Explore | Compare | Keep control |
| --- | --- | --- |
| Browse followers, following, mutuals, and non-reciprocal relationships. | Import snapshots over time to see additions and removals. | Keep archives and analytics on your device, with no Instagram login or telemetry. |

### How it works

1. Request your information from Instagram Accounts Center in **JSON** format.
2. Import the complete ZIP or extracted export folder into insIGht.
3. Search relationships, compare snapshots, and export selected lists as CSV or JSON.

[Start with the installation and import guide →](docs/GETTING_STARTED.md)

## Features

- Followers, following, mutuals, accounts that do not follow you back, and followers you do not follow.
- Search across each relationship category.
- Separate histories for multiple Instagram accounts.
- Snapshot comparisons that show additions and removals between imports.
- CSV and JSON reports saved through the operating system's native file dialog.
- Defensive ZIP and folder parsing with path, file-count, and decompressed-size limits.
- Local SQLite persistence in the operating system's application-data directory.

## Privacy, without vague promises

The current `main` application does not log in to Instagram, scrape profiles, upload archives, use telemetry, or require a network connection for analytics. Rust owns file access and archive parsing; the WebView cannot choose arbitrary filesystem paths. The downloadable v0.1.1 release preserves the no-login, no-scraping, no-upload, and no-telemetry boundary, but predates the current native path mediation and stricter import validation.

Your local database and exported reports can still contain personal information. insIGht does not currently encrypt its database, so protection depends on your operating-system account and disk security.

[Read the complete privacy and data-boundary documentation →](docs/PRIVACY.md)

## Downloads

Version 0.1.1 provides these unsigned preview builds:

| Platform | Current artifact choices | Architecture |
| --- | --- | --- |
| Windows | MSI or setup EXE | x86-64 |
| macOS | DMG | Apple Silicon |
| Linux | AppImage, DEB, or RPM | x86-64 |

The installers are not code-signed or notarized. Windows and macOS may display an unknown-developer warning. Review the release notes and verify the SHA-256 digest shown by GitHub before installing.

[Download the latest release →](https://github.com/almondsun/insight/releases/latest)

## Fame roadmap

Fame is a proposed optional ranking feature based on public follower and following counts. Its scoring and synthetic test foundations exist, but **Fame retrieval and its user workflow are not implemented or available**. The research design requires a licensed query-independent corpus, independently operated PIR replicas, mixed request and reply paths, fixed cover traffic, witnessed releases, reproducible measurements, and independent audits. It forbids direct Instagram access and fallback transport.

[Read the roadmap](docs/ROADMAP.md) · [Review the Fame engineering status](docs/fame/README.md)

## Documentation

- [Getting started](docs/GETTING_STARTED.md)
- [User guide](docs/USER_GUIDE.md)
- [Privacy and data handling](docs/PRIVACY.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Development and contributing](CONTRIBUTING.md)

## Build from source

You need Node.js 22 or newer, stable Rust, and the [Tauri 2 platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
git clone https://github.com/almondsun/insight.git
cd insight
npm ci
npm run tauri dev
```

Run the complete local CI-equivalent validation with:

```bash
npm run check
```

The standalone Vite server is useful for interface work, but native dialogs, archive imports, SQLite persistence, and exports require the Tauri runtime.

## Support and community

Use [GitHub Discussions](https://github.com/almondsun/insight/discussions) for questions, the structured [issue forms](https://github.com/almondsun/insight/issues/new/choose) for reproducible bugs or feature proposals, and the private process in [SECURITY.md](SECURITY.md) for vulnerabilities. Never publish a real Instagram export, username list, SQLite database, or login secret.

## Independent project

insIGht is not affiliated with, authorized, maintained, or endorsed by Instagram or Meta. Instagram is a trademark of its respective owner. This project reads export files that a user obtains through Instagram's official data-download feature.

## License

insIGht is available under the [MIT License](LICENSE).
