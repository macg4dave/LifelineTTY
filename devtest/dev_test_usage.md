

## Usage docs for run-dev.sh 🛠

This describes the current behavior. Scenarios can be supplied via templates (copied to `~/.serial_lcd/config.toml`) or directly via the `--config-file` flag.

### Prerequisites

On your **developer machine** (Linux desktop):

- Rust toolchain installed (`cargo`).
- `ssh` and `scp` installed.
- A terminal emulator that supports:

  - `--title`
  - `-- bash -lc "..."`

  e.g. `gnome-terminal` (default in dev.conf.example).

On your **Raspberry Pi**:

- SSH enabled and reachable (`PI_HOST`).
- A user account (`PI_USER`) that can:
  - SSH in without interactive prompts (SSH keys recommended).
  - Write to the directory that will hold the `lifelinetty` binary (`PI_BIN`’s parent).
- lifelinetty.service stopped or disabled while using this dev loop, so the daemon doesn’t fight you for the UART.

### 1. Configure dev.conf

Start from the example:

```bash
cd /home/dave/github/LifelineTTY
cp devtest/dev.conf.example devtest/dev.conf
```

Edit dev.conf:

- Basic Pi info:

  ```bash
  PI_USER=pi
  PI_HOST=192.168.20.106   # or your Pi’s hostname/IP
  PI_BIN=/home/$PI_USER/lifelinetty/lifelinetty
  ```

- Local binary path (what we build and upload):

  ```bash
  LOCAL_BIN=target/debug/lifelinetty
  ```

- Common CLI arguments for both local and remote runs:

  ```bash
  COMMON_ARGS="--run --device /dev/ttyUSB0 --baud 9600 --cols 16 --rows 2"
  ```

  You can override `REMOTE_ARGS` and `LOCAL_ARGS` separately if needed:

  ```bash
  REMOTE_ARGS=""
  LOCAL_ARGS=""
  ```

- Build command:

  ```bash
  BUILD_CMD="cargo build"
  ```

- Terminal emulator:

  ```bash
  TERMINAL_CMD=gnome-terminal
  ```

