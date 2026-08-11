# Releasing Nivune

GitHub Releases are the distribution channel for versioned desktop installers.

## Release checklist

- Verify import, persistence, search, comparison, deletion, and export workflows with sanitized fixtures.
- Build and launch the application on Windows, macOS, and Linux.
- Decide whether unsigned preview builds are acceptable or configure platform signing.
- Confirm the public version matches in `package.json` and the root Cargo workspace `Cargo.toml`. For `vX.Y.Z-preview.N`, set the Tauri bundle version to the Windows-compatible `X.Y.Z-N`; the release workflow verifies this mapping.
- Prepare concise release notes with user-visible changes, known limitations, and upgrade notes.
- Verify the platform and architecture table in [Getting Started](GETTING_STARTED.md) against the actual release assets.
- Verify that [GitHub repository metadata](GITHUB_METADATA.md) still matches the live About panel.
- Run `npm run docs:check` and visually inspect every changed screenshot.
- Install the pinned screenshot browser with `npx playwright install chromium` when setting up a new development machine.
- Review open dependency alerts. Document accepted prerelease risk; never dismiss or vendor a fix solely to make the dashboard green.

## Creating A Release

1. Merge the release changes into `main` after required checks pass.
2. Create and push a semantic version tag:

   ```bash
   git tag -a v0.2.0-preview.4 -m "Nivune 0.2.0-preview.4"
   git push origin v0.2.0-preview.4
   ```

3. The `Release` GitHub Actions workflow builds artifacts on Windows, macOS, and Linux.
4. For prerelease tags, a finalizer verifies Windows, macOS, and Linux asset
   coverage, generates `SHA256SUMS`, and publishes the prerelease. Stable tags
   remain drafts for manual artifact inspection and platform smoke testing.
5. Never reinterpret a failed build or missing platform asset as a releasable result.

## Signing

The current workflow does not configure Windows signing, Apple signing/notarization, or Linux package signing. Signing material must be stored as encrypted GitHub Actions secrets and must never be committed. Public stable releases should not be described as signed until each platform's verification has been completed.

Documentation must keep unsigned or partially signed platform limitations visible until verification is complete. Do not instruct users to disable operating-system security controls globally.

## Versioning

Use semantic versioning:

- Patch: compatible bug fixes
- Minor: compatible features
- Major: incompatible persisted-data, import, report, CLI, or public interface changes

Pre-release tags such as `v0.2.0-preview.4` are published automatically only
after the cross-platform build and checksum finalizer pass. Preview artifacts
remain unsigned until signing credentials are configured and verified.
