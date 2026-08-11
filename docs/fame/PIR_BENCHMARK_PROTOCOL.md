# Fame PIR Benchmark Protocol

Status: Phase 1 derived specification

Depends on:

- [Fame Privacy Threat Model](THREAT_MODEL.md)
- [Fame Trust-Bootstrap Specification](TRUST_BOOTSTRAP.md)

Architecture status: frozen

## 1. Decision to be made

Determine whether an independently operated two-server PIR deployment can
serve the licensed Fame corpus while preserving query-index privacy, fixed-shape
real/dummy behavior, query-independent failures, and a globally uniform cover
profile at sustainable client and operator cost.

The benchmark may reject every candidate. Failure to find a qualifying
implementation blocks networked Fame; it does not authorize single-server PIR,
direct lookup, custom unaudited cryptography, or a weaker fallback.

## 2. Required security contract

The minimum covered PIR adversary observes the complete internal state of one
protocol-following replica. At least one of the two independently operated
replicas remains honest and the replicas do not collude.

Each candidate must document:

- Its formal query-privacy game and assumptions.
- Whether privacy is information-theoretic or computational.
- Its concrete security parameters and proof-derived `epsilon_pir` or
  equivalent bound.
- Behavior when a replica, client, or response is malformed or actively
  malicious.
- Query, response, preprocessing, storage, and server-work complexity.
- Side-channel and constant-time claims relevant to query processing.
- Whether batching, caching, SIMD, or sharding changes observable behavior.

Before qualification, an independent cryptographic reviewer must approve a
scheme-level composition memo for the exact pinned implementation and global
configuration. It must cover the replicas' complete transcript and retained
state, request encoding, all probes, preprocessing, randomness, batching,
repeated and adaptive queries, multi-query composition, caching, sharding, and
the resulting overall `epsilon_pir`. A primitive proof alone is insufficient.

Proof-derived privacy is separate from empirical performance. A fast benchmark
cannot establish cryptographic privacy.

## 3. Candidate status

No maintained production-ready open-source two-server PIR service has been
established. Initial evaluation is therefore limited to unapproved candidates
and controls:

| Candidate | Use in this phase | Production status |
| --- | --- | --- |
| [Google Distributed Point Functions](https://github.com/google/distributed_point_functions) | Primary implementation candidate. Current source includes dense, simple-hashed, and cuckoo-hashed DPF-PIR clients, databases, two-server handlers, tests, and benchmarks. | Not approved. The repository is Apache-2.0 and describes itself as unsupported, as-is, and without correctness or security guarantees. It supplies library boundaries rather than independently operated network services. |
| Chor-style two-server XOR PIR | Paper-level security and cost control if an authoritative reviewed implementation is identified. | Do not create or adopt an unaudited production implementation. |
| [SimplePIR/DoublePIR](https://github.com/ahenzinger/simplepir) | Published single-server performance comparator only. | Incompatible with the frozen two-server assumption. |
| [Blyss SDK](https://github.com/blyssprivacy/sdk), [Spiral](https://github.com/menonsamir/spiral-rs), and [SealPIR](https://github.com/microsoft/SealPIR) | Integration and published-cost references only. | Single-server and/or research-grade; not eligible for selection. |

Google's repository provides incremental DPF primitives plus PIR clients,
fixed and hashed databases, request handlers, tests, and microbenchmarks. It
does not provide the independently operated service deployment, mixed
transport, globally signed profile, complete reply-context binding, telemetry,
admission policy, or operational controls required here. Project-owned harness
and service code receives no transitive security claim from the library. It can
establish performance feasibility only and is not eligible for production
promotion without a complete deployable boundary, proof-to-code conformance,
independent implementation and security audits, and an explicit maintenance
and support decision.

### 3.1 Initial inspection record

An isolated checkout inspected on 2026-08-08 resolved to commit
`859cafa71fc1e139c7b76d4d4c0f23438688a8ad`, described by Git as
`v0.0.0-10-g859cafa`. The latest commit date in that checkout is 2026-01-05.

The repository's `pir/` tree contains dense and sparse DPF-PIR implementations,
protobuf request/response types, client state, database code, tests, and Bazel
benchmarks. It supports direct plain requests to two replicas and a
Leader/Helper arrangement. The frozen experiment evaluates the plain
two-replica mode so one service does not become the client-visible concentrator
for both shares.

The inspected protobuf response does not by itself express the full request,
replica, corpus, profile, epoch, and slot binding required by Section 9. That
gap remains a blocking integration contract rather than being assumed solved by
the upstream library.

## 4. Source and supply-chain procedure

For every evaluated repository:

1. Clone into an isolated temporary or benchmark workspace, never into the
   application source tree.
2. Record the upstream URL, verified maintainer signature when available,
   exact commit, and digest-pinned source archive, submodules, dependencies,
   generators, toolchain, and build image. When no signed upstream artifact
   exists, document and independently verify the provenance method.
3. Verify the repository's documented security status and license notices.
4. Build offline and hermetically without mutable remote inputs.
5. Preserve local changes as an explicit patch series.
6. Produce an SBOM, source and build provenance record, reproducible build
   instructions, and independently reproduced binary digest.

Nothing is vendored or merged into Nivune during benchmarking. Vendoring is a
separate decision after security, feasibility, license, and audit gates pass,
and is limited to the smallest required modules.

## 5. Corpus model

The corpus publisher supplies a synthetic benchmark corpus with the same
fixed-width representation, padding, table load, bucket/probe layout, and
release-commitment behavior planned for production. It contains no real
Instagram usernames or account data.

Benchmark dimensions are derived from the licensed provider's documented
coverage estimate:

- One percent of projected production records for rapid profiling.
- Ten percent for scaling and bottleneck analysis.
- One hundred percent for the decisive feasibility run.
- The documented maximum release size for capacity and update testing.

Both replicas receive byte-identical corpus bytes, parameters, index mapping,
padding, and release commitment. Setup and release transitions occur on a
predetermined schedule independent of benchmark query contents.

The final report records record count `N`, fixed record width `B`, encoded
corpus size, load factor, padding, probe count, security parameters, and corpus
digest. Results without these values are not comparable.

## 6. Workload profiles

Before each decisive run, the benchmark coordinator commits publicly to a
secret deterministic seed. Reveal the seed only after immutable replica traces
and results are sealed. Each replica receives only its own query shares and
cannot derive benchmark labels before sealing.

Run separate production-randomness tests for CSPRNG quality, domain separation,
independence, concurrency, restart behavior, and key/nonces never being reused.
Run these profiles separately:

- Uniform valid real queries.
- Valid dummy queries drawn with the identical point-generation procedure.
- Corpus hits, safe misses, and boundary indices.
- Repeated and unique indices.
- Single-client serial load.
- Increasing globally uniform cover populations.
- Sustained steady state and short synchronized epoch boundaries.
- Cold cache, warm cache, and cache-pressure runs.
- Corpus update, replica restart, and version transition.
- One slow, unavailable, corrupt, stale, or actively malformed replica.
- Malformed and abusive client messages within the declared adversary model.

Real and dummy workloads use identical request generation, probes, request and
response bytes, server code path, full-corpus work, validation, reconstruction,
authentication, timeout, and local discard behavior.

Run both paired same-index equivalence tests and distributional tests using the
exact production username-to-probe mapping, scheduler, and dummy selector.
Dummies use the complete valid index distribution; there is no reserved dummy
range, marker, shard, flag, or separate client state transition before the
private-result sink.

## 7. Measurements

Measure per replica and end to end:

- Setup and preprocessing duration, CPU, memory, disk, and temporary storage.
- Query-key generation and client reconstruction CPU, memory, and latency.
- Request and response bytes before and after transport padding.
- Server CPU time, cycles, memory bandwidth, allocations, and bytes read.
- Median and tail latency at every tested concurrency level.
- Throughput, queueing delay, saturation point, and recovery behavior.
- Cold/warm-cache difference, NUMA placement, SIMD requirements, and
  cross-platform variance.
- Cost per globally scheduled slot and cost per active participant.
- Corpus release distribution, activation, and rollback behavior.
- Observable differences among real, dummy, hit, miss, malformed, overload,
  timeout, and replica-failure cases.

Record raw results in a machine-readable, immutable artifact with toolchain,
hardware, OS, firmware, topology, source commit, configuration, and corpus
digest.

## 8. Equivalence tests

Under identical seeds and simulated failures, require exact equality for all
client- and server-controlled real/dummy properties:

- Key, request, and response lengths.
- Probe count and database bytes processed.
- Code-path and workload-class identifiers.
- Validation and authentication operations.
- Padding, timeout, retry, and error decisions.
- Mixed reply construction inputs.
- Telemetry counters permitted by the allowlist.

Timing, cache state, allocation, branch, and hardware-counter traces are
compared statistically under a preregistered method. A difference blocks the
candidate until removed. An exclusion is permitted only when it already exists
in the frozen threat model and preregistration. Post-result observer
reclassification invalidates the run and requires the formal revision process.

A malicious replica's silence cannot be normalized into an inbound packet.
The client must nevertheless preserve query-independent timeout, retry, queue,
future-cover, and local error behavior, while the missing response remains an
explicit observer-model event.

## 9. Fault and abuse tests

Exercise:

- Wrong corpus digest or release version.
- Invalid response share and failed record authentication.
- Missing, duplicated, reordered, delayed, or oversized responses.
- Replica disagreement and asymmetric overload.
- Invalid DPF keys and worst-case valid keys.
- Resource exhaustion and concurrency saturation.
- Restart during corpus transition.
- Stale release replay and incompatible signed parameters.

Every accepted reply must be cryptographically bound to the replica identity,
exact request/share instance, corpus commitment and release, PIR parameters,
global profile, epoch and slot, and fixed response shape. Test both replica
roles across real/dummy, hit/miss, load, transition, replay, substitution,
corruption, delay, and omission cases. A candidate that cannot supply this
binding within the frozen architecture is blocked.

Admission control and overload handling use only protocol-visible fixed-rate
traffic. They cannot inspect or infer real/dummy state, hits, misses, queue
depth, run state, or user demand. Failures never cause direct fallback,
immediate query-dependent retry, or extra slots.

Memory-safe bounded parsing, canonical lengths, fixed resource ceilings, and
cross-request isolation are unconditional. When semantic DPF-key validity
cannot be checked, evidence must show that every accepted byte string,
including worst-case malformed keys, stays within the same bounded work
envelope and cannot corrupt state, affect other requests, or change corpus
integrity. Otherwise record `PIR_FEASIBILITY_BLOCKED`. Adding a new validity
proof or cryptographic mechanism requires architecture review; it cannot be
hidden inside service engineering.

## 10. Independent-operation audit

Verify that the two replica organizations do not share:

- Legal or administrative control.
- Signing, deployment, or observability credentials.
- Request logs, stable identifiers, or trace correlation systems.
- A cloud control plane or operator capable of observing both shares.
- A required ingress or egress vantage point that trivially joins both paths.
- A principal, credential, build pipeline, or update authority able to deploy
  correlating code to both replicas.

Shared libraries and corpus bytes are expected; shared operational control is
not. Each replica publishes a dependency, hosting, jurisdiction, administrator,
and incident-response disclosure for the correlated-compromise model.

The decisive run uses the actual independently administered candidate
environments and named operators. A local simulation may profile performance
but cannot satisfy the independence gate. Any principal able to deploy to or
observe both replicas is recorded as a correlated-compromise dependency.

## 11. Evidence-derived feasibility gates

Do not select numerical thresholds before the corpus model and initial scaling
runs exist. Before the decisive run, preregister:

- Required cryptographic security level.
- Projected cover population and candidate cover-rate range.
- Maximum acceptable client bandwidth, CPU, memory, and energy.
- Required service throughput, latency, availability, and capacity margin.
- Maximum sustainable operator cost.
- Real/dummy equivalence tests and statistical thresholds.
- Covered fault and abuse cases.

The decisive run uses the full projected corpus and preregistered environment.
Changing a threshold, workload, corpus representation, or measurement method
afterward invalidates the decision and requires a new run.

## 12. Decision rules

A candidate qualifies for the next phase only if:

- Its security definition satisfies the frozen two-server assumption.
- Its proof and parameters receive independent cryptographic review.
- The scheme-level composition memo establishes the complete-view bound and
  satisfies the preregistered overall `epsilon_pir`.
- Corpus-scale results meet every preregistered feasibility gate.
- Real/dummy and hit/miss equivalence pass.
- Failure and overload behavior remain query-independent and fail closed.
- Builds and measurements are independently reproducible.
- Memory-safe bounded handling of every accepted request is demonstrated.
- Replies satisfy the complete request, replica, corpus, parameter, profile,
  epoch, slot, and shape binding contract.
- The actual two-replica deployment passes the independent-operation audit.
- Licensing and vendoring obligations are acceptable.
- A complete deployable implementation passes independent implementation,
  security, and proof-to-code conformance audits and has an accepted
  maintenance plan.
- No unresolved critical or high security finding remains.

Malicious-replica fault tests do not extend a privacy proof that covers only a
protocol-following replica. Product claims remain limited to the reviewed
security definition.

If no candidate qualifies, record `PIR_FEASIBILITY_BLOCKED`. Do not weaken the
architecture. Reopening requires evidence that the frozen assumption is
infeasible or insufficient and a formal architecture revision.
