# Fame Privacy Threat Model

Status: Phase 1 specification

Architecture status: frozen

Implementation status: no Fame network functionality is implemented

Next derived document: trust-bootstrap specification

## 1. Purpose

This document defines the security and privacy claims that a future Fame
enrichment system must satisfy. It is intentionally separate from the current
Nivune application, which remains an offline archive-analysis tool.

The proposed system ranks accounts from an imported snapshot using public
follower and following counts. It must not send the imported account, its
relationship graph, or account-derived lookup activity to Instagram or to an
upstream data provider.

The umbrella public claim, if the evidence in this document is eventually
produced, is:

> Bounded, auditable untrackability under an explicit threat model, supported
> by reproducible measurements and independently verifiable assumptions.

This is not a claim of absolute anonymity, zero correlation, or protection
against every global observer.

## 2. Architectural principle

Private user activity must not causally influence upstream collection or
protocol-controlled observable traffic behavior. Network-induced variation
outside protocol control is modeled and measured separately.

Privacy safety takes precedence over retrieval availability. Failure may stop
private retrieval, but it must not weaken cryptography, change the cover
profile according to private work, or activate a fallback path.

## 3. Frozen system model

The architecture contains these roles:

1. One or more licensed data providers supply public profile counts on a
   predetermined schedule that is independent of client activity.
2. A corpus publisher normalizes the data and publishes signed, versioned
   corpus releases.
3. Two PIR replicas hold the same authenticated corpus. Each replica is
   operated by a legally and operationally independent organization.
4. Separately governed mixnet infrastructure carries fixed-shape PIR requests
   and unlinkable fixed-shape replies.
5. A resident Rust client generates globally uniform cover slots. Real PIR
   work replaces dummy work without changing protocol-controlled traffic.
6. Five independent release authorities operate cryptographically separate
   routine 3-of-5 and critical 4-of-5 threshold keys.
7. A transparency log and independent witnesses publish and attest consistent
   protocol, governance, compatibility, and corpus history.
8. Bootstrap directory services and independent time attestors provide only
   the generic discovery and fresh-time evidence required before normal
   participation.

The client WebView has no network capability. Queueing, PIR construction,
validation, scoring, persistence, and export remain in the trusted native
boundary.

The architecture explicitly has no direct Instagram access, direct-network
fallback, miss-driven ingestion, credential support, CAPTCHA bypass, or
activity-dependent proxy behavior.

## 4. Protected assets

The system protects:

- Imported account identity and relationship membership.
- Usernames selected for Fame enrichment.
- PIR indices, corpus hits and misses, and result status.
- Whether a scheduled active-participation slot contains real or dummy work.
- Queue depth, pending-work count, run membership, cancellation, and progress.
- Locally stored Fame observations and pending identifiers at rest.
- Protocol and corpus authenticity, ordering, compatibility, and freshness.
- Governance keys and the independence assumptions of operators and witnesses.

The public corpus, aggregate infrastructure capacity, protocol profile, and
the fact that a device performs cold-start bootstrap are not secret.

## 5. Trust boundaries

### 5.1 Local endpoint

The native process and its approved cryptographic modules are trusted to
protect plaintext account-derived data. The WebView is untrusted for network
access and must receive only typed presentation data.

An unlocked or fully compromised endpoint can disclose imported relationships,
queries, results, and exports. Endpoint compromise is not covered by the
network privacy guarantees.

### 5.2 Corpus boundary

Providers and the corpus publisher are trusted for authorized provenance and
accurate normalization. They are not trusted with client queries. There is no
technical or operational path from client activity, PIR service data, mixnet
traffic, or telemetry to corpus selection or refresh scheduling.

### 5.3 PIR boundary

The minimum covered PIR adversary controls and observes the complete internal
view of one replica while that replica follows the selected PIR protocol. The
query-index privacy guarantee requires that the other replica remains
independent and that the replicas do not collude. Collusion by both replicas
breaks this assumption and is measured only for failure characterization.

