# Security Policy

## Supported Versions

Security fixes are applied to the latest code on `main` until versioned releases are published.

## Reporting a Vulnerability

Do not open a public issue for vulnerabilities involving archive parsing, path traversal, local file access, data exposure, or dependency compromise. Use GitHub's private vulnerability reporting feature when enabled for the repository owner.

Include reproduction steps, affected versions or commits, impact, and a minimal sanitized fixture. Never attach real Instagram exports, login secrets, or personal account information.

## Data Boundary

insIGht is local-first and does not require Instagram login details or remote services. Imported archives are untrusted input. Reports and the local SQLite database may contain personal information and should be handled accordingly.
