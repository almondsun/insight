# Fame Traffic-Analysis Preregistration

Status: Phase 1 preregistration protocol; decisive values not yet frozen

Depends on:

- [Fame Privacy Threat Model](THREAT_MODEL.md)
- [Fame Trust-Bootstrap Specification](TRUST_BOOTSTRAP.md)
- [Fame PIR Benchmark Protocol](PIR_BENCHMARK_PROTOCOL.md)
- [Fame Mixnet Experiment Specification](MIXNET_EXPERIMENT_SPECIFICATION.md)

Architecture status: frozen

## 1. Purpose

This document defines the preregistration process for decisive traffic-analysis
evaluation. It prevents thresholds, adversaries, features, classifiers, and
datasets from being changed after outcomes are known.

No production privacy claim follows from this template. Before decisive capture,
the project must fill every field marked `PREREGISTER`, publish a content hash
and timestamped immutable copy, and obtain independent auditor acceptance.

## 2. Primary hypotheses

### H1: Activity unobservability

During an active participation interval, a covered observer receives a trace
from one of two equally likely worlds:

- `DUMMY`: all eligible slots contain valid dummy PIR operations.
- `REAL`: private operations replace a preregistered subset of dummy slots.

One atomic H1 trial is a complete preregistered active-session window. The
challenger fixes the signed profile, public workload envelope, supported
environment, observer projection, and modeled network-condition distribution;
samples a uniform secret bit; and generates either `DUMMY` or `REAL`. In
`REAL`, slot substitution locations and workloads come from a preregistered
distribution. Both worlds have matched public inputs and scheduled slot counts.

The adversary receives only the covered observer projection plus explicitly
preregistered public auxiliary information and training data. It operates under
the frozen feature, model, compute, query, and hyperparameter-search budget and
outputs one bit per independent trial. The primary score is:

`Adv_activity = abs(2 * accuracy - 1)`

The system passes only if the upper confidence bound of the maximum observed
advantage across every preregistered covered observer, classifier, workload,
and supported environment is no greater than `PREREGISTER: epsilon_activity`.

### H2: Sender-query unlinkability

One atomic H2 trial contains at least two simultaneously active senders and a
fixed public multiset of requests and replies. The adversary nominates two valid
sender-operation assignments with identical public inputs. The challenger
samples a uniform secret bit, executes that assignment under the preregistered
network-condition distribution, and reveals only the covered observer
projection and allowed auxiliary information. The adversary outputs its guess.

Reduce the primary game to balanced pairwise challenges so chance accuracy is
one half and define:

`Adv_link = abs(2 * accuracy - 1)`

The system passes only if the upper confidence bound of the maximum observed
advantage across all covered observer, request-path, reply-path, end-to-end,
and correlated-compromise conditions is no greater than
`PREREGISTER: epsilon_link`.

The H2 assignment, all paired request/reply observations, and their failure
outcomes remain one indivisible trial and partition unit.

These are empirical bounds against the preregistered attacker family and
resource budget. They are not universal upper bounds over all algorithms. Any
broader protocol claim separately requires the independently reviewed
exact-profile mixnet security and composition memo.

Cryptographic query-index privacy is not inferred from traffic traces. It is
evaluated by the PIR proof and conformance process.

## 3. Evaluation boundaries

Evaluate these boundaries separately:

- Cold-start bootstrap leakage.
- Transition into active participation.
- Steady active participation.
- Short connectivity disruption.
- Prolonged suspension and shutdown.
- Re-entry at a valid epoch boundary.

Only steady active participation is covered by H1. Boundary results cannot be
averaged into or used to strengthen the active-cover claim.

Bootstrap analysis asks whether protocol-controlled traces depend on private
purpose or compatible build, while acknowledging that participation attempts,
IP address, transport stack, and start time may remain observable.

Mandatory deterministic non-interference gates compare:

- Bootstrap traces across arbitrary private states and future workloads.
- Repeated bootstrap sessions for stable identifiers or resumption state.
- Every compatible build under identical public state, seeds, and injected
  events.
- Waiting, activation, stop, suspension, and re-entry transitions across
  private states.

Any protocol-controlled difference fails the applicable gate. Unavoidable
bootstrap network leakage is reported separately and never combined with H1.
Reuse or exposure of any stable identifier, credential, resumption handle, or
linkable identifier-derived value across bootstrap sessions automatically
produces `FAIL_BOOTSTRAP_NONINTERFERENCE`, even when it is independent of
private purpose.