An actively malicious replica may return malformed, inconsistent, stale, or no
response. Authentication and fail-closed handling must normalize the client's
local failure state, timeout, retry policy, and future cover behavior without
depending on real/dummy state. A malicious replica cannot be forced to emit a
fixed-shape reply, so missing inbound traffic remains explicit in the PIR and
mixnet observer models. Query privacy against an actively deviating replica is
claimed only if the selected PIR definition explicitly proves it; the PIR
benchmark must distinguish proof-derived privacy bounds from empirical
performance and failure tests.

### 5.4 Mixnet boundary

Relays, gateways, infrastructure providers, and observers may be curious,
compromised, or partially colluding. The model does not assume independent
relay compromise. It accounts for common ownership, administration,
jurisdiction, hosting, network transit, software control planes, coercion,
vulnerabilities, and correlated outages.

The mixnet guarantee is conditional on the topology, batching and latency,
cover population, compromised infrastructure, observation coverage, and the
adversary's end-to-end correlation capability. One honest relay alone is not
claimed to defeat a global observer.

### 5.5 Governance and release boundary

Routine releases require the routine 3-of-5 threshold key. Changes to wire
format, traffic, timing, routing, cryptographic semantics, privacy semantics,
or critical governance require the cryptographically distinct critical 4-of-5
key. A critical action must not be representable or accepted through the
routine signature domain.

Transparency witnesses are trusted only as a quorum. After a client obtains
fresh bootstrap evidence, log inclusion and consistency proofs plus the
configured witness quorum detect network-visible rollback, forks, split views,
and selective publication. Pre-network local rollback detection remains
dependent on the platform assurance class in Section 10.

### 5.6 Bootstrap trust services

The transparency log, witnesses, bootstrap directory, and time attestors are
separate trust roles even when the trust-bootstrap specification assigns more
than one role to an operator. The client tolerates a malicious log that cannot
obtain the required witness quorum, a compromised witness minority, a
compromised time-attestor minority, and stale or replayed directory material.

A compromised witness quorum, time-attestation quorum, or applicable
governance threshold violates the bootstrap trust assumption. Bootstrap
directory compromise must not authorize new trust keys; directory material is
accepted only when its monotonic sequence, required signature, transparency
history, and witness attestations validate from the embedded or retained trust
root. Exact role counts, independence requirements, and quorum intersections
are fixed by the trust-bootstrap specification before implementation.

## 6. Adversaries

The formal analysis and experiments must address these adversaries separately:

- A passive observer of the client access network.
- One curious or compromised PIR replica.
- Curious or compromised mix relays in correlated ownership groups.
- Partial ingress, egress, and relay observers.
- Colluding infrastructure within the explicitly declared compromise model.
- A malicious corpus replica returning corrupt, stale, or inconsistent data.
- A malicious or compromised routine release minority.
- A malicious transparency log without the required witness quorum.
- A compromised witness or time-attestor minority.
- A malicious or stale bootstrap directory and replayed bootstrap material.
- A network attacker replaying, delaying, dropping, duplicating, or reordering
  bootstrap and active-protocol traffic.
- A local attacker restoring a previously valid application state.
- Statistical classifiers attempting to distinguish real from dummy activity
  or link senders to queries.

The following violate assumptions or remain outside the guaranteed boundary:

- Both PIR replicas colluding.
- A global passive observer with comprehensive end-to-end visibility.
- Any bounded global-observation capability not explicitly fixed as a covered
  adversary before preregistration.
- Full endpoint compromise while private data is accessible.
- Compromise of a valid governance threshold.
- Malicious upstream data that carries valid authorized publisher signatures,
  except where provenance and audit controls detect it.
- User disclosure through screenshots, exports, or other out-of-band actions.

Excluded cases must never be described as covered adversaries in product
language or audit reports.

## 7. Formal security properties

