#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Build release artifacts locally (binary + .deb + .rpm) and optionally upload them to
GitHub Releases. The version is read from Cargo.toml; tag defaults to "v<version>".

Usage: scripts/local-release.sh [--target <triple>] [--tag <git-tag>] [--upload]

  --target <triple>   Optional Rust target triple (default: host). Example:
                      armv7-unknown-linux-gnueabihf
  --tag <git-tag>     Override release tag (default: v<Cargo version>)
  --upload            Push artifacts to GitHub Releases using the GitHub CLI.
  -h, --help          Show this message.

Prereqs: cargo, cargo-deb, cargo-generate-rpm, python3, and optionally the gh CLI.
EOF
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Missing required tool: $1" >&2
        exit 1
    fi
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_TRIPLE=""
TAG_OVERRIDE=""
UPLOAD=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)
            TARGET_TRIPLE="${2:-}"
            shift 2
            ;;
        --tag)
            TAG_OVERRIDE="${2:-}"
            shift 2
            ;;
        --upload)
            UPLOAD=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage
            exit 1
            ;;
    esac
done

require_cmd cargo
require_cmd cargo-deb
require_cmd cargo-generate-rpm
require_cmd python3

CRATE_VERSION="$(
    cargo metadata --no-deps --format-version 1 |
        python3 - <<'PY'
import json, sys
meta = json.load(sys.stdin)
for pkg in meta.get("packages", []):
    if pkg.get("name") == "seriallcd":
        print(pkg.get("version"))
        sys.exit(0)
print("Failed to find seriallcd in cargo metadata", file=sys.stderr)
sys.exit(1)
PY
)"

if [[ -z "${CRATE_VERSION}" ]]; then
    echo "Could not determine crate version" >&2
    exit 1
fi

RELEASE_TAG="${TAG_OVERRIDE:-v${CRATE_VERSION}}"

target_dir="target"
build_args=(--release)
deb_args=(--no-build)
rpm_args=()
arch_label="$(uname -m)"

if [[ -n "${TARGET_TRIPLE}" ]]; then
    build_args+=(--target "${TARGET_TRIPLE}")
    deb_args+=(--target "${TARGET_TRIPLE}")
    rpm_args+=(--target "${TARGET_TRIPLE}")
    target_dir="target/${TARGET_TRIPLE}"
    arch_label="${TARGET_TRIPLE}"
fi

echo "Building seriallcd ${CRATE_VERSION} (${arch_label})..."
cargo build "${build_args[@]}"
cargo deb "${deb_args[@]}"
cargo generate-rpm "${rpm_args[@]}"

BIN_PATH="${ROOT}/${target_dir}/release/seriallcd"
DEB_DIR="${ROOT}/${target_dir}/debian"
RPM_DIR="${ROOT}/${target_dir}/generate-rpm"

if [[ ! -f "${BIN_PATH}" ]]; then
    echo "Binary not found at ${BIN_PATH}" >&2
    exit 1
fi

DEB_PATH="$(ls -t "${DEB_DIR}"/seriallcd_*.deb 2>/dev/null | head -n 1 || true)"
RPM_PATH="$(ls -t "${RPM_DIR}"/seriallcd-*.rpm 2>/dev/null | head -n 1 || true)"

if [[ -z "${DEB_PATH}" ]]; then
    echo "No .deb artifact found in ${DEB_DIR}" >&2
    exit 1
fi

if [[ -z "${RPM_PATH}" ]]; then
    echo "No .rpm artifact found in ${RPM_DIR}" >&2
    exit 1
fi

OUT_DIR="${ROOT}/releases/${CRATE_VERSION}"
mkdir -p "${OUT_DIR}"

BIN_OUT="${OUT_DIR}/seriallcd-${CRATE_VERSION}-${arch_label}"
cp "${BIN_PATH}" "${BIN_OUT}"
cp "${DEB_PATH}" "${OUT_DIR}/"
cp "${RPM_PATH}" "${OUT_DIR}/"

echo "Artifacts written to ${OUT_DIR}:"
echo "  $(basename "${BIN_OUT}")"
echo "  $(basename "${DEB_PATH}")"
echo "  $(basename "${RPM_PATH}")"

if [[ "${UPLOAD}" -eq 1 ]]; then
    require_cmd gh

    if ! git rev-parse "${RELEASE_TAG}" >/dev/null 2>&1; then
        echo "Git tag ${RELEASE_TAG} not found; create it before uploading." >&2
        exit 1
    fi

    echo "Uploading assets to GitHub release ${RELEASE_TAG}..."
    gh release create "${RELEASE_TAG}" "${OUT_DIR}"/* \
        --title "seriallcd ${CRATE_VERSION}" \
        --notes "Local release build for seriallcd ${CRATE_VERSION}"
fi
