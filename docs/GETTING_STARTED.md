# Getting Started

This guide covers installing insIGht, requesting the right Instagram export, and creating your first local snapshot. Workflow instructions target the current `main` branch; the latest downloadable release has the differences documented below.

## 1. Download the right build

Open the [v0.2.0-preview.1 prerelease](https://github.com/almondsun/insight/releases/tag/v0.2.0-preview.1) and choose an artifact for your system.

| System | Choose | Notes |
| --- | --- | --- |
| Windows x86-64 | `.msi` or `-setup.exe` | Both install the desktop application. |
| macOS Apple Silicon | `aarch64.dmg` | Intel Mac builds are not currently published. |
| Debian or Ubuntu x86-64 | `amd64.deb` | Uses the system package manager. |
| Fedora, RHEL, or compatible x86-64 | `x86_64.rpm` | Uses the system package manager. |
| Other supported x86-64 Linux desktops | `amd64.AppImage` | Portable application image. |

Preview artifacts are not code-signed or notarized. Review the release notes and verify the artifact against the release's `SHA256SUMS` file. Do not disable operating-system security features globally to install insIGht.

Windows or macOS may require you to confirm that you trust an unknown developer. Linux AppImage users may need to mark the downloaded file as executable through file properties or `chmod +x`.

## Version differences

The stable release is v0.1.1. The v0.2.0-preview.1 prerelease contains the hardened workflow on `main`.

| Behavior | v0.2 preview / current `main` | Stable v0.1.1 |
| --- | --- | --- |
| Import and export path selection | Native code owns the dialog result; the WebView cannot submit arbitrary paths. | The WebView supplies paths selected through the Tauri dialog plugin. |
| Accepted import selection | Complete ZIP or extracted folder containing both relationship directions. | ZIP, folder, or individual JSON; partial relationship data is accepted. |
| Archive owner | Must be detected or entered and confirmed before commit. | No explicit owner-confirmation step. |
| Folder protections | Entry, depth, relevant-file, per-file, and aggregate relevant-byte limits. | Relevant-file and per-file limits; no folder entry, depth, or aggregate-byte limit. |

Both versions perform local analytics without Instagram credentials, scraping, archive upload, or telemetry. Use the v0.2 preview for the strongest implemented filesystem and parser boundaries.

## 2. Request a JSON export from Instagram

Instagram changes the wording and layout of Accounts Center periodically. In Instagram or Accounts Center:

1. Open **Your information and permissions**.
2. Choose the option to download or export your information.
3. Select the Instagram account you want to analyze.
4. Include follower and following information, or request a complete export.
5. Select **JSON**, not HTML.
6. Create the export, then download the ZIP when Instagram says it is ready.

Keep the ZIP private. It can contain considerably more personal information than insIGht reads.

## 3. Import the archive

Launch insIGht and choose one of these paths:

- **Import file** for the complete Instagram ZIP.
- **Folder** for the extracted root folder of that ZIP.

Current `main` requires both the follower and following JSON files. HTML exports, individual JSON files, and partial exports are not supported. The v0.1.1 picker also displays individual JSON files, but a complete ZIP or extracted folder is still the recommended input.

Before anything is written to the database, insIGht shows an import preview with the detected follower and following totals. Current `main` also asks you to confirm the archive owner's Instagram username. When owner metadata exists in the export, that value is shown read-only; otherwise, enter it yourself. Published v0.1.1 has a simpler preview without owner confirmation.

For a new account history, choose **Create new account** and provide a label. For a later export of the same account, add the snapshot to the already selected account.

## 4. Explore the dashboard

After confirmation, the newest snapshot becomes active. You can browse relationship categories, search by username, export a category, or import another snapshot to see changes.

Continue with the [user guide](USER_GUIDE.md), or see [troubleshooting](TROUBLESHOOTING.md) if the import is rejected.

## Build from source

Developers need Node.js 22 or newer, stable Rust, and the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for their platform.

```bash
git clone https://github.com/almondsun/insight.git
cd insight
npm ci
npm run tauri dev
```

Building from source does not add code signing or notarization.
