# PIR Feasibility Log

Status: upstream PIR tests pass in a pinned container; no benchmark or security
approval result

Protocol: [PIR Benchmark Protocol](PIR_BENCHMARK_PROTOCOL.md)

## Google DPF/PIR inspection — 2026-08-08

### Source identity

- Repository: <https://github.com/google/distributed_point_functions>
- Commit: `859cafa71fc1e139c7b76d4d4c0f23438688a8ad`
- Git description: `v0.0.0-10-g859cafa`
- Commit date: 2026-01-05
- License: Apache-2.0
- Checkout location: isolated temporary directory outside the insIGht tree
- Upstream status: explicitly unsupported and provided without correctness or
  security guarantees

The checkout contains dense, simple-hashed, and cuckoo-hashed DPF-PIR clients,
databases, two-server request handlers, tests, and microbenchmarks. This corrects
the earlier assumption that the repository contained only DPF primitives.

### Build attempts

#### Bazel 9.2.0

Command shape:

```text
bazelisk test //pir/...
```

Result: build-graph failure before compilation or tests.

- `cc_proto_library` was undefined in the upstream BUILD files.
- Resolved `rules_go` referenced a removed `CcInfo` symbol.
- Bazel rewrote the upstream `MODULE.bazel.lock` from lock format 16 to 28 and
  resolved newer transitive module versions.

This attempt demonstrates that the repository does not pin a Bazel version
compatible with its committed lock file. The temporary checkout was restored
before the next attempt.

#### Bazel 8.4.2 with GCC 16.1.1

Command shape:

```text
USE_BAZEL_VERSION=8.4.2 bazelisk test -c opt \
  --test_tag_filters=-benchmark //pir/...
```

Result: dependency compilation failure; zero tests executed.

The pinned upstream BoringSSL dependency failed under GCC 16 because its
`OPENSSL_memchr` implementation discarded a `const` qualifier and the build
treated the diagnostic as an error.

#### Bazel 8.4.2 with Clang 22.1.8

The same target was rerun with repository and action toolchain variables pinned
to the installed Clang toolchain.

Result: the identical BoringSSL const-qualifier defect failed compilation; zero
tests executed.

#### Bazel 8.4.2 with GCC 14 container

The exact source commit was cloned into a fresh, disposable, non-root container:

- image tag: `gcc:14-bookworm`
- image digest:
  `sha256:5e927c284bf55a7dc796262e311a0703344f62f41f5621eb56843111b1d37e15`
- Bazel: 8.4.2 via Bazelisk 1.27.0
- configuration: optimized build, benchmark-tagged tests excluded
- source modifications: none

Command shape:

```text
docker run --rm --user 1000:1000 -e HOME=/tmp/home -e USER=builder \
  -v <pinned-bazelisk>:/usr/local/bin/bazelisk:ro gcc:14-bookworm \
  bash -lc '<clone exact commit; USE_BAZEL_VERSION=8.4.2 \
    bazelisk test -c opt --test_tag_filters=-benchmark //pir/...>'
```

Result: successful build and test run.

- 46 targets found
- 24 test targets executed
- 24 tests passed; zero failed
- 2,699 build actions
- 547.779 seconds elapsed

The compiler emitted numerous warnings, including signedness comparisons,
control reaching the end of non-void functions, and a suspicious
`sizeof(bool*) / sizeof(bool)` expression. A test pass does not dispose of those
review items.

### Interpretation

The pinned container result supplies reproducible functional evidence for the
upstream PIR test suite. The native GCC 16 and Clang 22 failures remain a host
portability blocker. No upstream source or dependency was patched because doing
so would invalidate the source-identity baseline and require a separately
reviewed patch series.

The upstream CI workflow runs `bazel build` and `bazel test` but does not pin or
install an exact Bazel version in the workflow. The container above establishes
a candidate baseline; a qualifying benchmark still requires a hermetic build
definition, a recorded compiler and libc identity, and independent binary
reproduction.

The candidate remains unapproved. The upstream tests do not establish the
complete-view privacy property, response-context binding, dummy equivalence,
independent-operator behavior, workload capacity, or production suitability.
Those questions remain gated by the benchmark protocol and an independent
cryptographic review.
