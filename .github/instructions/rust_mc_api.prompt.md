---
name: rust_mc_api
description: "Prompt template for LifelineTTY public API and shared module changes."
---

Context
-------
- **Project**: LifelineTTY — Raspberry Pi 1 daemon that ingests newline JSON via `/dev/ttyUSB0` at 9600 8N1 by default (config/CLI overrides can point at `/dev/ttyAMA0`, `/dev/ttyS*`, USB adapters, etc.) and renders HD44780 LCD output.
- **Crate**: `lifelinetty`. Public APIs reside in `src/lib.rs`, `src/payload/`, `src/display/`, `src/lcd_driver/`, `src/app/`, plus helper modules re-exported for integration tests.
- **Constraints**: CLI flags fixed (`--run`, `--test-lcd`, `--test-serial`, `--device`, `--baud`, `--cols`, `--rows`, `--demo`); no new protocols, no networking, storage limited to `/run/serial_lcd_cache` + `~/.serial_lcd/config.toml`.
- **Tooling**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` (unit + integration). Keep runtime RSS < 5 MB and avoid busy loops.

Hard constraints
----------------
1. Preserve backward-compatible APIs unless the task explicitly authorizes breaking changes.
2. When breaking changes are unavoidable, add migration notes (README/docs) and update all affected tests.
3. Keep CLI flags, serial framing, and RAM-disk policies intact.
4. Run `cargo test` (after `cargo fmt`/`cargo clippy`) and include the full output.

Prompt template
---------------
Task:
"""
<Brief summary of API change>

Details:
- What to change: <public API additions/removals/behavior changes>
- Files: <list files>
- Tests/migration notes: <describe changes to tests or migration guidance>
"""

Assistant instructions
----------------------
1. Provide a concise 2–3 bullet plan referencing the modules you will edit.
2. Prefer additive APIs; if removing or changing signatures, justify the change and provide migration guidance.
3. Update/add tests demonstrating the contract (module tests, `tests/*`, doc examples).
4. Document user-visible behavior (README, docs, Rustdoc) and call out migrations explicitly.
5. Run `cargo test` (after formatting/linting) and include the full output.
6. Suggest optional improvements or cleanup when relevant.

Example prompts
---------------
- "Task: Re-export `payload::RenderFrame` for downstream tools. Details: add `pub use payload::RenderFrame;` in `src/lib.rs` and document the contract in `README.md`. Files: `src/lib.rs`, `README.md`. Tests: ensure `tests/integration_mock.rs` compiles without module path changes."
- "Task: Deprecate `payload::defaults()` in favor of `AppConfig::render_defaults`. Details: keep old function but mark `#[deprecated]`, update doc comments, add migration note in README. Files: `src/payload/mod.rs`, `README.md`. Tests: update payload parser tests to cover new helper."
- "Task: Hide `display::overlays::render_if_allowed` from public API while adding `display::overlays::RenderGate`. Details: make helper private, expose new struct with same functionality, document how to migrate. Files: `src/display/overlays.rs`, `src/lib.rs`, `docs/lcd_patterns.md`. Tests: add overlay unit tests." 
