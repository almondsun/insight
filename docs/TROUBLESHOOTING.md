# Troubleshooting

Start by confirming your version and that you have a complete Instagram export in JSON format. Nivune Preview 4 is the current prerelease; stable v0.1.1 and Preview 3 retain the former identity and differ as described in [version differences](GETTING_STARTED.md#version-differences).

## The file picker does not show my export

In Preview 4 and current `main`, **Import ZIP** accepts ZIP files. Published v0.1.1 also displays JSON files. In either version, choose the complete export ZIP rather than an individual relationship file. For an extracted export, choose **Folder** instead.

HTML exports are not supported. Request the export again and select JSON.

## No follower or following files were found

In Preview 4 and current `main`, the selected ZIP or folder must preserve Instagram's export structure and include the follower and following JSON files. Published v0.1.1 may accept one relationship direction, but a complete export is still recommended because derived categories otherwise have incomplete meaning. Do not select a parent downloads directory containing many unrelated folders, a single relationship file, or an HTML export.

## The import says both relationship files are required

Preview 4 and current `main` deliberately reject partial exports. Request or select an export containing both followers and following so every derived category has a defined meaning. Published v0.1.1 does not perform this completeness check.

## The archive is malformed, unsafe, too large, or contains too many entries

Preview 4 and current `main` treat exports as untrusted input and fail closed on malformed relationship JSON, unsafe ZIP metadata or paths, excessive traversal, or configured work and size limits. For ZIP and folder input, published v0.1.1 checks malformed relationship JSON, enclosed ZIP paths, relevant-file limits, per-file limits, and aggregate ZIP relationship bytes, but lacks the current folder and compressed-work protections. A standalone JSON file selected in v0.1.1 is read without that per-file size cap. Download the export again directly from Instagram. Do not use third-party tools to rewrite the archive unless you can verify exactly what they changed.

See [Privacy and Data Handling](PRIVACY.md#filesystem-and-parser-protections) for the current limits.

## The owner username is missing or rejected

In Preview 4 and current `main`, some exports do not include owner metadata in a form Nivune recognizes. Enter the Instagram username that owns the archive. Use only the username, without `@` or a profile URL. Published v0.1.1 does not request explicit owner confirmation.

When owner metadata is present, Preview 4 requires the confirmation to match it after trimming and case normalization. If the detected owner is wrong, cancel the import and verify that you selected the intended account's complete export.

## The snapshot is a duplicate

Snapshot identity is based on normalized follower and following membership, not the ZIP filename or import time. Import a newer export only after Instagram relationships have changed if you want a distinct snapshot.

## Changes are empty or unavailable

Changes require at least two snapshots in the same local account history. Confirm that both exports were added to the same selected account and that the intended relationship category and search field are active.

An empty result can also be correct: no usernames in that category changed between the two selected snapshots.

## CSV or JSON export was not created

Cancelling the native save dialog intentionally creates no file. Try again and choose a writable destination. Relationships exports the active category; Changes exports the selected comparison, category, and smart-list direction.

## Windows or macOS warns about an unknown developer

Current release artifacts are unsigned and not notarized. Verify that the file came from the official [Nivune releases page](https://github.com/almondsun/nivune/releases), review its release notes, and compare its SHA-256 digest with the release's `SHA256SUMS` entry before deciding whether to proceed.

Do not disable SmartScreen, Gatekeeper, antivirus, or other operating-system protections globally.

## The macOS download does not run on my Intel Mac

Published Nivune and former-name previews provide a macOS Apple Silicon (`aarch64`) artifact only. Intel macOS is not currently provided as a release download. Developers may attempt a source build with the required Tauri toolchain, but it is not part of the published support matrix.

## I found a bug

Read [SUPPORT.md](../SUPPORT.md), then use the structured bug form. Include the Nivune version, operating system, expected behavior, actual behavior, and reproducible steps using sanitized data.

Never attach a real Instagram archive, SQLite database, username list, local path, or login secret.