- Optional log watcher pane (Terminal #4):

  ```bash
  ENABLE_LOG_PANE=true
  LOG_WATCH_CMD="watch -n 0.5 ls -lh /run/serial_lcd_cache"
  ```

- Remote process cleanup pattern:

  ```bash
  PKILL_PATTERN=lifelinetty
  ```

- **Scenario templates** (Milestone 1 behavior):

  These live under `devtest/config` and are copied into `~/.serial_lcd/config.toml` on each side before running:

  ```bash
  CONFIG_SOURCE_FILE=devtest/config/local/default.toml
  # LOCAL_CONFIG_SOURCE_FILE=devtest/config/local/default.toml
  # REMOTE_CONFIG_SOURCE_FILE=devtest/config/remote/default.toml
  ```

  Available templates:

  - `local/default.toml` (baseline local)
  - `remote/default.toml` (baseline remote)
  - `lifelinetty.toml` (shared baseline)
  - `test-16x2.toml` (baseline 16x2)
  - `test-20x4.toml` (20x4 geometry)
  - `uart-ama0.toml` (Pi onboard UART)
  - `stress-9600.toml` (baseline baud + polling enabled)
  - `stress-19200.toml` (higher-baud probe; only after 9600 is stable)

  - If you leave the per-side overrides commented out, local uses `CONFIG_SOURCE_FILE` and remote uses `REMOTE_CONFIG_SOURCE_FILE` by default.
  - To test different scenarios (e.g., local stub vs. real UART on Pi), point `LOCAL_CONFIG_SOURCE_FILE` and `REMOTE_CONFIG_SOURCE_FILE` at different TOMLs.

- **Direct config-file use** (Milestone 5):

  The binary now accepts `--config-file <path>` as the highest-priority source. To drive scenarios via the flag instead of template copies, add it to your args in `dev.conf`, e.g.:

  ```bash
  COMMON_ARGS="--run --config-file devtest/config/test-16x2.toml --device /dev/ttyUSB0 --baud 9600 --cols 16 --rows 2"
  ```

  Template copying remains supported; the flag simply lets you bypass `~/.serial_lcd/config.toml` if desired.

> Milestone 5 will add `--config-file` to the binary. At that point you’ll change `COMMON_ARGS` / `REMOTE_ARGS` / `LOCAL_ARGS` to pass the scenario file directly. For now, the script implements scenario selection via template copying.

### 2. What run-dev.sh does

When you run:

```bash
cd /home/dave/github/LifelineTTY
./devtest/run-dev.sh
```

it performs:

1. **Load config**

   - Sources dev.conf.
   - Validates `PI_HOST` and `PI_BIN`.
   - Verifies `ssh` and `scp` are available.
   - Verifies that:
     - `LOCAL_CONFIG_SOURCE_FILE` exists.
     - `REMOTE_CONFIG_SOURCE_FILE` exists.

2. **Set up local temp HOME**

   - Creates a temp directory: `LOCAL_CONFIG_HOME=$(mktemp -d -t lifelinetty-home.XXXXXX)`.
   - Creates `"$LOCAL_CONFIG_HOME/.serial_lcd"`.
   - Copies `LOCAL_CONFIG_SOURCE_FILE` into:

     ```text
     $LOCAL_CONFIG_HOME/.serial_lcd/config.toml
     ```

   - Prints something like:

     ```text
     [CONFIG] Using local template devtest/config/lifelinetty.toml (local HOME=/tmp/lifelinetty-home.XXXX)
     ```

3. **Build locally**

   - Runs `BUILD_CMD` via `bash -c "$BUILD_CMD"`.
     - Default: `cargo build`.
   - Warns if `LOCAL_BIN` doesn’t exist or isn’t executable afterwards.

4. **Pre-flight remote**

   - Asserts the Pi is reachable via SSH (`ssh -o BatchMode=yes -o ConnectTimeout=5`).
   - Ensures `dirname(PI_BIN)` exists (and is writable by `PI_USER`), or emits instructions if not.
   - Creates `~/.serial_lcd` on the Pi and copies `REMOTE_CONFIG_SOURCE_FILE` to:

     ```text
     ~/.serial_lcd/config.toml
     ```

5. **Binary sync**

   - Copies `LOCAL_BIN` up to `PI_BIN` via `scp`.
   - Runs `chmod +x "$PI_BIN"` remotely.

6. **Remote cleanup**

   - Runs `pkill -f "$PKILL_PATTERN" || true` on the Pi, to kill any stale `lifelinetty` processes.
   - You’re expected to stop lifelinetty.service yourself (e.g. via `systemctl stop lifelinetty.service`) before using this loop.

7. **Build command lines**

   - Remote:

     ```bash
     REMOTE_CMD="$PI_BIN $COMMON_ARGS $REMOTE_ARGS"
     ```

   - Local (ensuring it uses the temp HOME):

     ```bash
     LOCAL_CMD="HOME=$LOCAL_CONFIG_HOME $LOCAL_BIN $COMMON_ARGS $LOCAL_ARGS"
     ```

   - Logs:

     ```bash
     LOG_CMD="$LOG_WATCH_CMD"
     ```

8. **Launch terminals**

   For each window, if `TERMINAL_CMD` is found, it runs:

   ```bash
   $TERMINAL_CMD --title "<Title>" -- bash -lc "<Command>; exec bash" &
   ```

   Otherwise, it falls back to running the command in the current shell.

   - **Terminal #1 – SSH**:

     ```bash
     SSH_SHELL_CMD=${SSH_SHELL_CMD:-"ssh $PI_USER@$PI_HOST"}
     # Title: SSH
     ```

   - **Terminal #2 – Remote**:

     ```bash
     remote_launch=$(printf 'ssh %s %q' "$PI_USER@$PI_HOST" "$REMOTE_CMD")
     # Title: Remote
     ```

   - **Terminal #3 – Local**:

     ```bash
     LOCAL_CMD   # as built above
     # Title: Local
     ```

   - **Optional Terminal #4 – Logs** (if `ENABLE_LOG_PANE=true`):

     ```bash
     log_launch=$(printf 'ssh %s %q' "$PI_USER@$PI_HOST" "$LOG_CMD")
     # Title: Logs
     ```

   You’ll see a summary:

   ```text
   [TERM] Terminals launched. Watch for windows named SSH/Remote/Local/Logs (if enabled).
   ```

9. **Exit behavior**

   - Closing any terminal stops just that process.
   - Re-running run-dev.sh:
     - Rebuilds (per `BUILD_CMD`).
     - Re-syncs binary.
     - Kills any stale remote processes.
     - Opens a fresh set of terminals.

### 3. Optional watchers

You already have:

- watch.sh – runs `cargo watch -q -s "cargo run -- $RUN_ARGS"` for local runs (driven by `COMMON_ARGS`/`LOCAL_ARGS`).
- watch-remote.sh – runs `cargo watch -q -s "./devtest/run-dev.sh"` to automatically rebuild and redeploy to the Pi on source changes.

Examples:

```bash
cd /home/dave/github/LifelineTTY
./devtest/watch.sh          # local-only dev loop
./devtest/watch-remote.sh   # full Milestone 1 hardware loop on changes
```

---
