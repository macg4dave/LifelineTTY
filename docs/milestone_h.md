# 📌 Milestone H — Custom Character Toolkit & Icon Library (P22)

*Draft specification for Milestone H of the LifelineTTY project. This file documents design intent only—no code here is executable.*

---

> Scope alignment: Roadmap item **P22 (custom character support + built-in icons)** promotes into Milestone H. Work delivered here must follow the charter guardrails: single `lifelinetty` binary, HD44780 + PCF8574 I²C wiring only, RAM-disk cache usage, <5 MB RSS on Raspberry Pi 1, and stable CLI/config schemas. This milestone also consumes the driver plumbing from **P21** where necessary (CGRAM helpers) but does not extend the CLI beyond existing flags.

## Goal

Ship a deterministic custom-character subsystem that can preload curated icon banks, hot-swap glyphs at runtime, and expose them through the payload/schema layer without forcing users to hand-code bitmaps. Operators should be able to request icons by semantic name (e.g., `heart`, `signal_5_bar`, `degree_c`) and trust that the render loop will orchestrate CGRAM slots, dedupe writes, and fall back gracefully when hardware constraints prevent loading more glyphs.

### Success criteria

- Built-in icon packs live in `src/payload/icons.rs` (or adjacent module) with unit tests enforcing byte patterns against the upstream reference (`duinoWitchery/hd44780`, public domain).
- Render loop manages at least two CGRAM banks (current frame + standby), swapping glyphs lazily so displays never flicker or block serial ingestion.
- Payloads can name icons (per line + overlay) and optionally request raw 5×8 bitmaps; invalid names or oversubscribed banks lead to documented fallbacks instead of panics.
- Config/profile layer exposes a safe default bank order plus optional user overrides stored in `~/.serial_lcd/config.toml`.
- Documentation (`docs/lcd_patterns.md`, `samples/*.json`, this milestone file) and tests (`src/display`, `tests/bin_smoke.rs`, `tests/integration_mock.rs`) cover icon selection, overrides, and CGRAM churn.

## Current architecture snapshot

- `src/display/lcd.rs` offers `Lcd::load_custom_bitmaps` but upstream callers must juggle 8 slots manually.
- `src/payload/icons.rs` only covers lightweight inline glyphs (e.g., heart) and lacks a canonical registry or licensing notes.
- `app::render_loop` writes text-first, so icon churn requires full-frame rewrites and risks I²C flicker if CGRAM changes mid-frame.
- `samples/` payloads reference only ASCII characters, so there is no regression coverage for CGRAM behavior.

Milestone H formalizes the icon pipeline, ensures we never exceed 8 active glyphs, and provides doc/test coverage for bank swaps.

## Workstreams

### 1. Icon asset ingest + registry (P22 ↔ source repo)

**Files:** `src/payload/icons.rs`, new `docs/icon_library.md`, `docs/roadmap.md`, `README.md` (icon section).

