#!/usr/bin/env bash
set -euo pipefail

# Regression test: local-release.sh must work when invoked from ./scripts (cwd != repo root).
# We mock cargo/cargo-deb/cargo-generate-rpm so no real builds or packaging occur.

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$ROOT/target/test_localrelease_cwd"
rm -rf "$TMP" && mkdir -p "$TMP/bin"

cleanup() {
    if [[ "${KEEP_TMP:-0}" == "1" ]]; then
        echo "KEEP_TMP=1 set; leaving ${TMP} for inspection" >&2
        return 0
    fi
    rm -rf "$TMP" || true
    rm -rf "$ROOT/releases/0.2.0" || true
    rm -rf "$ROOT/target/debian" || true
    rm -rf "$ROOT/target/generate-rpm" || true
    rm -f "$ROOT/target/release/lifelinetty" || true
}
trap cleanup EXIT

fake_path="$TMP/bin"

# Fake cargo: create the release binary where local-release.sh expects it.
cat > "$fake_path/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail

case "${1:-}" in
build)
    mkdir -p target/release
    printf 'fake-binary' > target/release/lifelinetty
    exit 0

    ;;
deb)
    # cargo deb --no-build
    mkdir -p target/debian
    printf 'fake-deb' > target/debian/lifelinetty_0.2.0-1_amd64.deb
    echo "$(pwd)/target/debian/lifelinetty_0.2.0-1_amd64.deb"
    exit 0

    ;;
generate-rpm)
    mkdir -p target/generate-rpm
    printf 'fake-rpm' > target/generate-rpm/lifelinetty-0.2.0-1.x86_64.rpm
    exit 0

    ;;
metadata)
    # Not used by this test (we don't provide python3), but keep predictable behavior.
    exit 1

    ;;
esac

# For everything else, just succeed.
exit 0
CARGO
chmod +x "$fake_path/cargo"

# Provide the cargo subcommand shims so require_cmd passes.
cat > "$fake_path/cargo-deb" <<'DEB'
#!/usr/bin/env bash
set -euo pipefail
exit 0
DEB
chmod +x "$fake_path/cargo-deb"

cat > "$fake_path/cargo-generate-rpm" <<'RPM'
#!/usr/bin/env bash
set -euo pipefail
exit 0
RPM
chmod +x "$fake_path/cargo-generate-rpm"

# Ensure python3 is NOT required; we want the script to fall back to parsing Cargo.toml.
# (We intentionally do not create a python3 shim.)

export PATH="$fake_path:$PATH"
export LIFELINETTY_TEST_MODE=1

# Run from scripts/ directory to replicate the user's failure mode.
pushd "$ROOT/scripts" >/dev/null
if ! ./local-release.sh > "$TMP/out.txt" 2>&1; then
    cat "$TMP/out.txt" >&2 || true
    exit 1
fi
popd >/dev/null

# Assert success markers.
grep -q "Building lifelinetty 0.2.0" "$TMP/out.txt"
grep -q "Artifacts written to" "$TMP/out.txt"

# Assert the expected outputs exist.
[[ -f "$ROOT/releases/0.2.0/lifelinetty_v0.2.0_x86_64" ]]
[[ -f "$ROOT/releases/0.2.0/lifelinetty_v0.2.0_x86_64.deb" ]]
[[ -f "$ROOT/releases/0.2.0/lifelinetty_v0.2.0_x86_64.rpm" ]]

echo "local-release cwd regression test OK"
