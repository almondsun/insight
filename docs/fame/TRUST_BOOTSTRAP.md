# Fame Trust-Bootstrap Specification

Status: Phase 1 derived specification

Depends on: [Fame Privacy Threat Model](THREAT_MODEL.md)

Architecture status: frozen

## 1. Scope

This document specifies how a cold client establishes enough trustworthy,
fresh, and ordered public state to enter the normal Fame privacy protocol. It
does not make bootstrap traffic anonymous. Bootstrap may reveal that a device
is attempting to participate, but it must carry no account-derived signal.

The mandatory dependency order is:

`embedded trust root -> transparency consistency and witness quorum -> nonce-bound fresh time -> expiry, activation, and compatibility validation -> normal participation`

The protocol fails closed at every transition. It has no direct Instagram or
alternate retrieval path.

## 2. Roles and quorum assumptions

| Role | Population | Acceptance rule | Assumption violation |
| --- | ---: | --- | --- |
| Routine release authority | 5 organizations | Routine 3-of-5 threshold signature | Valid routine-key threshold compromised |
| Critical governance authority | Same 5 organizations, separate key shares | Critical 4-of-5 threshold signature | Valid critical-key threshold compromised |
| Transparency log | 1 append-only log | Valid proof plus witness quorum | Log and witness quorum equivocate together |
| Transparency witness | 3 independent organizations | Matching 2-of-3 attestations | 2 witnesses attest a fork or invalid history |
| Time attestor | 3 independent organizations | Overlapping 2-of-3 fresh intervals | 2 attestors produce coordinated false time |
| Bootstrap directory publisher | Critical governance | Critical 4-of-5 signature and witnessed log inclusion | Critical threshold compromised |

Release authorities use two independently generated threshold keys. The
routine key cannot verify a critical action, and the critical action schema is
not valid in the routine signature domain.

Witness and time-attestor operator identities, legal ownership, administrators,
hosting dependencies, jurisdictions, and key custody are published. Apparent
operator count does not satisfy independence when these dependencies are
shared.

## 3. Embedded trust root

Every admitted client build embeds the same immutable `GenesisTrustRoot`
containing:

- Format, protocol lineage, and genesis identifier.
- Routine and critical threshold public keys and signature-suite identifiers.
- Transparency-log public key and an initial signed tree checkpoint.
- Three witness public keys and the 2-of-3 policy.
- Three time-attestor public keys and the 2-of-3 policy.
- Bootstrap-directory verification policy.
- Genesis bootstrap endpoints and pinned transport keys for the log, witnesses,
  and time attestors.
- Domain-separation labels for every signed artifact.
- Hashes of the canonical artifact schemas needed before compatibility checks.

An admitted build may also embed a newer `BootstrapCache` containing directory
endpoints, checkpoints, and public state. The cache is only an optimization: it
must carry the critical 4-of-5 authorization, inclusion proof, consistency
chain, and witness attestations connecting it to the immutable genesis root. A
new build cannot replace genesis keys, change protocol lineage, or make
otherwise unverifiable material authoritative.

The build and embedded material are authenticated by normal installer
integrity controls, but installer signing does not replace protocol threshold
signatures or genesis continuity.

Bootstrap endpoints are discovery-only. They cannot authorize protocol keys,
normal mix routes, PIR parameters, corpus state, or client compatibility.
Pinned bootstrap transport keys are evaluated without wall-clock certificate
validity so that TLS time validation cannot create a circular dependency.

An embedded endpoint may direct the client only to an artifact that validates
from the immutable genesis or previously activated retained trust state. If all
acceptable endpoints are unreachable, bootstrap stops until connectivity is
restored or an admitted build supplies a newer continuity-proven cache.

## 4. Signed artifact graph

Artifacts use deterministic encoding and domain-separated signatures. Each
artifact contains a schema identifier, monotonically increasing sequence,
content hash, issuer policy, and transparency-log coordinates.

The dependency graph is:

1. `GenesisTrustRoot` authenticates the initial governance, log, witness, time,
   and bootstrap verification keys and the protocol lineage.
2. A consistency proof connects the retained checkpoint to a newer signed tree
   head without using current time.
3. Witness attestations authenticate a consistent view of that tree head.
4. Time attestations bind a new client nonce to the witnessed tree head and a
   bounded time interval.
5. Fresh authenticated time permits evaluation of artifact activation and
   expiry.
6. The signed client policy and compatibility tuple authorize normal protocol
   participation.

No child artifact may introduce a new verification key unless the applicable
critical 4-of-5 transition is included in the verified log history.

## 5. Cold-start state machine

### 5.1 Load local state

Load the embedded bundle and the last committed `TrustedBootstrapState`, if
present. The local state contains:

