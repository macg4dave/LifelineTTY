#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Build release artifacts locally (binary + .deb + .rpm) and optionally upload them to
GitHub Releases. The version is read from Cargo.toml; tag defaults to "v<version>".

Usage: scripts/local-release.sh [--target <triple>] [--targets <t1,t2>] [--all-targets] [--tag <git-tag>] [--upload|--all]

  --target <triple>   Optional Rust target triple (can be repeated). Example:
                      armv7-unknown-linux-gnueabihf
  --targets <list>    Comma-separated list of target triples (overrides --target)
    --all-targets       Build for x86_64 + armv6 + armv7 + arm64 (predefined list)
  --tag <git-tag>     Override release tag (default: v<Cargo version>)
  --upload            Push artifacts to GitHub Releases using the GitHub CLI.
  --all               Convenience: build + package + upload (same as passing --upload).
  -h, --help          Show this message.

Prereqs: cargo, cargo-deb, cargo-generate-rpm, and optionally python3 + the gh CLI.
Version detection prefers `cargo metadata` + python3, and falls back to parsing Cargo.toml.
For cross builds via Docker, you also need Docker BuildKit/buildx.
EOF
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Missing required tool: $1" >&2
        exit 1
    fi
}

is_elf_binary() {
    local path="$1"
    if [[ ! -f "${path}" ]]; then
        return 1
    fi
    # Check ELF magic bytes: 0x7F 45 4C 46
    # Avoid external deps like `file` so tests can mock binaries.
    local magic
    magic="$(head -c 4 "${path}" 2>/dev/null | od -An -t x1 2>/dev/null | tr -d ' \n')"
    [[ "${magic}" == "7f454c46" ]]
}

strip_binary_if_possible() {
    local path="$1"

    # Only strip real ELF binaries. Tests create text placeholders.
    if ! is_elf_binary "${path}"; then
        return 0
    fi

    local strip_tool=""
    if command -v llvm-strip >/dev/null 2>&1; then
        strip_tool="llvm-strip"
    elif command -v strip >/dev/null 2>&1; then
        strip_tool="strip"
    fi

    if [[ -z "${strip_tool}" ]]; then
        echo "Warning: no strip tool found (install binutils or llvm); leaving ${path} unstripped" >&2
        return 0
    fi

    # Strip debug symbols; keep the binary otherwise intact.
    if ! "${strip_tool}" --strip-debug "${path}" >/dev/null 2>&1; then
        echo "Warning: ${strip_tool} failed on ${path}; leaving unstripped" >&2
        return 0
    fi
}

get_crate_version() {
    local version=""

    # Prefer cargo metadata (more robust in workspaces), but don't hard-fail if python3
    # isn't available; fall back to parsing Cargo.toml.
    if command -v cargo >/dev/null 2>&1 && command -v python3 >/dev/null 2>&1; then
        if version="$(
            cargo metadata --no-deps --format-version 1 2>/dev/null |
                python3 -c '
import json, sys
meta = json.load(sys.stdin)
for pkg in meta.get("packages", []):
    if pkg.get("name") == "lifelinetty":
        print(pkg.get("version"))
        sys.exit(0)
print("Failed to find lifelinetty in cargo metadata", file=sys.stderr)
sys.exit(1)
' 2>/dev/null
        )"; then
            :
        else
            version=""
        fi
    fi

    if [[ -z "${version}" ]]; then
        # Parse Cargo.toml for [package] version = "...".
        version="$(awk '
            $0 ~ /^\[package\][[:space:]]*$/ { in_pkg = 1; next }
            in_pkg && $0 ~ /^\[/ { in_pkg = 0 }
            in_pkg && $0 ~ /^version[[:space:]]*=/ {
                v = $0
                sub(/^version[[:space:]]*=[[:space:]]*/, "", v)
                sub(/#.*/, "", v)
                gsub(/[[:space:]]/, "", v)
                gsub(/^"|"$/, "", v)
                print v
                exit
            }
        ' "${ROOT}/Cargo.toml")"
    fi

    if [[ -z "${version}" ]]; then
        echo "Could not determine crate version" >&2
        return 1
    fi

    printf '%s\n' "${version}"
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Always run from the repository root so `cargo*` tools can find Cargo.toml even
# when this script is invoked from another working directory (e.g. ./scripts).
cd "${ROOT}"

TARGETS=()
TAG_OVERRIDE=""
UPLOAD=0
ALL_TARGETS=0
UPLOAD_ASSETS=()
ALL_TARGETS_DEFAULT=("x86_64-unknown-linux-gnu" "arm-unknown-linux-gnueabihf" "armv7-unknown-linux-gnueabihf" "aarch64-unknown-linux-gnu")

# Load reusable build helpers (derive_arch_label, is_inside_container, has_rust_target_installed)
source "${ROOT}/scripts/build_helpers.sh"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)
            TARGETS+=("${2:-}")
            shift 2
            ;;
        --targets)
            IFS=',' read -r -a TARGETS <<< "${2:-}"
            shift 2
            ;;
        --all-targets)
            ALL_TARGETS=1
            shift
            ;;
        --tag)
            TAG_OVERRIDE="${2:-}"
            shift 2
            ;;
        --all)
            UPLOAD=1
            shift
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

