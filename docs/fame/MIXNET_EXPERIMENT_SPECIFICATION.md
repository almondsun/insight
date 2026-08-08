# Fame Mixnet Experiment Specification

Status: Phase 1 derived specification

Depends on:

- [Fame Privacy Threat Model](THREAT_MODEL.md)
- [Fame Trust-Bootstrap Specification](TRUST_BOOTSTRAP.md)
- [Fame PIR Benchmark Protocol](PIR_BENCHMARK_PROTOCOL.md)

Architecture status: frozen

## 1. Decision to be made

Determine whether a maintained mixnet can carry the frozen two-server PIR
request and reply protocol while meeting preregistered sender-query
unlinkability and activity-unobservability criteria under an explicit,
correlated compromise model.

The experiment may reject every candidate. Failure does not authorize direct
transport, Tor fallback, a stable reply connection, adaptive cover, or weaker
operator-independence assumptions.

## 2. Claims under test

The experiment evaluates two properties separately:

- `sender-query unlinkability`: a covered observer cannot link an active sender
  to a PIR request or reply beyond preregistered advantage `epsilon_link`.
- `activity unobservability`: during an active participation interval, a
  covered observer cannot distinguish real-query substitution from dummy-only
  operation beyond preregistered advantage `epsilon_activity`.

For sender linkage, construct matched trials with at least two simultaneously
active senders, the same public traffic and request/reply multiset, and a secret
uniform permutation assigning operations to senders. In the primary balanced
pairwise game, the adversary nominates two valid assignments, the challenger
samples a secret uniform bit to select one, and the adversary guesses that bit
from its covered view. Define

`Adv_link = abs(2 * accuracy - 1)`

Report request-path, reply-path, and joint results separately. The upper
confidence bound of the maximum covered advantage must not exceed the
preregistered `epsilon_link`.

Cold-start bootstrap, participation start/stop, long suspension, shutdown, and
connectivity boundaries are outside the active-cover claim and receive their
own leakage analysis.

A comprehensive global passive observer is outside the guaranteed boundary.
Any bounded multi-vantage observer proposed for coverage must be fixed before
preregistration rather than selected after results are known.

## 3. Candidate status

No candidate is approved before experimentation.