## 4. Preregistration record

Before decisive capture, freeze:

- Git commits, source manifests, SBOMs, binary digests, and build images.
- PIR implementation, parameters, corpus digest, and replica deployment.
- Mixnet implementation, topology, directory, packet geometry, reply scheme,
  and all global profile parameters.
- Compatible client builds and supported platform/network configurations.
- Active participant population and synthetic workload generation.
- Covered observer views and correlated compromise domains.
- Trace fields, capture locations, clocks, synchronization, and retention.
- Dataset sizes derived by power analysis.
- Train, validation, and sealed test partitions.
- Feature extraction and normalization.
- Classifier implementations and hyperparameters.
- Random seeds and seed-commitment procedure.
- Statistical tests, confidence procedures, multiple-testing control, and
  evidence-derived thresholds.
- Missing-data, failed-run, outage, and outlier treatment.
- Positive and negative controls.
- The complete covered-condition matrix and longitudinal games.

The immutable preregistration receives a transparency-log entry. Any amendment
creates a new version and invalidates unevaluated or newly collected decisive
data from the old version.

## 5. Dataset construction

Use only synthetic PIR records and randomly generated, non-Instagram account
labels. No real archive, username, social graph, or Fame result enters the
dataset.

Generate paired worlds from a coordinator seed committed before collection and
revealed only after traces are sealed. Infrastructure operators receive only
the inputs needed for their role and do not receive world labels before seal.

Collect:

- Equal numbers of H1 `DUMMY` and `REAL` sessions.
- Balanced H2 sender-query pairings.
- Request-only, reply-only, and end-to-end traces.
- Empty, intermittent, and backlogged private workloads under the same cover
  profile.
- Hits, safe misses, repeated points, and unique points.
- Normal, overload, delayed, missing-reply, relay-churn, and partition cases.
- Each compatible build and supported platform/network configuration.
- Long-duration sessions sufficient for the preregistered intersection and
  disclosure horizon.
- Replay, duplication, tagging/watermarking, selective delay/drop, n-1,
  flooding, malicious gateway/service, SURB replay, route capture, and colluding
  request/reply observation conditions within the covered adversary model.

Freeze a condition matrix whose rows identify observer views, compromised
domains, active capability and budget, request/reply visibility, workload,
failure mode, supported environment, and observation horizon. Every covered row
enters the primary maximum or a separately named mandatory gate.

Preregister longitudinal sender-linkage and disclosure games. Each specifies
the sender population, observation windows, churn, auxiliary knowledge,
assignment challenge, output, advantage, and evidence-derived threshold.

Determine sample counts from a one-sided confidence-width analysis with
preregistered safe and unsafe margins, familywise false-pass probability at or
beyond the unsafe margin, cluster structure, and minimum independent cluster
count. The probability of achieving `PASS` at the specified safe alternative
must be at least `PREREGISTER: statistical power`. Positive-control
leak-detection power is calculated separately.

## 6. Observer views

Preregister each covered view and combination:

- Client access network.
- Entry gateway.
- Each mix layer.
- Correlated relay/operator domains.
- PIR ingress and egress.
- Reply gateway or mailbox.
- Partial ingress-plus-egress coverage.
- Hosting, autonomous-system, or shared-control-plane vantage points.
- Any bounded multi-vantage observer admitted by the threat model.

A comprehensive global passive observer remains outside the guarantee. It may
be evaluated only as failure characterization, visibly labeled as an excluded
adversary.

## 7. Trace schema and feature policy

The preregistered trace schema may include observer-visible:

- Relative packet time and direction.
- Observer-visible packet or record length.
- Connection and flow boundaries visible at that vantage point.
- Loss, duplication, retransmission, and ordering.
- Route-domain labels available to the specified observer.
- Public epoch, relay, overload, and network-condition state.

It must not include plaintext usernames, PIR indices, world labels as model
features, private queue state, run state, decrypted payloads unavailable to the
observer, or stable identifiers created solely for the experiment.

Feature extraction is deterministic, versioned, and fitted only on training
data. Preregister aggregation windows, sequence length, padding, missing-value
handling, normalization, dimensionality reduction, and categorical encoding.