- Highest trust-root, authority-set, directory, and compatibility sequences.
- Highest accepted log tree size and root hash.
- Highest accepted epoch and authenticated-time interval.
- Highest accepted sequence for each time attestor.
- Active protocol and corpus tuple identifiers.
- Local rollback-assurance class and primitive-specific state.

Reject malformed state. A `strong` configuration performs its qualifying
platform restoration check before using local state. A `weaker` configuration
does not trust local ordering until it has re-established current witnessed
state over the network.

### 5.2 Establish transparency consistency

All verification keys, quorum rules, endpoint-selection rules, uncertainty
limits, and freshness bounds used before fresh time comes only from the
immutable genesis state or the last fully activated and committed state. New
trust sets, directories, and epoch policies found in the log are staged. They
cannot verify bootstrap or help establish the time used to activate themselves.
Loss of the previously active quorum fails closed.

Use only genesis or previously activated generic bootstrap endpoints to obtain:

- A current signed tree head.
- An inclusion proof for each required bootstrap artifact.
- A consistency proof from the highest trusted checkpoint.
- Witness attestations over the same tree size, root hash, and log identity.

Require matching attestations from at least two independent witnesses. Reject
decreasing tree sizes, invalid consistency or inclusion proofs, mismatched
roots, unknown keys, and unlogged key transitions.

A new installation validates from the embedded checkpoint. A returning
`strong` client validates from its protected local checkpoint. A returning
`weaker` client validates from the locally stored checkpoint but claims
rollback detection only after the current witnessed history has been
re-established.

Witness propagation bounds are uniform parameters of the previously activated
bootstrap policy. When any accepted witness reports a newer tree head than the
candidate, the client obtains and verifies consistency to that head before
continuing. Inability to converge on one checkpoint within the authorized
propagation bound fails closed.

### 5.3 Establish fresh authenticated time

Generate a new 256-bit nonce from the operating system cryptographic random
source for every bootstrap attempt. The nonce is never reused or derived from
account state.

Send the same generic fixed-shape request to each configured time attestor. A
valid `TimeAttestation` binds:

- The complete client nonce.
- A lower and upper UTC bound.
- The attestor identity and attestation sequence.
- The verified transparency tree size and root hash.
- The immutable trust-root lineage and global protocol/profile identity.
- The epoch-policy identifier.
- A short signed validity bound defined by that epoch policy.

Accept time only from at least two independent attestors whose intervals
overlap, reference the verified checkpoint, use the current nonce, and produce
an intersection no wider than the globally authorized uncertainty limit.

Each honest time attestor retains its own monotonically increasing witnessed
checkpoint floor and refuses to attest a lower tree size or inconsistent root.
Before issuing attestations for a new validity window, it generates its own
unpredictable refresh nonce and obtains live responses from the active witness
quorum. Each witness response binds that nonce, its current signed tree head,
and its monotonic observation sequence after refreshing its log view. The time
attestor verifies matching heads and refuses service if it cannot obtain this
replay-resistant refresh within the active propagation bound.

The resulting `TimeAttestation` binds a digest of this witness-refresh evidence
and its observation interval. An auditor can therefore distinguish a current
witness refresh from reuse of an older signed tree head. Suppressing the log or
witness updates causes time-attestation failure rather than authorization of
stale state.

If an attestor reports a newer checkpoint, the client returns to Section 5.2,
proves consistency to that checkpoint, obtains its witness quorum, and issues a
new nonce. Fresh time is accepted only when the attestor quorum binds the same
checkpoint and protocol lineage. The previously activated bootstrap policy
defines a bounded log, witness, and time-attestor propagation assumption;
failure to converge within it fails closed.

Recorded attestations cannot answer a new nonce. Responses for a previous
nonce, a different tree head, trust lineage, protocol/profile, a decreasing
attestor sequence, or a disjoint time interval are invalid.

After acceptance, use the monotonic process clock to advance the authenticated
interval. Wall-clock changes cannot advance, reverse, or extend it.

### 5.4 Validate activation and compatibility

Using fresh authenticated time, validate:

- Artifact activation epochs and expirations.
- Authority, witness, time-attestor, and directory transitions.
- The client compatibility policy locally.
- The complete compatibility tuple:

  `client compatibility policy x protocol manifest x corpus release x PIR parameters x cryptographic suite x epoch policy`

- The next globally valid participation boundary.

Exact build, version, platform, architecture, assurance class, policy branch,
and capabilities are never sent to bootstrap or normal infrastructure.

Routine 3-of-5 policy may admit a build only when all critical-authorized
observable-behavior identifiers remain unchanged. Any change to wire, traffic,
timing, routing, reply, cryptographic, privacy, overload, or telemetry behavior
requires a critical 4-of-5 action.

### 5.5 Commit and wait for an epoch