All advantage bounds below are symbolic until the threat-model and benchmark
phases justify numerical thresholds. Final thresholds must be preregistered
before decisive evaluation.

### 7.1 Query-index privacy

For any two valid indices chosen by an adversary controlling one PIR replica,
the adversary receives a challenge view for one index. Its advantage over
random guessing must be at most `epsilon_pir` under the selected PIR security
definition and parameters.

Real and dummy operations use the same PIR construction, probe count,
randomness quality, validation, workload class, database-access behavior,
response size, and error handling. Dummy-safe positions must not create a
distinct cache, shard, computation, or access pattern.

### 7.2 Sender-query unlinkability

Given active senders and valid PIR operations, an adversary within the covered
mixnet observation model attempts to link a sender to a request or reply. Its
advantage must be at most `epsilon_link` for the preregistered topology,
traffic profile, participation population, and correlated compromise model.

Requests and replies use mixed paths. A PIR replica receives no stable client
identifier, direct response connection, exact build identity, platform, or
rollback-assurance class.

### 7.3 Activity unobservability

During an active participation interval, an adversary receives a trace from
one of two worlds: all eligible slots are dummy, or private work replaces some
dummy slots. Its distinguishing advantage must be at most `epsilon_activity`.

Distinguishing advantage is the primary acceptance metric. ROC AUC, balanced
accuracy, TPR/FPR, and calibration are diagnostic unless preregistered as
additional gates.

The property covers protocol-controlled wire format, logical sizes, cover
schedule, timing distribution, routing policy, retry semantics, failure
behavior, and request/reply processing. OS and network-stack behavior such as
MTU handling, congestion control, socket scheduling, and retransmissions is
outside direct protocol control, but must be modeled and minimized rather than
silently treated as indistinguishable.

### 7.4 Collection independence

Consider two worlds with identical provider inputs and predetermined ingestion
schedules but arbitrary differences in queries, hits, misses, queue state,
runs, popularity, timing, errors, cancellation, and telemetry. Corpus
membership, refresh priority, upstream retrieval, and future collection must
be identical.

This is a causal non-interference requirement, not a best-effort policy.

### 7.5 Release authenticity, consistency, and rollback resistance

A client accepts a release only if it has:

- A valid signature under the action's required threshold key.
- A locally compatible signed client policy.
- A valid transparency inclusion proof.
- A consistency proof from the client's highest trusted checkpoint.
- The required independent witness attestations.
- Fresh authenticated time proving activation and non-expiry.
- Monotonic versions no lower than the applicable assurance-class floor:
  state validated by the qualifying platform primitive for `strong`
  configurations, or freshly re-established witnessed transparency state for
  `weaker` configurations.

The authenticated compatibility tuple is:

`client compatibility policy x protocol manifest x corpus release x PIR parameters x cryptographic suite x epoch policy`

Compatibility is enforced locally. Exact build, application version, OS,
architecture, policy branch, capability set, and rollback class are not sent
to PIR, mixnet, corpus, transparency, or telemetry services. All admitted
clients using one global profile produce the same protocol-controlled behavior.

### 7.6 Bootstrap purpose non-interference

Consider two cold starts with the same public trust state and modeled network
conditions but arbitrary differences in imported accounts, Fame runs, queues,
corpus interests, and future work. The protocol-controlled bootstrap trace --
including request selection, endpoints, counts, logical sizes, initiation,
timing distribution, recovery, and visible failures -- must be identically
distributed. Platform and external network effects are recorded separately in
the bootstrap observer model.

### 7.7 Cross-build profile equivalence

For any two locally compatible client builds admitted to the same signed
global profile, and for the same public inputs, query-independent randomness,
and simulated network events, their protocol-controlled bootstrap and active
traces must be identical. Exact build, version, platform, capability, policy
branch, and rollback class must not change a network-visible protocol choice.
Live platform and network-stack differences are measured as external effects
in the traffic-analysis phase rather than assumed away.