| Candidate | Experiment role | Material risks |
| --- | --- | --- |
| [Katzenpost](https://github.com/katzenpost/katzenpost) | Primary protocol-fit candidate. It supplies fixed-length Sphinx packet classes, service-node queries, SURBs, reply queues, randomized per-hop delay, and real-or-decoy scheduling. | AGPL-3.0 integration obligations, 0.x maturity, specification/version drift, test-network emphasis, unreliable delivery, entry-gateway volume visibility, and possible long-term disclosure attacks. |
| [Nym](https://github.com/nymtech/nym) | Operational-maturity and Rust-integration baseline. It supplies released binaries, a public operator ecosystem, Poisson real-or-cover queues, and SURB replies. | Multiple packet and ACK classes, fragmentation, SURB replenishment, adaptive backpressure pacing, configurable cover bypass, opaque sender tags, and mixed per-crate licensing. |

Both are continuous-time randomized-delay mixnets rather than batch/shuffle
mixes. Their upstream anonymity claims are inputs to review, not evidence that
this project's profile meets its own threat model.

Pin an exact tag, commit, packet geometry, directory behavior, topology,
scheduler, delay distribution, and dependency graph for each experiment.
Draft or conflicting upstream documentation is resolved against the pinned
code and recorded as a specification discrepancy.

## 4. Source and deployment provenance

Use the same isolated, hermetic source procedure required by the PIR benchmark:

- Clone outside the application tree.
- Verify available maintainer signatures and digest-pin all source, submodules,
  dependencies, generators, toolchains, and images.
- Build offline and emit an SBOM and provenance record.
- Preserve all profile changes as an explicit patch and configuration set.
- Independently reproduce experiment binaries.

Do not vendor or merge either candidate during this phase. Licensing review
must specifically address the boundary between the desktop application,
resident service, client library or daemon, and deployed mix infrastructure.

## 5. Locked experiment profile

For each candidate, construct one globally uniform `MixExperimentProfile`
containing:

- Packet type, payload geometry, padding, fragmentation prohibition or rule,
  ACK behavior, and reply geometry.
- Active-participation slot process and provisional send-rate range.
- Per-hop delay distribution and all randomized timing parameters.
- Route length, eligible topology, and route-selection distribution.
- Connection reuse, gateway behavior, mailbox policy, and session lifetime.
- Number and construction of single-use reply blocks per logical slot.
- Timeout, relay replacement, retry, overload, and recovery behavior.
- Directory epoch and profile activation rules.
- Telemetry allowlist and capture points used only for the experiment.

The profile is identical for every compatible build. Application version,
platform, queue state, real/dummy state, and demand cannot change it.

Adaptive pacing, cover disabling, workload-dependent backpressure, automatic
retries, SURB replenishment, packet-class switching, or fragmentation is
disabled unless the experiment proves that the behavior is itself fixed by the
global profile and independent of private work.

Each candidate must provide a complete observable state machine from process
startup through bootstrap, directory discovery, gateway registration,
authentication, transport and version negotiation, mailbox setup, replay-state
handling, waiting, active participation, suspension, and shutdown. Inventory
every control flow, credential, identifier, ACK, sender tag, directory fetch,
and reply-block replenishment message.

Classify each flow as bootstrap, waiting, active, or shutdown. Bootstrap flows
must satisfy the generic bootstrap private-state and cross-build
non-interference rules. Active control traffic is part of the global profile
and equivalence tests. Mandatory candidate behavior that exposes exact build or
platform, creates a stable cross-session identity, or depends on private work
blocks the candidate.

## 6. Real and dummy request path

Each scheduled logical slot always enters one canonical end-to-end transcript.
Let the signed PIR profile require `P` fixed probes per logical operation. The
resident scheduler produces two shares for every probe, one for each replica,
so every slot schedules exactly `2 * P` fixed-shape mixed requests and allocates
exactly `2 * P` fixed-shape unlinkable reply paths.

The private queue supplies either a valid real operation or a freshly generated
valid dummy operation to an opaque payload boundary. Both produce the same
probe/share vector shape. Everything after that boundary is independent of the
selection.

Packetization, padding, ACK/control messages, reply blocks, and local state
transitions for the complete two-replica transcript are fixed by the global
profile. Route randomness is generated from the same CSPRNG and domain-separated
by slot, replica, probe, and direction without using private payload state.

Real and dummy requests use identical:

- PIR share and application framing length.
- Packet class, padding, fragmentation, and ACK behavior.
- Route-selection distribution and connection policy.
- Timing and per-hop delay distribution.
- Service destination class and server processing contract.
- Timeout, overload, relay replacement, and failure handling.

The entry gateway may observe participation and approximate active volume. It
must not receive a build identifier, queue signal, or a protocol-visible change
between real and dummy slots.

Full-service dummies execute the same work at both replicas. Test all partial
outcomes: both replies, either single reply, no replies, corrupt shares, mixed
delay, and asymmetric overload. These outcomes cannot change request counts,
future cover, route selection, or immediate retry behavior.

## 7. Unlinkable reply path

Every real and dummy request carries the same fixed number and format of
single-use reply blocks or the candidate's equivalent unlinkable reply
mechanism.

Each PIR replica returns the same fixed-size encrypted response through a
separately selected mixed reply path. The replica receives no source address,
stable client identity, reusable reply address, direct socket, exact build,
platform, assurance class, or query-state signal.

Reply credentials are scoped to one slot and never reused. Sender tags,
mailbox identifiers, ACKs, reply-block replenishment, and fragmentation are
treated as part of the observable protocol and must not become stable or
real/dummy-dependent identifiers.

A malicious or failed replica cannot be forced to send a reply. Missing inbound
traffic remains part of the observer model. Client-controlled timeout, local
failure, queueing, future cover, and retry behavior remain identical regardless
of whether the missing response corresponded to real or dummy work.

## 8. Topology and correlated compromise model

The experiment records every relay, gateway, service node, directory authority,
hosting provider, autonomous system, jurisdiction, administrator, build/update
principal, and observability system.

Group nodes into correlated compromise domains using:

- Legal and beneficial ownership.
- Administrative credentials and deployment control.
- Cloud, hosting, network transit, DNS, certificate, and monitoring providers.
- Jurisdiction and coercion exposure.
- Shared software supply chain and vulnerability class.
- Common incident-response and update authority.

Do not calculate route compromise by multiplying independent per-node
probabilities. Evaluate compromise by correlated-domain scenarios and
sensitivity ranges.

The candidate profile must prevent one operator or correlated domain from
occupying multiple route layers. Inability to meet preregistered route/domain
diversity blocks the candidate. Any explicitly retained overlap is a covered
compromise scenario and must still pass `epsilon_link`; it is not an exception.
Actual experiment routes, not advertised operator counts, determine diversity.

## 9. Experiment worlds

Use sanitized synthetic PIR traffic and run at least these paired worlds:

- Dummy-only versus real-query substitution.
- Hits versus safe misses.
- Repeated versus unique private indices.
- Empty versus backlogged private queue with the same cover profile.
- Successful versus corrupt, delayed, or absent PIR reply.
- Idle UI versus active Fame UI while the resident service remains active.
- Compatible client builds implementing the same signed profile.
- Supported OS and network stacks under matched public inputs.
- Multiple active senders with secret randomized pairwise request and reply
  assignments for the executable sender-linkage game.

Run separate participation-boundary experiments for first bootstrap, restart,
short loss, prolonged suspension, shutdown, and re-entry at an epoch boundary.
Do not combine their leakage with the active-world result.

For bootstrap and candidate control-plane traces, run matched worlds that vary
private state and compatible build while holding public state and modeled
network events fixed. Endpoint choice, artifacts, counts, sizes, timing,
credentials, negotiation, retries, and failures must satisfy the applicable
non-interference property.

## 10. Observer views

Capture and analyze each view independently and in preregistered combinations:

- Client access network.
- Entry gateway.
- Individual and correlated groups of mix layers.
- PIR service ingress and egress.
- Reply gateway or mailbox.
- Partial ingress-plus-egress observers.
- Hosting, autonomous-system, and shared-control-plane vantage points.
- Bounded multi-vantage observers fixed before evaluation.

Raw research capture contains no real usernames or personal archives. Access is
restricted, retention is defined before collection, and released evidence is
sanitized without removing features used by the preregistered classifiers.

## 11. Deterministic equivalence harness

With identical seeds and simulated network events, require exact equality of
all protocol-controlled real/dummy traces:

- Logical packet counts and sizes in both directions.
- Scheduled send times and delay samples.
- Route choices and reply-path construction.
- Connection reuse and gateway/mailbox operations.
- ACK, fragmentation, and reply-block behavior.
- Timeout, retry, relay replacement, overload, and recovery decisions.
- Externally visible protocol errors and allowed telemetry.

Also compare compatible builds under the same profile. Any build-dependent
protocol trace is a compatibility fingerprint and blocks that build.

## 12. Live-network measurements

Measure:

- End-to-end request and reply latency distributions.
- Loss, duplication, reordering, corruption, and timeout rates.
- Packet geometry at every observable boundary.
- Cover overhead, CPU, memory, bandwidth, wakeups, and energy.
- Queue stability and completion time over the projected corpus population.
- Route and correlated-domain diversity.
- Gateway volume and session-linkability leakage.
- Sender-query and real/dummy classifier features and performance.
- Long-duration intersection and disclosure signals.
- Behavior during directory changes, relay churn, overload, and partitions.

Distinguishing advantage is the primary privacy metric. ROC AUC, balanced
accuracy, TPR/FPR, calibration, and feature importance are diagnostics unless
the traffic-analysis preregistration promotes them to additional gates.

## 13. Failure and overload behavior

Inject relay loss, malicious delay, dropped requests, missing replies, stale
directories, gateway failure, mailbox loss, overload, connection churn, and
network partitions.

Also exercise replay, duplication, tagging and watermarking, selective delay or
drop, n-1 and flooding attacks, malicious gateways and service nodes, SURB
replay, route capture, and colluding request/reply observation within each
covered adversary model.

An independent exact-profile security memo must map the pinned implementation,
patches, cryptographic protections, scheduler, request/reply construction, and
topology assumptions to every covered passive and active adversary. Empirical
classification under benign traffic cannot replace this composition analysis.
An uncovered required threat-model behavior blocks the candidate.

Admission, rate limits, load shedding, and recovery operate only on
protocol-visible cover traffic. They cannot inspect or infer real/dummy state,
PIR index, result, queue depth, or run state.

Admissible state is limited to ephemeral connection conformance with the signed
global slot and packet profile plus aggregate infrastructure saturation. Any IP,
gateway credential, or session key visible for enforcement is documented as
gateway leakage, is not persisted into telemetry, and cannot select a different
cover profile. No admission identifier is exposed to PIR replicas.

When transport permits a response, overload uses the same fixed-shape mixed
reply class. When it does not, the missing reply is explicitly modeled. Exact
cover-trace equivalence must hold through saturation and recovery.

Missed slots are discarded. Failures do not cause immediate private retries,
catch-up bursts, alternate direct routes, per-client cover changes, or fallback
transport. Real work can only replace a future normal dummy slot.

When the signed profile cannot run safely, active participation stops under the
global failure rule and resumes only at a future epoch boundary after the
trust-bootstrap prerequisites succeed.

## 14. Evidence-derived gates

Initial runs derive feasible ranges; they do not set production claims. Before
the decisive evaluation, preregister:

- Covered observer and correlated compromise scenarios.
- Minimum cover population and effective anonymity measures.
- Required route/domain diversity and observation exposure.
- Provisional global cover profile selected from measured PIR and mix costs.
- Maximum client resource and infrastructure cost.
- `epsilon_link` and `epsilon_activity` with confidence procedures.
- Required availability and capacity margin.
- Deterministic and statistical equivalence tests.
- Long-duration observation window and disclosure tests.
- Immutable profile, topology, capture-pipeline, dataset-construction,
  partition, feature, classifier, hyperparameter, statistical-test,
  confidence, multiplicity, and analysis-code hashes required by the Traffic
  Analysis Preregistration.

Post-result changes to observers, thresholds, profiles, features, or methods
invalidate the run and require new preregistration.

## 15. Decision rules

A candidate proceeds only if:

- Its pinned profile implements fixed-shape mixed requests and unlinkable
  fixed-shape replies without stable service-visible client identity.
- Real/dummy deterministic equivalence passes.
- Live measurements meet preregistered unlinkability and unobservability gates.
- Correlated-domain topology and operator independence meet the threat model.
- Failure, overload, retry, and recovery remain query-independent.
- Client cost, service capacity, and completion time meet evidence-derived
  feasibility gates.
- Source, binaries, configuration, and results are independently reproducible.
- Licensing permits the intended deployment and distribution boundary.
- Independent security and traffic-analysis review has no unresolved critical
  or high finding.

If neither candidate qualifies, record `MIXNET_FEASIBILITY_BLOCKED`. Do not
weaken the frozen architecture. Reopening requires evidence that a frozen
assumption is infeasible or insufficient and a formal revision.
