---
name: rust_mc
description: "Repository-aware Copilot template for multi-file changes inside LifelineTTY."
---

Context
-------
- **Project**: LifelineTTY — a tiny Rust daemon for Raspberry Pi 1 (ARMv6) that reads newline-delimited JSON from `/dev/ttyUSB0` (9600 8N1) by default and, via config/CLI overrides, any `/dev/tty*` path (e.g., `/dev/ttyAMA0`, `/dev/ttyS0`) before rendering two LCD lines through a PCF8574 I²C backpack.
- **Crate**: `lifelinetty`. Major modules: `src/cli.rs`, `src/app/`, `src/serial/`, `src/display/`, `src/lcd_driver/`, `src/config/`, with integration tests under `tests/`.
- **Tooling**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release` when needed. Keep runtime RSS < 5 MB and avoid busy loops.
- **Storage policy**: only `~/.serial_lcd/config.toml` is persistent; every other write (logs, temp files, chunk buffers) must stay inside `/run/serial_lcd_cache`.
- **CLI stability**: flags stay `--run`, `--test-lcd`, `--test-serial`, `--device`, `--baud`, `--cols`, `--rows`, `--demo`. No new interfaces (no networking, HTTP, sockets) without explicit approval.
- **Allowed crates**: std, `hd44780-driver`, `linux-embedded-hal`, `rppal`, `serialport`, `tokio-serial` (optional `async-serial` feature), `tokio` (only when async serial is required), `serde`, `serde_json`, `crc32fast`, `ctrlc`, optional `anyhow`/`thiserror`, `log`/`tracing`.

Hard constraints
----------------
1. Make the smallest change that satisfies the request; never invent new protocols or CLI flags without written approval.
2. Add or refresh tests for every behavioral change (module tests or integration tests under `tests/`).
3. Update README/Rustdoc whenever user-facing behavior (flags, config schema, LCD output) changes.
4. Keep writes inside `/run/serial_lcd_cache` unless operating on `~/.serial_lcd/config.toml`.
5. Run the requested `cargo test` command locally and include the full output (after `cargo fmt`/`cargo clippy`).

Prompt template
---------------
Task:
"""
<One-line summary of the requested change>

Details:
- What to change: <short description of edits or behavior change>
- Files to touch: <comma-separated list>
- Tests: <which tests to add/run>
- Docs: <README/spec updates or leave blank>
- Constraints / do not modify: <list anything that must stay intact>
"""

Assistant instructions
----------------------
1. Outline a 2–3 bullet plan referencing the affected modules.
2. Implement the change using idiomatic Rust (`Result`, no unchecked `unwrap` outside tests) and respect RAM-disk/LCD constraints.
3. Add/adjust tests proving the behavior (module tests or integration suites under `tests/`).
4. Run `cargo test` (or the provided command) and include the full output. Fix failures before returning.
5. Return:
   - A concise summary with file paths.
   - Exact patches using `apply_patch` V4A diff format.
   - Test output (and failure logs, if any, before the fix).
   - Optional follow-up ideas or manual verification notes.

Example prompts
---------------
- "Task: Harden config loader defaults. Details: reject invalid LCD dimensions, ensure scroll/page fallbacks. Files: `src/config/loader.rs`, `tests/bin_smoke.rs`. Tests: `cargo test config`. Docs: README config table."
- "Task: Add reconnect metrics to `--test-serial`. Details: print counters from `LoopStats`. Files: `src/app/render_loop.rs`, `src/cli.rs`. Tests: extend `tests/bin_smoke.rs::prints_loop_stats`. Constraints: keep CLI flag list unchanged."
- "Task: Refresh LCD overlays for dashboard mode. Details: adjust `src/display/overlays.rs` to reduce flicker, add regression test. Files: `src/display/overlays.rs`, `tests/integration_mock.rs`. Tests: `cargo test overlays`."

Usage notes
-----------
- Keep the guardrails near the top; Copilot only reads the first few kilobytes.
- When changes impact serial framing, RAM usage, or CLI flags, call those out explicitly so reviewers can cross-check against `.github/copilot-instructions.md`.
