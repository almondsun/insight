# Security Policy

## Supported Versions

| Version | Security support |
| --- | --- |
| `main` | Active development and primary security-fix target |
| Latest published release | Supported for reports; fixes may require upgrading to a newer release when published |
| Older releases | Not supported |

Security fixes land on `main`. A fix is backported only when the maintainer explicitly publishes a patched release; no long-term-support branch is maintained. The latest v0.1.1 binaries predate security hardening currently on `main`, as documented in [Getting Started](docs/GETTING_STARTED.md#version-differences).

## Reporting a Vulnerability

Do not open a public issue for vulnerabilities involving archive parsing, path traversal, local file access, data exposure, or dependency compromise. Use GitHub's private vulnerability reporting feature when enabled for the repository owner.

Include reproduction steps, affected versions or commits, impact, and a minimal sanitized fixture. Never attach real Instagram exports, login secrets, or personal account information.

## Data Boundary

Nivune is local-first and does not require Instagram login details or remote services. Imported archives are untrusted input. Reports and the local SQLite database may contain personal information and should be handled accordingly.

The current database is not encrypted by Nivune. Fame networking remains unavailable; its proposed trust and network boundaries are documented separately and must not be described as production security guarantees.

See [Privacy and Data Handling](docs/PRIVACY.md) for the complete current-product boundary.