No feature may be added after viewing sealed-test performance. Exploratory
features require a new preregistration and new decisive dataset.

## 8. Classifier suite

Preregister exact implementations and hyperparameters for at least:

- Regularized logistic regression over aggregate trace features.
- A fixed gradient-boosted decision-tree model over the same features.
- A fixed sequence model over direction/time/length sequences.
- A nearest-neighbor or template-matching traffic-analysis baseline.

Hyperparameter selection uses only training and validation partitions under a
fixed search budget. The sealed test set is evaluated once per preregistration.

The primary result is the maximum covered distinguishing advantage, not the
result of a preferred classifier. Diagnostics include ROC AUC, balanced
accuracy, TPR/FPR, calibration, confusion matrices, and feature importance.

## 9. Partitioning and leakage prevention

Atomic trials and samples sharing stochastic lineage, session identity,
mutable state, incidents, reply pairs, or preprocessing caches cannot cross
train and test partitions. Fixed route, operator, and topology factors are
preregistered condition strata and clustering dimensions; use dedicated
held-out deployments only where the generalization claim requires them.

Before collection, assign independent capture blocks, clients, sessions,
hierarchical seeds, paired challenges, time windows, and preprocessing caches
to exactly one partition. H1 and H2 atomic trials cannot be split.

Within each capture block, build a correlation graph connecting samples that
share stochastic lineage, session identity, paired trial, reply pair, injected
incident, or mutable cache. Assign each whole component to its block's
preassigned partition. Common fixed infrastructure, operator, or topology is a
condition stratum and cluster-aware inference dimension rather than an edge
that collapses all partitions. Use dedicated held-out deployments when the
preregistered generalization claim requires them.

If an unplanned incident or dependency discovered after capture bridges
partitions, invalidate the affected data under the missing-data policy. Do not
reassign it after labels or results are available.

Use partition-scoped hierarchical seeds. Commit them before collection and
reveal them only after sealing. Randomize and block capture order. Audit
filenames, run identifiers, coordinator metadata, wall times, manifests, and
caches for direct or indirect label leakage.

Include preregistered held-out conditions for:

- Later collection time.
- Unseen compatible build.
- Unseen route combination.
- Unseen supported network condition.

No packet, derived window, retry sequence, reply pair, or shared random seed may
straddle partitions.

Operators, model developers, and auditors do not receive sealed-test labels
until the pipeline and output commitments are fixed.

## 10. Statistical analysis

Preregister:

- Significance level: `PREREGISTER`.
- Confidence level: `PREREGISTER`.
- Statistical power: `PREREGISTER`.
- Cluster unit for uncertainty estimation: `PREREGISTER`.
- Confidence method: `PREREGISTER`.
- Multiple-comparison correction: `PREREGISTER`.
- Safe and unsafe advantage margins: `PREREGISTER`.
- Minimum independent clusters per covered condition: `PREREGISTER`.
- `epsilon_activity`: `PREREGISTER from threat and feasibility evidence`.
- `epsilon_link`: `PREREGISTER from threat and feasibility evidence`.

Use a one-sided non-inferiority formulation whose unsafe null is that advantage
is at least the preregistered privacy threshold. Control the preregistered
false-pass rate across the complete classifier/condition family with one
simultaneous upper-confidence procedure, such as a frozen cluster max-statistic
bootstrap or another method justified before capture. Record trial weighting,
cluster weighting, model selection, and multiplicity handling explicitly.

Confidence bounds must respect session, route, client, time, and operator
clustering. Ordinary packet-level bootstrap resampling is prohibited when it
treats dependent packets as independent.

Passing requires the preregistered upper confidence bound for the maximum
covered advantage to remain at or below its threshold. Failure of one covered
condition is not averaged away by stronger results elsewhere.

## 11. Controls and validity checks

Use:

- A negative control with labels permuted after grouping.
- A positive control containing a deliberately documented protocol-visible
  difference calibrated near the minimum relevant advantage for each trace
  modality, isolated from production code.
- Deterministic replay to verify feature and model reproducibility.
- Duplicate-data and partition-leakage detection.
- Clock-offset and capture-loss sensitivity analysis.
- Label-blind pipeline execution before unsealing.

