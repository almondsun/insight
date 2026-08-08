# Fame feasibility lab

This lab is a synthetic, non-production evidence harness. It never reads an
Instagram archive and contains no Instagram client, PIR transport, mixnet
transport, credential path, or fallback network path.

Run the deterministic local checks with:

```bash
cargo run --manifest-path tools/fame-lab/Cargo.toml -- status
cargo run --manifest-path tools/fame-lab/Cargo.toml -- corpus --input lab/fame/fixtures/corpus.json
cargo run --manifest-path tools/fame-lab/Cargo.toml -- scheduler --manifest lab/fame/fixtures/manifest.json --tuple lab/fame/fixtures/tuple.json --username synthetic_alice
```

Every external gate reports `BLOCKED` until evidence exists. A blocked result
must not be converted into a guessed parameter, simulated production pass, or
weaker protocol. Candidate source trees belong in an ignored, isolated work
directory and are never vendored into the desktop application.

Pinned candidate inputs and toolchains are in `toolchain.lock.json`. The Python
classifier smoke harness consumes only explicitly synthetic CSV traces. Its
`requirements.in` lists direct inputs but is not a reproducible lock; a hashed
transitive lock is a remaining prerequisite. The smoke harness is not the
preregistered evaluation and cannot produce a privacy pass. Final traffic
thresholds remain unset until they are evidence-derived and preregistered.

Reproduce the pinned upstream Google DPF/PIR functional test baseline with
`bash lab/fame/reproduce_dpf.sh`. The script verifies Bazelisk and source
identity and uses a digest-pinned GCC 14 container. A passing upstream suite is
functional evidence only; it does not approve the candidate for production.
Bazel and transitive dependencies are still fetched over the network and are
not fully hash-locked, so this is a pinned baseline rather than a hermetic
reproduction. Hermetic dependency capture remains a `BLOCKED` production gate.
