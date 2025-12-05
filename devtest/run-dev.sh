#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONF_FILE="$SCRIPT_DIR/dev.conf"

if [[ -f "$CONF_FILE" ]]; then
    # shellcheck disable=SC1090
    source "$CONF_FILE"
else
    echo "[ERROR] Missing dev.conf. Copy dev.conf.example and customize it." >&2
    exit 1
fi

command -v ssh >/dev/null 2>&1 || { echo "[ERROR] ssh is required." >&2; exit 1; }
command -v scp >/dev/null 2>&1 || { echo "[ERROR] scp is required." >&2; exit 1; }

PI_USER=${PI_USER:-pi}
PI_BIN=${PI_BIN:-/home/"$PI_USER"/lifelinetty/lifelinetty}
REMOTE_ARCH=${REMOTE_ARCH:-}
LOCAL_ARCH=${LOCAL_ARCH:-}

: "${PI_HOST:?PI_HOST must be set in dev.conf (hostname, e.g. 192.168.20.106)}"
: "${PI_BIN:?PI_BIN must be set in dev.conf}"

TERMINAL_CMD=${TERMINAL_CMD:-gnome-terminal}


LOCAL_BIN=${LOCAL_BIN:-target/debug/lifelinetty}
LOCAL_BIN_SOURCE=${LOCAL_BIN_SOURCE:-}
REMOTE_BIN_SOURCE=${REMOTE_BIN_SOURCE:-}
COMMON_ARGS=${COMMON_ARGS:---demo}
REMOTE_ARGS=${REMOTE_ARGS:-}
LOCAL_ARGS=${LOCAL_ARGS:-}
BUILD_CMD=${BUILD_CMD:-cargo build}
ENABLE_LOG_PANE=${ENABLE_LOG_PANE:-false}
LOG_WATCH_CMD=${LOG_WATCH_CMD:-watch -n 0.5 ls -lh /run/serial_lcd_cache}
PKILL_PATTERN=${PKILL_PATTERN:-lifelinetty}

# Scenario-aware config templates. By default, use a single template for both
# local and remote, but allow overrides so dev.conf can point each side at a
# different TOML if needed.
CONFIG_SOURCE_FILE=${CONFIG_SOURCE_FILE:-"$SCRIPT_DIR/config/local/default.toml"}
LOCAL_CONFIG_SOURCE_FILE=${LOCAL_CONFIG_SOURCE_FILE:-"$CONFIG_SOURCE_FILE"}
REMOTE_CONFIG_SOURCE_FILE=${REMOTE_CONFIG_SOURCE_FILE:-"$SCRIPT_DIR/config/remote/default.toml"}

if [[ ! -f "$LOCAL_CONFIG_SOURCE_FILE" ]]; then
    echo "[ERROR] Missing local dev config template $LOCAL_CONFIG_SOURCE_FILE" >&2
    exit 1
fi
if [[ ! -f "$REMOTE_CONFIG_SOURCE_FILE" ]]; then
    echo "[ERROR] Missing remote dev config template $REMOTE_CONFIG_SOURCE_FILE" >&2
    exit 1
fi

LOCAL_CONFIG_HOME_SET=false
if [[ -z "${LOCAL_CONFIG_HOME:-}" ]]; then
    LOCAL_CONFIG_HOME=$(mktemp -d -t lifelinetty-home.XXXXXX)
    LOCAL_CONFIG_HOME_SET=true
fi
LOCAL_CONFIG_DIR="$LOCAL_CONFIG_HOME/.serial_lcd"
mkdir -p "$LOCAL_CONFIG_DIR"
cp "$LOCAL_CONFIG_SOURCE_FILE" "$LOCAL_CONFIG_DIR/config.toml"
echo "[CONFIG] Using local template $LOCAL_CONFIG_SOURCE_FILE (local HOME=$LOCAL_CONFIG_HOME)"

cleanup_local_home() {
    if $LOCAL_CONFIG_HOME_SET && [[ -d "$LOCAL_CONFIG_HOME" ]]; then
        rm -rf "$LOCAL_CONFIG_HOME"
    fi
}
trap cleanup_local_home EXIT

printf '[BUILD] Using "%s"\n' "$BUILD_CMD"
bash -c "$BUILD_CMD"

REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ -n "$LOCAL_ARCH" ]]; then
    LOCAL_BIN_SOURCE="$REPO_ROOT/releases/debug/$LOCAL_ARCH/lifelinetty"
fi

if [[ -z "$LOCAL_BIN_SOURCE" ]]; then
    LOCAL_BIN_SOURCE="$LOCAL_BIN"
fi

if [[ -n "$REMOTE_ARCH" ]]; then
    REMOTE_BIN_SOURCE="$REPO_ROOT/releases/debug/$REMOTE_ARCH/lifelinetty"
fi

if [[ -z "$REMOTE_BIN_SOURCE" ]]; then
    REMOTE_BIN_SOURCE="$LOCAL_BIN"
fi

if [[ ! -x "$LOCAL_BIN_SOURCE" ]]; then
    echo "[ERROR] Local binary $LOCAL_BIN_SOURCE is missing or not executable." >&2
    exit 1
fi