The negative control should remain at chance within its preregistered bound.
The positive control must be detected at its preregistered minimum sensitivity;
otherwise the evaluation is `INVALID_EVALUATION`. A negative-control failure is
also `INVALID_EVALUATION`.

## 12. Network-induced variation

Deterministic simulation first requires exact equality of every
protocol-controlled real/dummy trace under identical seeds and injected events.

Live analysis then models transport implementation, MTU behavior, congestion,
socket scheduling, retransmission, relay load, loss, and routing variation.
For every environment claimed as supported, H1 and H2 use the complete
observer-visible live trace including these effects. Feature-level exclusion is
prohibited. An environment may be excluded only before decisive capture, in
which case support and the public claim are narrowed accordingly.

## 13. Failure and missing-data policy

Preregister how to handle:

- Incomplete captures and clock desynchronization.
- Participant or relay dropout.
- Missing PIR replies and timeouts.
- Infrastructure incidents and profile violations.
- Corrupt traces and collection-tool failure.

Disposition is decided label-blind before unsealing. Observer-visible absence,
timeout, retransmission, and missingness remain features or outcomes rather
than exclusions. Exclusion is allowed only for preregistered acquisition
failures shown independent of the challenge world.

Freeze stopping and rerun rules before capture, publish all denominators, and
apply preregistered worst-case and sensitivity analyses for missing trials.
Unavailable required conditions produce `INVALID_EVALUATION` or
`FEASIBILITY_BLOCKED`, never `PASS`.

Do not discard a run because classification is unexpectedly successful or
because a privacy-relevant failure occurred. Protocol violations receive their
own labeled analysis and remain in the applicable primary maximum.

## 14. Preregistration sealing chronology

Create one canonical root manifest using the hash and signature algorithms in
the active critically authorized suite. It hashes every source and binary,
container image, global profile, topology, condition matrix, workload and label
generator, seed commitment, capture schema, partition assignment, feature and
model implementation, hyperparameter budget, statistical procedure, threshold,
and report template.

The critical governance quorum signs that manifest. Record it in the witnessed
transparency log and record independent auditor acceptance before decisive
capture. After capture and before label reveal, publish immutable raw-capture
and metadata hashes linked to the same root.

Only then reveal coordinator seeds and labels and run the sealed analysis.
Amended preregistrations require new data; captures and results bound to an old
root cannot be reused for a new decisive evaluation.

## 15. Reproducibility and independent audit

Publish or provide to the independent auditor:

- Preregistration and transparency proof.
- Pinned source, SBOMs, build provenance, and binary digests.
- Synthetic workload generator and committed/revealed seeds.
- Capture and sanitization pipeline.
- Immutable raw-trace hashes and access procedure.
- Feature, training, evaluation, and reporting code.
- Machine-readable results and environment manifests.

The auditor independently rebuilds the tools, verifies dataset partitioning,
reruns the complete sealed evaluation, and confirms that the report includes
all preregistered covered conditions.

The auditor also rebuilds the client, relays, PIR services, capture tools, and
analysis images from pinned sources; verifies raw-capture provenance and seals;
and independently reproduces a preregistered end-to-end capture subset using
fresh committed seeds.

## 16. Decision outcomes

- `PASS`: H1, H2, every bootstrap/profile/transition deterministic gate, every
  covered condition-matrix and longitudinal gate, sealing and missing-data
  validity, positive and negative controls, the accepted exact-profile mixnet
  security memo, and independent reproduction all pass.
- `FAIL_ACTIVITY`: H1 exceeds its preregistered bound.
- `FAIL_LINKABILITY`: H2 exceeds its preregistered bound.
- `FAIL_BOOTSTRAP_NONINTERFERENCE`: bootstrap depends on private purpose or
  future work.
- `FAIL_PROFILE_EQUIVALENCE`: compatible builds or participation transitions
  differ in protocol-controlled behavior.
- `FAIL_LONGITUDINAL`: a required linkage or disclosure gate fails.
- `INVALID_EVALUATION`: preregistration, capture, partition, control, or
  reproducibility requirements fail.
- `FEASIBILITY_BLOCKED`: required PIR, mixnet, population, or capacity evidence
  is unavailable.

Only `PASS` permits progression toward the integrated experimental deployment.
Any changed profile, implementation, topology, threshold, observer model, or
material operating condition requires a new preregistration and decisive
dataset.
