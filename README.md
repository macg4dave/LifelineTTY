# SerialLCD (skeleton)

Serial-to-LCD daemon for Raspberry Pi / PiKVM style targets. It reads local status (to be defined) and sends lines over a serial link to a character LCD. This repository is intended to be driven entirely by Codex/Copilot; scope is locked down so the AI stays on target.

## Scope

- In scope: single binary `seriallcd` that drives a character LCD over a serial/UART link. Configuration is local only (CLI flags or a future config file).
- Out of scope: networking, cloud sync, telemetry, remote control, GUI, databases, authentication systems, or additional binaries unless explicitly requested.
- Target platform: Raspberry Pi OS (ARM); serial device such as `/dev/ttyAMA0` or `/dev/ttyUSB0`.
- Interface: CLI subcommand `run` with flags for serial device, baud, and LCD geometry.

Adjust the above if your hardware differs; keep the list explicit so the AI avoids feature creep.

## Layout

- `src/main.rs` — binary entrypoint.
- `src/lib.rs` — shared types and error handling.
- `src/cli.rs` — minimal CLI parser and defaults.
- `src/app.rs` — daemon wiring (placeholder).
- `src/serial.rs` — stub serial transport.
- `src/lcd.rs` — stub LCD driver.
- `seriallcd.service` - example systemd unit.
- `.github/instructions/` - prompt templates and AI guardrails.

## Usage (skeleton)

```sh
cargo build
cargo run -- run --device /dev/ttyUSB0 --baud 9600 --cols 16 --rows 2
```

### CLI flags

`seriallcd run [--device <path>] [--baud <number>] [--cols <number>] [--rows <number>] [--payload-file <path>] [--backoff-initial-ms <number>] [--backoff-max-ms <number>] [--pcf8574-addr <auto|0xNN>]`

- `--device` serial port path; default `/dev/ttyAMA0`.
- `--baud` baud rate; default `115200`.
- `--cols` / `--rows` LCD geometry; defaults `20x4`.
- `--payload-file` load and render a single JSON payload from disk (smoke testing helper).
- `--backoff-initial-ms` / `--backoff-max-ms` reconnect backoff window; defaults `500` / `10000`.
- `--pcf8574-addr` I2C backpack address or `auto` probe (default).
- `--log-level` (`error|warn|info|debug|trace`) and `--log-file <path>` for logging; env overrides: `SERIALLCD_LOG_LEVEL`, `SERIALLCD_LOG_PATH`.
- `--demo` cycles built-in demo pages on the LCD (no serial input required).
- `-h/--help`, `-V/--version` for docs/version.

## Build requirements & how-to

- Native (x86_64 host): Rust toolchain + build essentials. On Debian/Ubuntu/WSL2: `sudo apt update && sudo apt install -y build-essential pkg-config libudev-dev make`. Run `make x86` (or `cargo build --release`). Output lands in `releases/debug/x86/seriallcd[.exe]`.
- ARMv6 (Pi 1/Zero) via Docker: Docker Desktop/Engine with BuildKit + `buildx` enabled and internet access to pull toolchains. Run `make armv6` to build inside the `docker/Dockerfile.armv6` image and export the runtime filesystem to `releases/debug/armv6` (binary ends up at `releases/debug/armv6/usr/local/bin/seriallcd`). The ARMv6 build now uses an `armv6-linux-musleabihf` toolchain (static musl) to avoid Debian's ARMv7 baseline that was triggering `Illegal instruction` on real Pi 1/Zero hardware.
- Both at once: `make all` runs the native build and the Docker ARMv6 build.
- Clean artifacts: `make clean` wipes `releases/debug/`.
- If you prefer an image instead of extracted files, use the Dockerfile directly: `docker buildx build --platform linux/arm/v6 -f docker/Dockerfile.armv6 -t seriallcd:armv6 --load .`
- WSL2: ensure Docker Desktop is running and the Docker CLI/socket is available inside WSL2; install the same apt packages above inside WSL2 so `make` and the build toolchain are present.

### Testing / bin smoke check

- `cargo test` (or `make test`) runs unit tests plus a CLI smoke test that executes the built `seriallcd` binary: `seriallcd --version` and `seriallcd run --payload-file samples/test_payload.json`. This keeps the runtime path exercised as part of the build/test flow without needing real serial hardware.

### Config file

Persistent settings live at `~/.serial_lcd/config.toml` and use a simple `key = value` format:

```toml
device = "/dev/ttyAMA0"
baud = 115200
cols = 20
rows = 4
```

CLI flags override config values when provided.

## Dependencies (allowed)

- `hd44780-driver` for the LCD controller.
- `linux-embedded-hal` and `rppal` for Raspberry Pi I²C/hal support.
- `serialport` for synchronous UART.
- `tokio-serial` (behind the `async-serial` feature) and `tokio` for optional async serial handling.