## 8. Bootstrap security and observability

Cold-start bootstrap is outside the active-cover activity-unobservability
guarantee. It may reveal that a device is attempting to participate, but it
must reveal nothing about why.

Bootstrap traffic contains only generic requests for transparency consistency,
witness attestations, directory discovery, and fresh time. It carries no Fame
query, username, imported-account data, corpus interest, queue or run state,
pending-work count, exact client version, platform, local capability, future
workload, or stable identifier across sessions.

The mandatory non-circular dependency order is:

1. Load the embedded trust root and highest locally retained state.
2. Establish monotonic transparency consistency and witness quorum.
3. Establish fresh authenticated time using an unpredictable per-bootstrap
   client nonce bound into every accepted time attestation.
4. Validate expiry, activation, client compatibility, and the complete
   compatibility tuple.
5. Begin normal participation only at a valid global epoch boundary.

The embedded trust root contains the governance, witness, time-attestation,
transparency-log, and bootstrap-directory verification material required for
these steps. Bootstrap directory and key evolution rely on authenticated
monotonic sequence, signatures, consistency proofs, and witnesses rather than
wall-clock time alone.

Authentication, ordering, freshness, and validity are distinct:

- Signatures establish authenticity.
- Monotonic sequences and transparency proofs establish ordering and history
  consistency.
- Nonce-bound attestations establish freshness.
- Authenticated time establishes expiry and activation validity.

Previously valid recorded material cannot answer a new freshness challenge.
No artifact may depend for freshness solely on another artifact whose own
validity depends on the same not-yet-established time state.

IP-, transport-, and network-layer observations during bootstrap are recorded
as bootstrap leakage, not claimed to be hidden by active cover traffic.

## 9. Participation and failure state machine

The threat model recognizes these externally relevant states:

- `offline`: no privacy-protocol traffic.
- `bootstrap`: generic trust, transparency, directory, and time operations.
- `waiting_for_epoch`: prerequisites are valid; no early participation.
- `active`: the globally signed cover profile is running.
- `suspended`: sleep, shutdown, connectivity loss, or safety failure prevents
  safe participation.

Start, stop, restart, prolonged suspension, and connectivity boundaries may be
observable. The active activity-unobservability guarantee applies only within
`active` intervals.

Missed slots are discarded. They are never replayed, compressed, caught up, or
compensated. Recovery occurs only at a future global epoch boundary after all
security prerequisites are re-established.

If enough infrastructure remains to execute the signed profile safely, normal
slots continue with identical dummy substitution and fixed-shape failures. If
safe execution is impossible, participation stops according to the global
failure rule, independently of private queue state.

## 10. Time and local rollback

Authenticated time is:

`authentic source + freshness + assurance-class anti-rollback state`

After startup, reboot, monotonic-clock reset, or excessive uncertainty, the
client generates an unpredictable nonce and obtains quorum attestations bound
to that nonce, bounded time intervals, the current witnessed checkpoint, and
the protocol identity. Previously recorded attestations are insufficient.

For a `strong` configuration, the client must successfully use its qualifying
platform primitive to detect restoration before normal network participation
and enforce any monotonic check specific to that primitive. For a `weaker`
configuration, no pre-network rollback-detection claim is made: the acceptance
floor is re-established from fresh time and the current witnessed transparency
history before normal participation.

Within a boot session, elapsed time uses a monotonic clock. Wall-clock changes
cannot advance, reverse, or extend accepted protocol state. Rollback,
contradictory attestations, excessive uncertainty, or discontinuity stops
retrieval until fresh time is established and the next epoch boundary arrives.

Encrypted storage alone does not provide rollback resistance. Phase 1 assigns
each Windows, macOS, Linux, hardware, and virtualization configuration one of
two assurance levels:

- `strong`: a demonstrated platform primitive detects restoration of a
  previously valid protected state before normal network participation.
