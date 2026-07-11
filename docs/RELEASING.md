# Releasing insIGht

GitHub Releases are the distribution channel for versioned desktop installers.

## Before The First Release

- Verify import, persistence, search, comparison, deletion, and export workflows with sanitized fixtures.
- Build and launch the application on Windows, macOS, and Linux.
- Decide whether unsigned preview builds are acceptable or configure platform signing.
- Confirm the version matches in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Prepare concise release notes with user-visible changes, known limitations, and upgrade notes.

## Creating A Draft Release

1. Merge the release changes into `main` after required checks pass.
2. Create and push a semantic version tag:

   ```bash
   git tag -a v0.1.0 -m "insIGht 0.1.0"
   git push origin v0.1.0
   ```

3. The `Release` GitHub Actions workflow builds artifacts on Windows, macOS, and Linux.
4. Inspect every artifact and test installation on its target platform.
5. Replace the generated draft body with release notes and publish the draft.

## Signing

The current workflow does not configure Windows signing, Apple signing/notarization, or Linux package signing. Signing material must be stored as encrypted GitHub Actions secrets and must never be committed. Public stable releases should not be described as signed until each platform's verification has been completed.

## Versioning

Use semantic versioning:

- Patch: compatible bug fixes
- Minor: compatible features
- Major: incompatible persisted-data, import, report, CLI, or public interface changes

Pre-release tags such as `v0.2.0-beta.1` should be marked as pre-releases manually before publication.