CRATE_VERSION="$(get_crate_version)"

if [[ "${ALL_TARGETS}" -eq 1 ]]; then
    TARGETS=("${ALL_TARGETS_DEFAULT[@]}")
fi

if [[ "${#TARGETS[@]}" -eq 0 ]]; then
    TARGETS=("")
fi

if [[ -z "${CRATE_VERSION}" ]]; then
    echo "Could not determine crate version" >&2
    exit 1
fi

RELEASE_TAG="${TAG_OVERRIDE:-v${CRATE_VERSION}}"

package_artifacts() {
    local triple="$1"
    local arch_label="$2"
    local target_dir="$3"
    local deb_args_str="$4"
    local rpm_args_str="$5"

    local BIN_PATH="${ROOT}/${target_dir}/release/lifelinetty"
    local DEB_DIR="${ROOT}/${target_dir}/debian"
    local RPM_DIR="${ROOT}/${target_dir}/generate-rpm"

    IFS=' ' read -r -a deb_args <<< "${deb_args_str}"
    IFS=' ' read -r -a rpm_args <<< "${rpm_args_str}"

    if [[ ! -f "${BIN_PATH}" ]]; then
        echo "Binary not found at ${BIN_PATH}" >&2
        exit 1
    fi

    # In real release runs we expect an actual ELF binary. Test scripts in this
    # repo intentionally use placeholder files; they should set
    # LIFELINETTY_TEST_MODE=1 to bypass this check.
    if [[ "${LIFELINETTY_TEST_MODE:-0}" != "1" ]]; then
        if ! is_elf_binary "${BIN_PATH}"; then
            echo "Built artifact at ${BIN_PATH} is not an ELF binary." >&2
            echo "Tip: remove target/ and rebuild (e.g. 'cargo clean'), or ensure you're not running with mocked build tools." >&2
            exit 1
        fi
    fi

    # Make the target binary real and small: strip when possible.
    strip_binary_if_possible "${BIN_PATH}"

    # Ensure the deb/rpm metadata paths (Cargo.toml) always point at the correct
    # binary. For cross targets, stage the built binary into target/release.
    local staged_backup=""
    local staged_target="${ROOT}/target/release/lifelinetty"
    mkdir -p "${ROOT}/target/release"
    if [[ -n "${triple}" ]]; then
        if [[ -f "${staged_target}" ]]; then
            staged_backup="$(mktemp)"
            cp "${staged_target}" "${staged_backup}"
        fi
        cp "${BIN_PATH}" "${staged_target}"
    fi
    # Also strip the staged path (host path used by packaging metadata) when it
    # differs from BIN_PATH.
    strip_binary_if_possible "${staged_target}"

    # We strip ourselves above (only if ELF), so tell cargo-deb not to run strip.
    cargo deb --no-strip "${deb_args[@]}"
    cargo generate-rpm "${rpm_args[@]}"

    local DEB_PATH
    local RPM_PATH
    # cargo-deb and cargo-generate-rpm sometimes write to target/{debian,generate-rpm}
    # even when --target is specified. Search both locations.
    DEB_PATH="$(
        ls -t "${DEB_DIR}"/lifelinetty_*.deb "${ROOT}/target/debian"/lifelinetty_*.deb 2>/dev/null | head -n 1 || true
    )"
    RPM_PATH="$(
        ls -t "${RPM_DIR}"/lifelinetty-*.rpm "${ROOT}/target/generate-rpm"/lifelinetty-*.rpm 2>/dev/null | head -n 1 || true
    )"

    if [[ -z "${DEB_PATH}" ]]; then
        echo "No .deb artifact found in ${DEB_DIR}" >&2
        exit 1
    fi

    if [[ -z "${RPM_PATH}" ]]; then
        echo "No .rpm artifact found in ${RPM_DIR}" >&2
        exit 1
    fi

    local BIN_OUT="${OUT_DIR}/lifelinetty_v${CRATE_VERSION}_${arch_label}"
    local DEB_OUT="${OUT_DIR}/lifelinetty_v${CRATE_VERSION}_${arch_label}.deb"
    local RPM_OUT="${OUT_DIR}/lifelinetty_v${CRATE_VERSION}_${arch_label}.rpm"

    cp "${BIN_PATH}" "${BIN_OUT}"
    cp "${DEB_PATH}" "${DEB_OUT}"
    cp "${RPM_PATH}" "${RPM_OUT}"

    echo "Artifacts written to ${OUT_DIR}:"
    echo "  $(basename "${BIN_OUT}")"
    echo "  $(basename "${DEB_OUT}")"
    echo "  $(basename "${RPM_OUT}")"

    UPLOAD_ASSETS+=("${BIN_OUT}" "${DEB_OUT}" "${RPM_OUT}")

    # Restore any staged host-path binary after packaging so subsequent targets
    # don't accidentally reuse it.
    if [[ -n "${triple}" ]]; then
        if [[ -n "${staged_backup}" ]]; then
            cp "${staged_backup}" "${staged_target}" || true
            rm -f "${staged_backup}" || true
        else
            rm -f "${staged_target}" || true
        fi
    fi
}