if [[ ! -x "$REMOTE_BIN_SOURCE" ]]; then
    echo "[ERROR] Remote binary $REMOTE_BIN_SOURCE is missing or not executable." >&2
    exit 1
fi

remote_dir=$(dirname "$PI_BIN")
remote_target="$PI_USER@$PI_HOST"

SSH_OPTIONS=(-o BatchMode=yes -o ConnectTimeout=5)

run_remote_cmd() {
    local cmd="$1"
    if ! ssh "${SSH_OPTIONS[@]}" "$remote_target" "$cmd"; then
        local rc=$?
        echo "[ERROR] Remote command failed on $remote_target: $cmd" >&2
        echo "[ERROR] See above for the ssh output." >&2
        return $rc
    fi
}

assert_remote_reachable() {
    if ! run_remote_cmd "true"; then
        echo "[ERROR] Unable to reach $remote_target. Verify network connectivity and SSH credentials." >&2
        exit 1
    fi
}

ensure_remote_dir() {
    if ! run_remote_cmd "mkdir -p '$remote_dir'"; then
        cat <<'EOF' >&2
[ERROR] Could not create $remote_dir on $remote_target.
Make sure the path exists and is writable by $PI_USER (for example,
  ssh $remote_target sudo mkdir -p '$remote_dir' && \
    ssh $remote_target sudo chown $PI_USER '$remote_dir'
)
Alternately, point PI_BIN at a directory the SSH user already owns.
EOF
        exit 1
    fi
}

deploy_remote_config() {
    if ! run_remote_cmd "mkdir -p ~/.serial_lcd"; then
        echo "[ERROR] Unable to create remote config directory ~/.serial_lcd" >&2
        exit 1
    fi
    echo "[DEPLOY] Copying remote config $REMOTE_CONFIG_SOURCE_FILE to $remote_target:~/.serial_lcd/config.toml"
    scp "${SSH_OPTIONS[@]}" "$REMOTE_CONFIG_SOURCE_FILE" "$remote_target:~/.serial_lcd/config.toml"
}

assert_remote_reachable
echo "[DEPLOY] Ensuring remote directory $remote_dir"
ensure_remote_dir
deploy_remote_config

echo "[DEPLOY] Copying $REMOTE_BIN_SOURCE to $remote_target:$PI_BIN"
scp "${SSH_OPTIONS[@]}" "$REMOTE_BIN_SOURCE" "$remote_target:$PI_BIN"
run_remote_cmd "chmod +x '$PI_BIN'"

echo "[REMOTE] Killing stale processes matching '$PKILL_PATTERN'"
if ! run_remote_cmd "pkill -f '$PKILL_PATTERN' || true"; then
    echo "[WARN] pkill reported an error; collecting remote diagnostics" >&2
    run_remote_cmd "echo '[REMOTE-DEBUG] user: ' \$(whoami); id; ls -ld '$remote_dir'; ls -l '$PI_BIN' || true; ps -ef | grep -i lifelinetty || true" || true
fi

build_cmd_string() {
    local cmd="$1"; shift
    for chunk in "$@"; do
        [[ -n "$chunk" ]] || continue
        cmd+=" $chunk"
    done
    printf '%s' "$cmd"
}

REMOTE_CMD=$(build_cmd_string "$PI_BIN" "$COMMON_ARGS" "$REMOTE_ARGS")
LOCAL_CMD=$(build_cmd_string "HOME=$LOCAL_CONFIG_HOME" "$LOCAL_BIN_SOURCE" "$COMMON_ARGS" "$LOCAL_ARGS")
LOG_CMD="$LOG_WATCH_CMD"

terminal_available=true
if ! command -v "$TERMINAL_CMD" >/dev/null 2>&1; then
    echo "[WARN] Terminal program $TERMINAL_CMD not found; falling back to current shell" >&2
    terminal_available=false
fi

launch_window() {
    local title="$1"
    local cmd="$2"
    if $terminal_available; then
        printf '[TERM] Opening %s window\n' "$title"
        "$TERMINAL_CMD" --title "$title" -- bash -lc "$cmd; exec bash" &
    else
        printf '[TERM] %s command (fallback): %s\n' "$title" "$cmd"
        bash -lc "$cmd; exec bash"
    fi
}

# Terminal #1: persistent SSH shell for manual diagnostics/log inspection.
SSH_SHELL_CMD=${SSH_SHELL_CMD:-"ssh $remote_target"}
launch_window "SSH" "$SSH_SHELL_CMD"

REMOTE_LAUNCH_DELAY=${REMOTE_LAUNCH_DELAY:-1.0}
printf -v remote_launch 'ssh %s %q' "$remote_target" "sleep $REMOTE_LAUNCH_DELAY; $REMOTE_CMD"
launch_window "Remote" "$remote_launch"

launch_window "Local" "$LOCAL_CMD"

if [[ "$ENABLE_LOG_PANE" == "true" ]]; then
    printf -v log_launch 'ssh %s %q' "$remote_target" "$LOG_CMD"
    launch_window "Logs" "$log_launch"
fi

printf '[TERM] Terminals launched. Watch for windows named SSH/Remote/Local/Logs (if enabled).'
