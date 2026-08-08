# Troubleshooting

Start by confirming your version and that you have a complete Instagram export in JSON format. The stable release is v0.1.1; the v0.2.0-preview.1 prerelease contains the stricter import hardening described in [version differences](GETTING_STARTED.md#version-differences).

## The file picker does not show my export

On current `main`, **Import file** accepts ZIP files. Published v0.1.1 also displays JSON files. In either version, choose the complete export ZIP rather than an individual relationship file. For an extracted export, choose **Folder** instead.

HTML exports are not supported. Request the export again and select JSON.

## No follower or following files were found

On current `main`, the selected ZIP or folder must preserve Instagram's export structure and include the follower and following JSON files. Published v0.1.1 may accept one relationship direction, but a complete export is still recommended because derived categories otherwise have incomplete meaning. Do not select a parent downloads directory containing many unrelated folders, a single relationship file, or an HTML export.

## The import says both relationship files are required

This message comes from current `main`, which deliberately rejects partial exports. Request or select an export containing both followers and following so every derived category has a defined meaning. Published v0.1.1 does not perform this completeness check.

## The archive is malformed, unsafe, too large, or contains too many entries

Current `main` treats exports as untrusted input and fails closed on malformed relationship JSON, unsafe ZIP paths, excessive traversal, or any configured size limit. For ZIP and folder input, published v0.1.1 checks malformed relationship JSON, enclosed ZIP paths, relevant-file limits, per-file limits, and aggregate ZIP relationship bytes, but lacks current `main`'s folder entry, depth, and aggregate-byte checks. A standalone JSON file selected in v0.1.1 is read without that per-file size cap. Download the export again directly from Instagram. Do not use third-party tools to rewrite the archive unless you can verify exactly what they changed.

See [Privacy and Data Handling](PRIVACY.md#filesystem-and-parser-protections) for the current limits.

## The owner username is missing or rejected

On current `main`, some exports do not include owner metadata in a form insIGht recognizes. Enter the Instagram username that owns the archive. Use only the username, without `@` or a profile URL. Published v0.1.1 does not request explicit owner confirmation.

When owner metadata is present, current `main` requires the confirmation to match it after trimming and case normalization. If the detected owner is wrong, cancel the import and verify that you selected the intended account's complete export.

## The snapshot is a duplicate

Snapshot identity is based on normalized follower and following membership, not the ZIP filename or import time. Import a newer export only after Instagram relationships have changed if you want a distinct snapshot.

## Changes are empty or unavailable

Changes require at least two snapshots in the same local account history. Confirm that both exports were added to the same selected account and that the intended relationship category and search field are active.

An empty result can also be correct: no usernames in that category changed between the selected snapshot and its immediately prior import.

## CSV or JSON export was not created

Cancelling the native save dialog intentionally creates no file. Try again and choose a writable destination. Accounts view exports the active relationship category; Changes view exports additions and removals for that category and requires an immediately prior snapshot.

## Windows or macOS warns about an unknown developer

Current release artifacts are unsigned and not notarized. Verify that the file came from the official [almondsun/insight release](https://github.com/almondsun/insight/releases/latest), review its release notes, and compare its SHA-256 digest with GitHub's displayed digest before deciding whether to proceed.

Do not disable SmartScreen, Gatekeeper, antivirus, or other operating-system protections globally.

## The macOS download does not run on my Intel Mac

Version 0.1.1 publishes a macOS Apple Silicon (`aarch64`) artifact only. Intel macOS is not currently provided as a release download. Developers may attempt a source build with the required Tauri toolchain, but it is not part of the published support matrix.

## I found a bug

Read [SUPPORT.md](../SUPPORT.md), then use the structured bug form. Include the insIGht version, operating system, expected behavior, actual behavior, and reproducible steps using sanitized data.

Never attach a real Instagram archive, SQLite database, username list, local path, or login secret.