- `weaker`: rollback is detected only after fresh time and current witnessed
  transparency state are re-established.

The platform matrix must cover privileges, availability, reboot, application
update, disk restoration, cloning, VM snapshots, hardware replacement,
counter reset, secure-hardware failure, atomic updates, and crash recovery.
Hardware-backed key storage alone is not evidence of rollback resistance.

No remote per-client rollback anchor is introduced without a formal
architecture revision and correlation analysis.

## 11. Overload, abuse, and telemetry

Admission control, load shedding, rate limiting, relay replacement, retries,
and abuse mitigation operate only on protocol-visible cover traffic. They do
not use real/dummy state, usernames, indices, hits, misses, queues, runs, or
private demand.

Overload responses are fixed-shape and use the same reply behavior. A failed
real operation can only replace a future ordinary dummy slot. There is no
immediate retry, alternative route caused by private state, or fallback.

Telemetry is allowlisted and aggregate. It excludes usernames, indices,
hit/miss state, source addresses, stable identifiers, exact client versions,
platforms, rollback classes, queues, runs, per-user demand, and route histories.
Ephemeral transport metadata must not enter persistent request logs.

## 12. Safety invariants

Bootstrap failure, uncertain time, uncertain rollback state, incompatible
state, missing governance or witness quorum, transparency inconsistency, PIR
failure, and mixnet failure may disable retrieval. They must never:

- Weaken a cryptographic or independence requirement.
- Activate direct access or fallback transport.
- Change cover behavior according to private workload.
- Introduce query-dependent retry, routing, or recovery.
- Accept stale, downgraded, forked, expired, or incompletely authorized state.
- Distinguish real operations from dummy operations.

An availability failure becomes a privacy failure when it creates
query-dependent or distinguishable behavior.

## 13. Required evidence and gates

Numerical thresholds are evidence-derived rather than chosen in this document.
Before decisive evaluation, the project must preregister:

- Covered adversary configurations and violated assumptions.
- Corpus scale and PIR security parameters.
- Cover population and effective anonymity measures.
- Mix topology, batching, latency, route diversity, and correlated compromise
  model.
- Observation positions and exposure assumptions.
- Capacity and failure-domain measurements.
- Trace datasets, train/test separation, classifiers, statistical tests,
  confidence levels, and acceptance thresholds.
- `epsilon_pir`, `epsilon_link`, and `epsilon_activity`, with the derivation for
  each bound.

Deterministic tests require exact equality of client-controlled packet counts,
logical sizes, scheduled times, route decisions, retry decisions, timeout
behavior, and visible protocol errors under identical seeds and simulated
network events. Live-network traces are evaluated statistically because loss,
latency, and relay behavior are not fully client-controlled.

Independent auditors must reproduce the decisive evaluation. Changing a
threshold, dataset, adversary view, or method after evaluation invalidates that
run and requires a new preregistration.

## 14. Phase exit criteria

The formal threat-model phase is complete only when review confirms that:

- Every public claim maps to a formal property and covered adversary.
- Assumptions, violations, exclusions, and participation boundaries are
  explicit.
- The bootstrap dependency graph is non-circular.
- Compatibility does not create protocol-controlled client fingerprints.
- Governance actions map unambiguously to their cryptographic quorum.
- Strong rollback claims are limited to demonstrated platform capabilities.
- Safety invariants and fail-closed transitions are complete.
- Evidence-derived metrics have named derivation points in subsequent phases.

The locked derived-document order is:

1. Trust-bootstrap specification.
2. PIR benchmark protocol.
3. Mixnet experiment specification.
4. Traffic-analysis preregistration.

The architecture may be reopened only when a derived phase, implementation
evidence, or an independent audit shows that a frozen assumption is infeasible,
an intended guarantee cannot meet its preregistered criterion, or a material
deficiency cannot be corrected within the existing architecture. A stronger
theoretical mechanism by itself is not sufficient.
