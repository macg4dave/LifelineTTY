---
name: rust_bugfix
description: "Prompt template for reproducing and fixing bugs inside LifelineTTY."
---

Context
-------
- **Project**: LifelineTTY — single Rust daemon for Raspberry Pi 1 (ARMv6) that ingests newline JSON frames via `/dev/ttyUSB0` at 9600 8N1 by default (config/CLI overrides can switch to `/dev/ttyAMA0`, `/dev/ttyS*`, USB adapters, etc.) and drives an HD44780 LCD via PCF8574.
- **Crate**: `lifelinetty`. Source hotspots: `src/cli.rs`, `src/app/`, `src/serial/`, `src/display/`, `src/lcd_driver/`, `src/config/`. Integration tests live in `tests/`.
- **Tooling**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` (or targeted command). Keep runtime RSS < 5 MB, avoid busy loops.
- **Storage**: only `~/.serial_lcd/config.toml` (persistent) and `/run/serial_lcd_cache` (tmp) may be written.
- **CLI**: flags stay `--run`, `--test-lcd`, `--test-serial`, `--device`, `--baud`, `--cols`, `--rows`, `--demo`. No new interfaces unless charter updated.

Hard constraints
----------------
1. Reproduce the bug/failing test locally and capture the exact error output.
2. Apply the smallest fix compliant with storage + CLI guardrails (no new flags/protocols).
3. Add a regression test that fails before the fix and passes after (unit or integration in `tests/`).
4. Update README/Rustdoc if user-facing behavior changes.
5. Run the specified `cargo test` command (after `cargo fmt`/`cargo clippy`) and include the full output.

Prompt template
---------------
Task:
"""
<One-line summary of the bug>

Details:
- Failure: <panic/backtrace/error output or failing test name>
- Repro steps: <commands or user flows to trigger the bug>
- Suspected files: <list or leave blank>
- Tests to run: <test command or leave default `cargo test -p <crate_name>`>
- Constraints / do not modify: <list any files/behaviors that must remain unchanged>
"""

Assistant instructions
----------------------
1. Summarize a 2–3 bullet plan referencing impacted modules/tests.
2. Reproduce the failure (or cite the failing test) and quote the exact panic/backtrace/CLI output.
3. Add a minimal regression test (unit or integration) that covers the issue.
4. Implement the fix with the smallest diff, respecting RAM-disk + CLI constraints.
5. Run the specified `cargo test` command; include failing output (if observed) followed by the passing run.
6. Return:
   - Concise summary with file paths.
   - Exact patch(es) in `apply_patch` format.
   - Test output (failure + success as applicable).
   - Manual verification steps (if hardware behavior was inspected).

Example prompts
---------------
- "Task: Fix panic when LCD dims after reconnect. Failure: `render_loop::set_backlight` unwraps because `current_frame` was `None`. Repro: `cargo test render_loop::backlight_recovers`. Files: `src/app/render_loop.rs`. Tests: add regression test plus update `tests/fake_serial_loop.rs`."
- "Task: `tests/bin_smoke.rs::applies_cols_rows_override` intermittently fails on Pi because defaults ignore CLI flags. Repro steps + log snippet. Files: `src/cli.rs`, `src/config/loader.rs`. Add regression test verifying CLI > config precedence."