build_with_cargo() {
    local triple="$1"
    local arch_label="$2"

    local target_dir="target"
    local deb_args=(--no-build)
    local rpm_args=()
    local build_args=(--release)

    if [[ -n "${triple}" ]]; then
        if ! rustup target list --installed | grep -qx "${triple}"; then
            echo "Rust target ${triple} not installed. Install with: rustup target add ${triple}" >&2
            exit 1
        fi
        build_args+=(--target "${triple}")
        deb_args+=(--target "${triple}")
        rpm_args+=(--target "${triple}")
        target_dir="target/${triple}"
    fi

    echo "Building lifelinetty ${CRATE_VERSION} (${arch_label})..."
    if [[ "${SKIP_BUILD_ACTIONS:-0}" == "1" ]]; then
        echo "SKIP_BUILD_ACTIONS=1; skipping cargo build (test mode)"
        return 0
    fi

    local bin_path="${ROOT}/${target_dir}/release/lifelinetty"
    local attempt
    for attempt in 1 2; do
        cargo build "${build_args[@]}"

        if [[ "${LIFELINETTY_TEST_MODE:-0}" == "1" ]]; then
            break
        fi

        if is_elf_binary "${bin_path}"; then
            break
        fi

        if [[ "${attempt}" -eq 1 ]]; then
            echo "Warning: ${bin_path} is not an ELF binary; cleaning and rebuilding (target/ may be polluted)." >&2
            cargo clean >/dev/null 2>&1 || true
            continue
        fi

        echo "Built artifact at ${bin_path} is not an ELF binary even after a clean rebuild." >&2
        exit 1
    done

    package_artifacts "${triple}" "${arch_label}" "${target_dir}" "${deb_args[*]}" "${rpm_args[*]}" 
}

build_with_docker() {
    local triple="$1"
    local arch_label="$2"
    local platform="$3"
    local dockerfile="$4"
    local image="$5"
    local target_dir="target/${triple}"
    local release_dir="${ROOT}/${target_dir}/release"

    if [[ "${SKIP_BUILD_ACTIONS:-0}" == "1" ]]; then
        echo "SKIP_BUILD_ACTIONS=1; skipping docker build (test mode)"
        return 0
    fi
    require_cmd docker
    mkdir -p "${release_dir}"

    echo "Building lifelinetty ${CRATE_VERSION} (${arch_label}) via Docker (${platform})..."
    docker buildx build --platform "${platform}" --target artifact --load -f "${dockerfile}" -t "${image}" "${ROOT}"
    local cid
    cid="$(docker create "${image}")"
    docker cp "${cid}:/usr/local/bin/lifelinetty" "${release_dir}/lifelinetty"
    docker rm "${cid}" >/dev/null

    local deb_args=(--no-build --target "${triple}")
    local rpm_args=(--target "${triple}")
    package_artifacts "${triple}" "${arch_label}" "${target_dir}" "${deb_args[*]}" "${rpm_args[*]}"
}

