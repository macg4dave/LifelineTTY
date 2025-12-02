---
name: rust_feature
description: "Prompt template for adding features or capabilities to the LifelineTTY daemon."
---

Context
-------
- **Project**: LifelineTTY — a single Rust daemon for Raspberry Pi 1 (ARMv6) that reads newline-delimited JSON from `/dev/ttyUSB0` at 9600 8N1 by default (config/CLI overrides can point at `/dev/ttyAMA0`, `/dev/ttyS0`, USB adapters, etc.) and renders two LCD lines via a PCF8574 I²C backpack.
- **Crate**: `lifelinetty`. Key modules: `src/cli.rs`, `src/app/`, `src/serial/`, `src/display/`, `src/lcd_driver/`, `src/config/`, plus integration tests under `tests/`.
- **Tooling**: run `cargo fmt`, `cargo clippy -- -D warnings`, and the requested `cargo test` command (typically `cargo test`). Keep runtime RSS < 5 MB and avoid busy loops.
- **Storage policy**: only `~/.serial_lcd/config.toml` is persistent; every other write (temp files, logs, file chunks) must live in `/run/serial_lcd_cache`.
- **CLI stability**: flags stay `--run`, `--test-lcd`, `--test-serial`, `--device`, `--baud`, `--cols`, `--rows`, `--demo`. No new interfaces (no networking, HTTP, sockets) without explicit approval.
- **Allowed crates**: std, `hd44780-driver`, `linux-embedded-hal`, `rppal`, `serialport`, `tokio-serial` (async feature), `tokio` (only when async serial is required), `serde`, `serde_json`, `crc32fast`, `ctrlc`, optional `anyhow`/`thiserror`, `log`/`tracing`.

Hard constraints
----------------
1. Implement only the requested feature slice; no new CLI flags or protocols unless the task explicitly asks.
2. Update README/Rustdoc/config docs for any user-facing change (flags, payload schema, LCD behavior).
3. Add or refresh tests proving the feature (unit tests, integration under `tests/`, or golden payload fixtures).
4. Respect storage rules: only `~/.serial_lcd/config.toml` (persistent) and `/run/serial_lcd_cache` (tmp) may be written.
5. Run the specified `cargo test` command and include the full output after `cargo fmt`/`cargo clippy`.

Prompt template
---------------
Task:
"""
<One-line summary of the feature>

Details:
- What to build: <short description and acceptance criteria>
- Files to touch: <list or leave blank>
- Tests: <which tests to add/update or leave blank>
- Docs: <README/API docs to update or leave blank>
- Constraints / do not modify: <list any files/behaviors that must remain unchanged>
"""

Assistant instructions
----------------------
1. Outline a brief 2–3 bullet plan naming the modules you will touch.
2. Implement the smallest viable slice of the feature, honoring RAM-disk rules and keeping CLI semantics stable.
3. Add focused unit/integration tests that prove the behavior (e.g., new payload parsing, LCD overlay changes, config evolution).
4. Update README/Rustdoc/config docs when users see new behavior.
5. Run the specified `cargo test` (after `cargo fmt`/`cargo clippy`) and include the full output.
6. Deliver:
   - Short summary of changes with file paths.
   - Exact patch(es) in `apply_patch` format.
   - Test output showing passing runs (include failing output if reproduced beforehand).
   - Optional next steps (e.g., follow-up polish or documentation tasks).

Example prompts
---------------
- "Task: Add `display_mode = "panel"` payload handling. Details: mirror current LCD frame onto an auxiliary HD44780 panel. Files: `src/display/overlays.rs`, `src/payload/parser.rs`. Tests: extend `tests/integration_mock.rs::renders_panel_mode`. Docs: README display modes."
- "Task: Implement `serialsh` feature gate. Details: add CLI flag but keep default disabled, wire stub handler. Files: `src/cli.rs`, `src/app/mod.rs`. Tests: update `tests/bin_smoke.rs` to prove flag parsing. Docs: README CLI table."
- "Task: Add config-driven polling profile. Details: support `[profiles.default]` in `~/.serial_lcd/config.toml` to control CPU/disk polling intervals. Files: `src/config/loader.rs`, `src/app/render_loop.rs`. Tests: new unit test for profile merging + integration test verifying default."
