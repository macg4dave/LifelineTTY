#!/usr/bin/env bash
set -euo pipefail

# Regression test: Dockerfiles used by release tooling must expose an 'artifact'
# stage and local-release.sh must build that stage explicitly.

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

assert_contains() {
    local needle="$1"
    local file="$2"
    if ! grep -Fq -- "$needle" "$file"; then
        echo "FAIL: expected '$needle' in $file" >&2
        exit 2
    fi
}

assert_not_contains() {
    local needle="$1"
    local file="$2"
    if grep -Fq -- "$needle" "$file"; then
        echo "FAIL: did not expect '$needle' in $file" >&2
        exit 2
    fi
}

# Dockerfiles must have an artifact stage.
assert_contains " AS artifact" "$ROOT/docker/Dockerfile.armv6"
assert_contains " AS artifact" "$ROOT/docker/Dockerfile.armv7"
assert_contains " AS artifact" "$ROOT/docker/Dockerfile.arm64"
assert_contains " AS artifact" "$ROOT/docker/Dockerfile.amd64"

# ARMv6 Dockerfile should not require fetching toolchains from musl.cc.
assert_not_contains "https://musl.cc" "$ROOT/docker/Dockerfile.armv6"
assert_not_contains "curl -L \"https://musl.cc" "$ROOT/docker/Dockerfile.armv6"

# Release script should build the artifact stage explicitly.
assert_contains "--target artifact" "$ROOT/scripts/local-release.sh"

echo "docker artifact stage tests OK"