Atomically commit the new trusted state before normal participation. A strong
configuration commits through its qualifying restoration-detection or
monotonic platform primitive. A weaker configuration commits encrypted state
without claiming pre-network restoration detection.

Wait for the next global epoch boundary. Never begin early, replay missed
slots, or emit compensating traffic.

## 6. Bootstrap trace non-interference

For identical public trust state and modeled network events, two clients with
arbitrarily different accounts, queues, runs, corpus interests, and future work
produce the same distribution of protocol-controlled bootstrap:

- Endpoints and request selection.
- Request count and logical sizes.
- Initiation and timing distribution.
- Timeout, retry, and recovery decisions.
- Visible errors and termination conditions.

Endpoint order, artifact-fetch set, padding, timing, retries, and
protocol-controlled transport settings are functions only of the globally
authorized bootstrap profile. Embedded endpoint changes are critical profile
transitions and cannot vary by build within one admitted profile.

Bootstrap uses no client authentication, cookies, persistent resumption token,
or other stable identifier across sessions. The fresh nonce is uniformly
random, scoped to one bootstrap attempt, and not reused as a normal-protocol
identity. Build-specific content negotiation and protocol branches are
prohibited.

IP address, transport fingerprint, OS network-stack behavior, connectivity,
and bootstrap start/stop timing remain observable leakage and are evaluated
separately from active-cover activity unobservability. Protocol-controlled
settings must minimize these fingerprints where practical; residual platform
and network effects remain explicit measurements rather than assumed parity.

## 7. Directory and key evolution

Bootstrap directory updates require critical 4-of-5 authorization, witnessed
log inclusion, a strictly increasing sequence, and a predecessor hash.
Directory validity for discovery is based on this ordered history rather than
unestablished wall-clock time. Activation for normal protocol use is evaluated
only after fresh time exists.

Planned authority-key rotation requires the old critical 4-of-5 authorization
and the new key's required proof of possession. Emergency revocation and
replacement require the critical 4-of-5 path. When quorum is unavailable,
there is no unsigned recovery or exception path.

Forked directories, inconsistent authority histories, missing witnesses, or
inability to prove continuity stop bootstrap.

## 8. Clock discontinuity and restart

The globally signed epoch policy defines maximum time uncertainty and clock
discontinuity. These values are uniform and cannot be tuned per client.

Wall-clock rollback, a monotonic-clock reset, reboot, suspension beyond the
authenticated interval, contradictory attestations, or excessive uncertainty
invalidates current time. The client returns to bootstrap, creates a new nonce,
and repeats the trust sequence.

Previously valid time material cannot extend an expired manifest or activate a
future one. The highest accepted epoch and artifact sequences never decrease
within the applicable assurance model.

## 9. Platform rollback interface

Implementation must expose a narrow platform adapter with these semantic
operations:

- Determine whether a qualifying primitive is available under the current
  hardware, OS, privilege, and virtualization configuration.
- Detect restoration before normal network participation.
- Read or validate primitive-specific monotonic state when applicable.
- Atomically bind a committed bootstrap-state digest to the primitive.
- Distinguish reset, replacement, corruption, permission loss, and ordinary
  first use.

Only configurations demonstrated to meet all required semantics are `strong`.
All others are `weaker` and must re-establish rollback detection through fresh
time and witnessed transparency before participation. Hardware-backed key
storage by itself does not qualify.

The platform capability matrix is evidence, not configuration supplied by the
client to the network. Assurance class never changes protocol-visible behavior.

## 10. Failure behavior

Any invalid signature, proof, witness quorum, nonce, time interval, sequence,
directory, compatibility tuple, or platform restoration state stops bootstrap.

Failures never:

- Carry private work into bootstrap.
- Select different endpoints because work is pending.
- Change the future cover profile according to queue state.
- Permit a stale, downgraded, forked, expired, or partially authorized tuple.
- Activate normal PIR, mixnet, Instagram, or other fallback traffic.
- Cause catch-up traffic after recovery.

Availability is restored only by completing the same bootstrap sequence and
waiting for a valid future epoch.

## 11. Platform capability baseline

No supported desktop OS receives a blanket `strong` classification. Strong
status is granted only to an enumerated, tested configuration.

