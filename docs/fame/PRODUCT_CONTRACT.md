# Fame Product Contract

Status: implementation contract for `fame-v1`

The privacy architecture remains governed by the frozen documents in this
directory. This contract defines product behavior; it does not approve a PIR,
mixnet, corpus provider, operator, or production privacy claim.

## Population and owner exclusion

The default population is the normalized union of followers and following in
the latest snapshot. The imported account owner is excluded locally before any
private queue is constructed. When authoritative owner metadata is absent, the
user must enter and confirm the owner username locally. It is never sent to the
corpus, PIR replicas, mixnet, bootstrap services, or telemetry.

Usernames are trimmed, lowercased for identity, limited to 30 ASCII characters,
and contain only letters, digits, period, and underscore. Display casing is
preserved separately.

## Observations and statuses

An observation contains normalized username, follower count, following count,
precision (`exact` or `approximate`), observation time, corpus release, source,
and authenticated record status. Run membership is immutable.

Networked statuses are `pending`, `exact`, `approximate`, `private`, `missing`,
`blocked`, `failed`, and `cancelled`. Only exact and approximate observations
are ranked. Other rows appear after ranked results in normalized-username order.
A miss never triggers collection or refresh.

New runs reuse the newest authenticated observation for each member by default.
Refresh-all creates a new historical run. Account or snapshot deletion cascades
through run membership; observations with no remaining reference are removed.

## Formula and ordering

For followers `F` and following `G`, `fame-v1` is:

```text
F * log2(2F / (F + G)) + G * log2(2G / (F + G)), when F > G
0, otherwise
```

A logarithmic term with a zero count contributes zero. Results must be finite.
Rows sort by score descending, followers descending, then normalized username
ascending. Rank is one-based. Counts, score, precision, observation time,
source, corpus release, and formula version remain available for audit/export.

## Lifecycle and interfaces

Only one Fame run may be active globally. Runs are explicitly started, may be
cancelled, and may resume only through future normal cover slots. Cancellation,
misses, failures, queue state, and run membership never affect cover behavior or
corpus ingestion.

The native boundary will expose consent, participation status, run start,
resume, cancellation, inspection, refresh-all, deletion, and versioned CSV/JSON
export. Progress events contain aggregate completed/total counts and public
failure state only; they never contain private usernames or indices.

Until the frozen feasibility and audit gates pass, these networked interfaces
remain unavailable and the application continues to operate offline.
