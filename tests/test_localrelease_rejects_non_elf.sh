#!/usr/bin/env bash
set -euo pipefail

# Regression test: local-release.sh should refuse to publish/copy non-ELF binaries
# in real runs (i.e., when not in LIFELINETTY_TEST_MODE).

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$ROOT/target/test_localrelease_rejects_non_elf"
rm -rf "$TMP" && mkdir -p "$TMP/bin"

cleanup() {
    rm -rf "$TMP" || true
    rm -rf "$ROOT/releases/0.2.0" || true
    rm -rf "$ROOT/target/debian" || true
    rm -rf "$ROOT/target/generate-rpm" || true
    rm -f "$ROOT/target/release/lifelinetty" || true
}
trap cleanup EXIT

fake_path="$TMP/bin"

# Fake cargo: create a NON-ELF placeholder as the release binary.
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
    mkdir -p target/debian
    printf 'fake-deb' > target/debian/lifelinetty_0.2.0-1_amd64.deb
    exit 0
    ;;
generate-rpm)
    mkdir -p target/generate-rpm
    printf 'fake-rpm' > target/generate-rpm/lifelinetty-0.2.0-1.x86_64.rpm
    exit 0
    ;;
metadata)
    exit 1
    ;;
esac

exit 0
CARGO
chmod +x "$fake_path/cargo"

# Provide cargo subcommand shims so require_cmd passes.
for tool in cargo-deb cargo-generate-rpm python3; do
    cat > "$fake_path/$tool" <<TOOL
#!/usr/bin/env bash
set -euo pipefail
exit 0
TOOL
    chmod +x "$fake_path/$tool"
done

export PATH="$fake_path:$PATH"

# Run and assert it fails with the expected message.
set +e
out="$TMP/out.txt"
bash "$ROOT/scripts/local-release.sh" >"$out" 2>&1
rc=$?
set -e

if [[ $rc -eq 0 ]]; then
    echo "FAIL: expected local-release.sh to fail on non-ELF binary" >&2
    cat "$out" >&2 || true
    exit 2
fi

grep -q "is not an ELF binary" "$out" || {
    echo "FAIL: expected ELF guard message" >&2
    cat "$out" >&2 || true
    exit 2
}

echo "local-release non-ELF guard test OK"