- Mirror the public-domain bitmaps from [duinoWitchery/hd44780 — LCDCustomChars](https://github.com/duinoWitchery/hd44780/blob/master/examples/ioClass/hd44780_I2Clcd/LCDCustomChars/LCDCustomChars.ino) into a structured Rust registry:
  - Define `IconBitmap { name: &'static str, rows: [u8; 8], tags: &[IconTag] }`.
  - Group icons into packs (status, arrows, weather, diagnostics) to simplify CGRAM planning.
  - Preserve provenance comments + license statements in both the Rust module and the Markdown catalog.
- Add unit tests guaranteeing the byte arrays stay in sync with the catalog (hash comparison or static asserts).

### 2. CGRAM bank manager & runtime plumbing

**Files:** `src/display/lcd.rs`, `src/lcd_driver/mod.rs`, `src/app/render_loop.rs`, `src/state.rs`.

- Implement a small `CgramBank` struct that tracks slot occupancy, last-used timestamps, and checksum of bitmap data.
- Render loop asks the bank manager for `IconPlan` prior to writing LCD lines:
  1. Determine which icons the frame needs (from payload overlay + config defaults).
  2. Reuse slots if the same bitmap is already loaded.
  3. When more than 8 icons requested, fall back to priority rules (user-configured or default heuristics) and record which ones were skipped.
- Ensure CGRAM writes happen up-front in each render iteration; once the glyphs are staged we write the text buffer. Guard this sequence with small timing gaps (per HD44780 spec) but avoid busy loops.
- Provide metrics/log hooks under `/run/serial_lcd_cache/icon_bank.log` when swaps occur too frequently; this aids tuning before Milestone E multi-panel work.

### 3. Payload & config integration

**Files:** `src/payload/parser.rs`, `src/payload/schema.rs`, `samples/payload_examples.json`, `src/config/loader.rs`, `docs/architecture.md`.

- Extend payload schema with optional `"icons": ["name"]` arrays per line/overlay plus an optional inline bitmap descriptor:

  ```json
  {
    "schema_version": 1,
    "line1": "TEMP \u0000",
    "icons": {"slot0": "thermometer", "slot1": {"rows": [4,14,31,4,4,14,0,0]}}
  }
  ```

- Parser validates icon names against the registry and enforces ≤8 unique glyphs per frame.
- Config gains a `[display.icons]` table allowing operators to reorder default banks, disable certain packs, or pin icons to slots.
- Document the schema additions and config knobs in README/docs.

### 4. Tooling, tests, and developer ergonomics

**Files:** `tests/bin_smoke.rs`, `tests/integration_mock.rs`, `tests/fake_serial_loop.rs`, `src/display/tests.rs` (new), `scripts/` helper if needed.

- Add host-only tests that spin up the stub LCD driver, request icon-heavy frames, and assert we never exceed 8 CGRAM writes per update.
- Provide CLI/doc instructions for running `lifelinetty --test-lcd --demo-icons` (reusing the existing `--demo` flag under a submode) so hardware verifiers can watch icon swaps.
- Consider a small `scripts/gen_icon_table.rs` (dev-only) that regenerates `docs/icon_library.md` from the Rust registry to prevent drift.

### 5. Documentation + samples refresh

**Files:** `docs/icon_library.md`, `docs/lcd_patterns.md`, `docs/roadmap.md`, `README.md`, `samples/payload_examples.json`.

- Publish a Markdown catalog describing each icon (name, hex rows, screenshot) plus attribution to the upstream repo (public domain acknowledgement).
- Update `docs/lcd_patterns.md` with icon-heavy layout examples (battery meter, progress bars, arrow-based navigation) and note how the bank manager prioritizes them.
- Add new payload samples demonstrating both semantic names and inline bitmaps; include one failure-mode sample that intentionally exceeds 8 icons to illustrate fallback logging.

## Icon sources & licensing

The icon data originates from the Arduino `hd44780` library example **LCDCustomChars.ino** by Bill Perry (public domain). Milestone H imports the relevant glyph arrays into both `docs/icon_library.md` and the Rust registry while keeping the following requirements:

- Maintain attribution comments referencing the upstream commit hash and repository URL.
- Preserve the public-domain declaration verbatim so downstream redistributors understand the licensing provenance.
- Document any LifelineTTY-specific modifications (renamed icons, normalized bit order) inside the catalog.

## Acceptance checklist

1. `cargo test -p lifelinetty` exercises icon registry + CGRAM manager on both Linux (real driver) and host-mode stubs without regressions.
2. Icon registry + catalog stay synchronized via tests or generation script; reviewing the Markdown file shows the same byte rows the code will load.
3. Payload + config schema updates are documented and validated; invalid icon names produce actionable errors and never crash the daemon.
4. RAM usage stays within charter limits even during heavy icon churn (bank manager avoids unbounded allocations and logs swap rates for tuning).
5. README/docs/samples explain how to request built-in icons, override slots, and fall back to plain ASCII when custom glyphs are unavailable.

## Sample payloads

```json
{"schema_version":1,"line1":"TEMP\\x00","line2":"42C","icons":{"slot0":"thermometer","slot1":"degree_c"}}
{"schema_version":1,"line1":"NET","line2":"\\x00\\x01\\x02","icons":{"slot0":"signal_bar_2","slot1":"signal_bar_5","slot2":"alert_triangle"}}
```

## Out of scope

- Adding new CLI flags beyond existing `--demo` and config tables. Icon enablement piggybacks on payload/config data only.
- Supporting >8 concurrent glyphs; Milestone H enforces the HD44780 CGRAM limit and focuses on smart eviction rather than virtual banks.
- Network/file I/O for icon assets (all data ships with the binary or user config under `~/.serial_lcd`).

## Rollout plan

1. Land the icon registry + catalog plus CGRAM manager under feature flag `"icon-bank-preview"` (default on in debug/nightly builds).
2. Run `--test-lcd --demo` smoke loops on actual hardware for two weeks to validate flicker-free swaps before enabling in production builds.
3. Document upgrade steps in `docs/releasing.md`, including how to diff icon catalogs between releases and how to reset caches if glyph corruption is observed.