## Next steps for Codex/Copilot

1) Confirm the hardware protocol: serial framing, LCD commands, and data source (what text goes to the display).  
2) Fill `.github/copilot-instructions.md` TODOs with concrete values (allowed crates, stable CLI flags, hardware assumptions).  
3) Implement the real serial transport and LCD command set; add integration tests or a simulator.  
4) Wire the daemon loop to pull the desired status metrics and refresh the display.  
5) Update the systemd unit if paths/users differ from your target environment.

## Docker cross-build (ARMv6)

See `docker/README.md` for a BuildKit-based flow to produce an `armv6` image targeting Raspberry Pi 1 / BCM2835:

```sh
docker buildx build --platform linux/arm/v6 -f docker/Dockerfile.armv6 -t seriallcd:armv6 .
```

## JSON payload format (v1)

The daemon ingests JSON objects (line-delimited) describing what to render. The raw frame must be <=512 bytes (`MAX_FRAME_BYTES`). Fields:

- `version` (int, optional) — only `1` is accepted when present.
- `line1`, `line2` (strings, required) — text for each row; `{0xNN}` placeholders emit custom glyphs; `mode:"banner"` clears `line2`.
- Bar graph: `bar` (0-100) or `bar_value` + `bar_max` (default `100`, minimum `1`) compute a percent; `bar_label` text prefix; `bar_line1`/`bar_line2` pick target row (dashboard mode forces bottom row).
- Backlight/alert: `backlight` (default `true`), `blink` (default `false`).
- Scrolling/paging: `scroll` (default `true`), `scroll_speed_ms` (default `250`), `page_timeout_ms` (default `4000`).
- Lifetimes: `duration_ms` or legacy `ttl_ms` — auto-expire after ms; `config_reload` (bool) hints the daemon to reload config.
- Actions: `clear` (bool) clears display first; `test` (bool) shows a test pattern.
- Mode/icons: `mode` in `normal|dashboard|banner` (default `normal`); `icons` array of `battery|arrow|heart` (unknown entries are ignored).
- Optional integrity: `checksum` hex string (`crc32` of the payload with `checksum` omitted); mismatches are rejected.

### Example payloads

```json
{"version":1,"line1":"Up 12:34  CPU 42%","line2":"RAM 73%","bar_value":73,"bar_max":100,"bar_label":"RAM","mode":"dashboard","page_timeout_ms":6000}
{"version":1,"line1":"ALERT: Temp","line2":"85C","blink":true,"duration_ms":5000}
{"version":1,"line1":"NET {0x00} 12.3Mbps","line2":"","bar_value":650,"bar_max":1000,"bar_label":"NET","icons":["battery"]}
{"version":1,"line1":"Long banner text that scrolls","line2":"ignored","mode":"banner","scroll_speed_ms":220}
{"version":1,"line1":"Backlight OFF demo","line2":"It should go dark","backlight":false}
{"version":1,"line1":"Clear + Test Pattern","line2":"Ensure wiring is OK","clear":true,"test":true}
{"version":1,"line1":"Scroll disabled","line2":"This line stays put","scroll":false,"page_timeout_ms":4500}
{"version":1,"line1":"Config reload hint","line2":"Reload config now","config_reload":true}
```

More samples live in `samples/payload_examples.json` and `samples/test_payload.json`.

### Feeding payloads

- One-shot from disk (no serial input): `seriallcd run --device /dev/ttyUSB0 --payload-file samples/test_payload.json`.
- Real hardware: upstream software should send newline-terminated JSON frames (≤512 bytes) at the configured baud; unknown lines are ignored and checksum failures are rejected.
- Built-in demo mode: `seriallcd run --demo` cycles through 20+ example payloads to exercise scrolling, bars, blink/backlight, icons, and test patterns without needing serial input.

### Runtime logging & reloads

- Logs default to `info` on stderr; set `--log-level`/`--log-file` or `SERIALLCD_LOG_LEVEL`/`SERIALLCD_LOG_PATH`. A shutdown summary prints frames accepted/rejected, checksum failures, duplicates, and reconnect attempts.
- Sending a frame with `"config_reload": true` reloads scroll/page timeouts, backoff settings, and device/baud (triggers reconnect if changed). Geometry (`cols/rows`) remains fixed until restart. Reload success/failure is logged with details.

### Limits and defaults

- Frame size: 512 bytes max (JSON string length). Oversized frames are rejected.
- Defaults: `scroll_speed_ms=250`, `page_timeout_ms=4000`, `bar_max=100`, `backlight=true`, `scroll=true`, `blink=false`, `mode=normal`.
- Dashboard mode always renders the bar on the bottom row even if `bar_line1=true`.
- Unknown `icons` entries are ignored; unknown `mode` falls back to `normal`.
