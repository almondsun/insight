# Product Roadmap

This roadmap separates features people can use today from work that requires further implementation or external evidence. It is directional, not a release-date commitment.

## Available today

The current desktop application provides:

- Complete Instagram JSON ZIP and extracted-folder imports.
- Followers, following, mutuals, and both non-reciprocal categories.
- Searchable local relationship lists.
- Multiple account histories and immutable snapshots.
- Additions and removals for a selected snapshot and its immediately prior import.
- CSV and JSON relationship and change exports.
- Account rename, full-account deletion, and a read-only Fame engineering status view.
- Local SQLite persistence with no Instagram login, scraping, upload, or telemetry.

## Product hardening

The next conventional product work is to strengthen distribution and usability without changing the local-first boundary:

- Sign and notarize release artifacts for supported platforms.
- Expand platform and installer smoke testing.
- Continue expanding platform and installer smoke coverage.
- Keep import compatibility aligned with documented Instagram JSON export changes.

Specific items enter a release only after implementation and validation; their presence here does not mean they are available.

## Fame research

Fame is a proposed opt-in ranking of accounts in the latest snapshot's followers and following union, excluding the imported account owner. The versioned `fame-v1` formula, persistence foundation, synthetic corpus records, protocol models, and laboratory scheduler scaffolding are implemented.

The following are not implemented or approved:

- Licensed, query-independent corpus ingestion and signed production releases.
- A deployable two-server PIR system run by legally and operationally independent organizations.
- Separately governed mixed request and reply paths with fixed-rate cover behavior.
- Threshold governance, witnessed transparency, fresh authenticated time, and platform rollback adapters.
- End-to-end dummy and real wire equivalence.
- Production privacy parameters, reproducible traffic-analysis evidence, and independent audits.
- A user consent, run, progress, ranking, refresh, or Fame-result export interface.

Fame must progress in the locked order:

1. Formal threat model.
2. Trust-bootstrap specification.
3. PIR benchmark protocol.
4. Mixnet experiment specification.
5. Traffic-analysis preregistration.
6. Integrated prototype only after earlier gates pass.
7. Independent audit and experimental deployment.

The architecture forbids direct Instagram access, credentials, fallback transport, miss-driven collection, CAPTCHA bypass, and absolute anonymity claims. Production retrieval remains disabled unless its preregistered feasibility and privacy gates pass.

Read the [Fame engineering index](fame/README.md) and [current implementation status](fame/IMPLEMENTATION_STATUS.md) for the authoritative details.

## Reopening the Fame architecture

The frozen architecture is reopened only if a derived phase, implementation evidence, benchmark, or independent audit demonstrates that an existing assumption is infeasible or insufficient. A theoretically stronger mechanism by itself is not enough.