OUT_DIR="${ROOT}/releases/${CRATE_VERSION}"
mkdir -p "${OUT_DIR}"

for TARGET_TRIPLE in "${TARGETS[@]}"; do
    arch_label="$(derive_arch_label "${TARGET_TRIPLE}")"

    case "${TARGET_TRIPLE}" in
        "")
            build_with_cargo "" "${arch_label}"
            ;;
        x86_64-unknown-linux-gnu)
            if (uname -m | grep -Eq 'x86_64|amd64') && ! is_inside_container; then
                echo "Detected x86_64 host — building natively with cargo"
                build_with_cargo "" "${arch_label}"
            else
                build_with_docker "${TARGET_TRIPLE}" "${arch_label}" "linux/amd64" "docker/Dockerfile.amd64" "lifelinetty:amd64"
            fi
            ;;
        arm-unknown-linux-gnueabihf)
            build_with_docker "${TARGET_TRIPLE}" "${arch_label}" "linux/arm/v6" "docker/Dockerfile.armv6" "lifelinetty:armv6"
            ;;
        armv7-unknown-linux-gnueabihf)
            build_with_docker "${TARGET_TRIPLE}" "${arch_label}" "linux/arm/v7" "docker/Dockerfile.armv7" "lifelinetty:armv7"
            ;;
        aarch64-unknown-linux-gnu)
            # Prefer a native host build when we're on an aarch64 host and rustup target exists
            # - FORCE_DOCKER=1 will force Docker build
            # - USE_HOST_BUILD=1 (default behavior) will attempt host build first
            if [[ "${FORCE_DOCKER:-0}" == "1" ]]; then
                echo "FORCE_DOCKER=1 set; using Docker for aarch64"
                build_with_docker "${TARGET_TRIPLE}" "${arch_label}" "linux/arm64/v8" "docker/Dockerfile.arm64" "lifelinetty:arm64"
            else
                if is_inside_container; then
                    echo "Container environment detected; using Docker for aarch64"
                    build_with_docker "${TARGET_TRIPLE}" "${arch_label}" "linux/arm64/v8" "docker/Dockerfile.arm64" "lifelinetty:arm64"
                else
                    if [[ "${USE_HOST_BUILD:-1}" == "1" ]] && has_rust_target_installed "${TARGET_TRIPLE}" && (uname -m | grep -Eq 'aarch64|arm64'); then
                        echo "Detected aarch64 host and rustup target installed — building natively with cargo"
                        build_with_cargo "${TARGET_TRIPLE}" "${arch_label}"
                    else
                        echo "Falling back to Docker cross-build for aarch64"
                        build_with_docker "${TARGET_TRIPLE}" "${arch_label}" "linux/arm64/v8" "docker/Dockerfile.arm64" "lifelinetty:arm64"
                    fi
                fi
            fi
            ;;
        *)
            build_with_cargo "${TARGET_TRIPLE}" "${arch_label}"
            ;;
    esac
done

if [[ "${UPLOAD}" -eq 1 ]]; then
    require_cmd gh

    if ! git rev-parse "${RELEASE_TAG}" >/dev/null 2>&1; then
        echo "Git tag ${RELEASE_TAG} not found; create it before uploading." >&2
        exit 1
    fi

    echo "Uploading assets to GitHub release ${RELEASE_TAG}..."
    if gh release view "${RELEASE_TAG}" >/dev/null 2>&1; then
        gh release upload "${RELEASE_TAG}" "${UPLOAD_ASSETS[@]}" --clobber
    else
        gh release create "${RELEASE_TAG}" "${UPLOAD_ASSETS[@]}" \
            --title "lifelinetty ${CRATE_VERSION}" \
            --notes "Local release build for lifelinetty ${CRATE_VERSION}"
    fi
fi
