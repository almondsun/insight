# Fame Privacy Engineering

Status: architecture frozen; feasibility not established

Fame is a proposed optional enrichment feature. Its versioned scoring core and
database foundation are implemented, but no Fame retrieval or user-facing run
workflow exists and the current insIGht application remains offline. These
documents define gates that must pass before any Fame networking can be
considered for experimental use.

## Locked Phase-1 artifacts

1. [Formal threat model](THREAT_MODEL.md)
2. [Trust-bootstrap specification](TRUST_BOOTSTRAP.md)
3. [PIR benchmark protocol](PIR_BENCHMARK_PROTOCOL.md)
4. [Mixnet experiment specification](MIXNET_EXPERIMENT_SPECIFICATION.md)
5. [Traffic-analysis preregistration](TRAFFIC_ANALYSIS_PREREGISTRATION.md)

Product behavior:

- [Fame product contract](PRODUCT_CONTRACT.md)
- [Implementation status](IMPLEMENTATION_STATUS.md)

Feasibility evidence:

- [PIR feasibility log](PIR_FEASIBILITY_LOG.md)

The architecture requires a query-independent licensed corpus, two
independently operated PIR replicas, separately governed mixed request and
reply paths, globally uniform cover traffic, witnessed transparency, and
fail-closed clients. It does not permit direct Instagram requests or fallback
transport.

No open-source PIR or mixnet component is approved by these documents. Candidate
repositories remain benchmark inputs until their exact versions,
configurations, integrations, and deployments pass the specified evidence and
independent-audit gates.

The architecture may be reopened only when a derived phase, implementation
evidence, or an independent audit demonstrates that a frozen assumption is
infeasible or insufficient. A theoretically stronger mechanism alone is not a
reason to change it.
