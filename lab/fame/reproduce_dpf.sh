#!/usr/bin/env bash
set -euo pipefail

readonly DPF_COMMIT="859cafa71fc1e139c7b76d4d4c0f23438688a8ad"
readonly BAZELISK_URL="https://github.com/bazelbuild/bazelisk/releases/download/v1.27.0/bazelisk-linux-amd64"
readonly BAZELISK_SHA256="e1508323f347ad1465a887bc5d2bfb91cffc232d11e8e997b623227c6b32fb76"
readonly GCC_IMAGE="gcc:14-bookworm@sha256:5e927c284bf55a7dc796262e311a0703344f62f41f5621eb56843111b1d37e15"

printf '%s\n' 'NOTICE: Bazel and transitive dependencies are network-fetched; this run is not hermetic.' >&2

lab_tmp="$(mktemp -d)"
trap 'rm -rf -- "$lab_tmp"' EXIT

curl --fail --location --proto '=https' --tlsv1.2 "$BAZELISK_URL" --output "$lab_tmp/bazelisk"
printf '%s  %s\n' "$BAZELISK_SHA256" "$lab_tmp/bazelisk" | sha256sum --check --status
chmod 0755 "$lab_tmp/bazelisk"

git clone --filter=blob:none https://github.com/google/distributed_point_functions.git "$lab_tmp/dpf"
git -C "$lab_tmp/dpf" checkout --detach "$DPF_COMMIT"
test "$(git -C "$lab_tmp/dpf" rev-parse HEAD)" = "$DPF_COMMIT"

docker run --rm --user "$(id -u):$(id -g)" \
  --env HOME=/tmp/home --env USER=builder \
  --volume "$lab_tmp/bazelisk:/usr/local/bin/bazelisk:ro" \
  --volume "$lab_tmp/dpf:/src" \
  --workdir /src "$GCC_IMAGE" \
  bash -lc 'USE_BAZEL_VERSION=8.4.2 bazelisk test -c opt --test_tag_filters=-benchmark //pir/...'
