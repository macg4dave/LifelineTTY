---
name: rust_api_surface
description: "Prompt template for managing and documenting the LifelineTTY public API surface."
---

Context
-------
Context
-------
- **Project**: LifelineTTY — a single Rust daemon for Raspberry Pi 1 (ARMv6) that exposes a CLI binary (`lifelinetty`) and renders HD44780 LCD output driven by newline-delimited JSON sourced from `/dev/ttyUSB0` at 9600 8N1 by default (config/CLI overrides can target `/dev/ttyAMA0`, `/dev/ttyS0`, USB adapters, etc.).
- **Crate**: `lifelinetty`. Public surface primarily flows through `src/lib.rs`, `src/cli.rs`, `src/app/`, and modules re-exported for integration tests.
- **Source layout**: runtime logic under `src/app/`, `src/serial/`, `src/display/`, `src/lcd_driver/`, `src/config/`; integration tests live under `tests/`.
- **Tooling**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`. Keep runtime RSS < 5 MB, avoid busy loops, and obey storage guardrails (`/run/serial_lcd_cache`, `~/.serial_lcd/config.toml`).

Hard constraints
----------------
1. Preserve backward-compatible APIs/CLI contracts unless the task explicitly authorizes a breaking change.
2. If a breaking change is unavoidable, include migration notes (README + docs) and update all affected tests.
3. Keep CLI flags and serial framing stable; ensure any new exports continue to respect RAM-disk + config rules.
4. Run `cargo test` (or the specified command) and include the full output after `cargo fmt`/`cargo clippy`.

Prompt template
---------------
Task:
"""
<Brief summary of API surface change>

Details:
- What to change: <public exports, visibility, re-exports, module structure>
- Files: <list files>
- Tests / migration notes: <describe test updates + doc/migration guidance>
"""

Assistant instructions
----------------------
1. Provide a concise 2–3 bullet plan referencing the modules you will touch (`src/lib.rs`, `src/app/*`, etc.).
2. Prefer additive APIs; if visibility must shrink, explain why and show migration steps.
3. Update/extend tests demonstrating the public contract (unit tests or integration tests under `tests/`).
4. Document any user-visible change (README, Rustdoc, spec files) and note migrations clearly.
5. Run `cargo test` (after `cargo fmt`/`cargo clippy`) and paste the full output.
6. Return:
   - Short summary of changes with file paths.
   - Exact patch(es) in `apply_patch` format.
   - The `cargo test` output showing passing tests (include failing output if reproduced first).
   - Suggested next steps or optional improvements.

Example prompts
---------------
- "Task: Export `LoopStats` struct for telemetry. Details: re-export from `src/app/render_loop.rs` via `src/lib.rs`. Files: `src/lib.rs`, `src/app/render_loop.rs`. Tests/migration: add unit test verifying public visibility."
- "Task: Hide `serial::fake` module from the crate root. Details: keep it test-only to reduce API surface. Files: `src/lib.rs`, `src/serial/mod.rs`. Tests: ensure `tests/fake_serial_loop.rs` still imports via `crate::serial::fake`. Docs: note in README dev section."
- "Task: Introduce `DisplayMode` enum to payload API. Details: add Rustdoc + re-export for external senders. Files: `src/payload/mod.rs`, `src/lib.rs`. Tests: update parser tests to cover new variants."