| Configuration | Phase 1 baseline | Required evidence for `strong` |
| --- | --- | --- |
| Windows with physical TPM 2.0 | Candidate only | Demonstrate that the supported Windows/device configuration permits safe TPM NV provisioning, read, and update before networking; handles command blocking, authorization, atomicity, TPM clear, and replacement fail-closed. |
| macOS with Apple silicon, T2, or Touch ID hardware | `weaker` for Phase 1 | Only an existing documented platform primitive demonstrated to provide the required app-state restoration-detection semantics can change this classification within the frozen architecture. Secure Enclave or Keychain key storage alone does not qualify; a third-party alternative requires formal architecture reopening. |
| Linux with physical TPM 2.0 | Candidate only | Enumerate supported distro, device permissions, hierarchy ownership, provisioning, firmware, atomicity, TPM clear, and replacement behavior; demonstrate that disk restoration does not restore the TPM state. |
| Virtual machine with vTPM only | `weaker` | A vTPM may be restored with the VM snapshot and is not an external rollback anchor. |
| Virtual machine exposing VM Generation ID | Candidate only | Demonstrate that the guest reads and binds the generation identifier before networking and that every supported snapshot, restore, clone, import, and migration path changes it. |
| TPM clear or hardware replacement | Safety failure | Stop and rebootstrap; key loss is disruption evidence, not proof of app-state rollback. |

The implementation must test clean boot, crashes between primitive and database
commit, OS upgrade, backup restoration, cloning, TPM reset, hardware
replacement, VM snapshot, import/export, migration, and nested virtualization.
An administrator controlling the TPM or hypervisor is outside a strong claim
unless separately included in the documented platform assumptions.

Official capability references:

- Microsoft: [TPM Base Services](https://learn.microsoft.com/en-us/windows/win32/tbs/tpm-base-services-portal),
  [`Tbsip_Submit_Command`](https://learn.microsoft.com/en-us/windows/win32/api/tbs/nf-tbs-tbsip_submit_command),
  [TPM command blocking](https://learn.microsoft.com/en-us/windows/win32/tbs/command-blocking),
  and [TPM fundamentals](https://learn.microsoft.com/en-us/windows/security/hardware-security/tpm/tpm-fundamentals).
- Apple: [Protecting keys with the Secure Enclave](https://developer.apple.com/documentation/security/protecting-keys-with-the-secure-enclave)
  and [Secure Enclave](https://support.apple.com/guide/security/secure-enclave-sec59b0b31ff/web).
- Linux kernel: [TPM documentation](https://docs.kernel.org/security/tpm/).
- `tpm2-tools`: [`tpm2_nvdefine`](https://tpm2-tools.readthedocs.io/en/latest/man/tpm2_nvdefine.1/),
  [`tpm2_nvincrement`](https://tpm2-tools.readthedocs.io/en/latest/man/tpm2_nvincrement.1/),
  and [`tpm2_clear`](https://tpm2-tools.readthedocs.io/en/latest/man/tpm2_clear.1/).
- Microsoft: [Virtual Machine Generation ID specification](https://go.microsoft.com/fwlink/?LinkId=260709).
- QEMU: [TPM device](https://www.qemu.org/docs/master/specs/tpm.html)
  and [VM Generation ID](https://www.qemu.org/docs/master/specs/vmgenid.html).
- libvirt: [Domain XML `genid` and TPM device](https://libvirt.org/formatdomain.html).

These references establish candidate capabilities, not application-level proof.
Runtime validation is mandatory before assigning `strong`.

## 12. Required test vectors and evidence

The implementation specification must provide deterministic fixtures for:

- Valid and invalid threshold signature domains and quorums.
- Transparency inclusion, consistency, fork, and split-view cases.
- Witness agreement, minority compromise, and quorum compromise.
- Current, replayed, wrong-nonce, wrong-checkpoint, disjoint, and excessively
  wide time attestations.
- An old witnessed checkpoint presented with a fresh nonce, a time attestor
  reporting a newer checkpoint, and a future trust set attempting to validate
  its own activation.
- One compromised time attestor paired with an honest attestor whose log or
  witness refresh traffic is suppressed; the honest attestor must refuse.
- Wrong-protocol, wrong-profile, and wrong-trust-lineage time attestations.
- Directory rotation, authority rotation, revocation, downgrade, and sequence
  rollback.
- Routine-compatible and critical-only client changes.
- Strong and weaker rollback transitions, reset, clone, restoration, and crash
  recovery.
- Bootstrap traces for different private states and compatible client builds.
- Sleep, restart, network loss, stale state, and epoch-boundary recovery.

The platform capability matrix must cite authoritative platform evidence and
runtime validation for every strong classification. Missing or ambiguous
evidence results in the weaker classification.

## 13. Exit criteria

This specification is ready for implementation only when:

- The artifact schemas and domain-separation rules are unambiguous.
- Threshold and witness independence assumptions are documented.
- The trust graph has no time or key-validation cycle.
- Freshness requires a new unpredictable nonce.
- Cross-build and private-state bootstrap traces are testably equivalent.
- Every platform configuration has an evidence-backed assurance class.
- Failure transitions preserve safety over liveness.
- An independent security review finds no unresolved high-severity issue.

This document does not select PIR parameters, mixnet topology, cover rate, or
traffic-analysis thresholds. Those decisions remain in their locked derived
phases.
