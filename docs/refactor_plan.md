# Modularization Plan for seriallcd (for gpt5.1)

## Objectives

- Restructure code by function into directories, keeping each module focused on a single responsibility.
- Rename variables, structs, and functions to be human readable and intention revealing.
- Add a short block-level comment above every non-trivial code block (loops, branches, helper groups) to explain purpose, not mechanics.
- Identify duplicated logic and consolidate it into shared helpers instead of repeating it across modules.
- Preserve existing behaviour, tests, and CLI surface while making the code easier to navigate and extend.

## Target Layout (proposed)

- `src/main.rs`: ultra-thin entry that only parses CLI and invokes an `AppRunner`.
- `src/app/`: orchestration layer.
  - `mod.rs`: exports `AppRunner` and shared types.
  - `lifecycle.rs`: startup/shutdown wiring, ctrlc handler, boot/shutdown display hooks.
  - `connection.rs`: serial connect/reconnect with backoff and logging.
  - `render_loop.rs`: main loop that coordinates pages, scrolling, heartbeat, and button input.
  - `events.rs`: small structs/enums for events (button press, new frame, reconnect, heartbeat tick).
- `src/display/`:
  - `mod.rs`: public facade returning a trait for rendering.
  - `lcd.rs`: current `Lcd` wrapper moved here.
  - `overlays.rs`: helpers like bar rendering, heartbeat/icon overlay, scroll windowing.
- `src/payload/`:
  - Keep parsing in `parser.rs`; keep `RenderFrame` building there.
  - Move icon parsing and display mode helpers into `icons.rs`.
- `src/serial/`:
  - `mod.rs`: re-export sync/async implementations.
  - `sync.rs`: current `serial.rs` content, with clearer names (e.g., `SerialConnection`, `read_next_line`).
  - `async.rs`: current `serial_async.rs`.
  - `backoff.rs`: encapsulate reconnect/backoff math and timers.
- `src/config/`:
  - Split into `mod.rs` (public API) and `loader.rs` (file I/O + parsing) to isolate filesystem work.
- `src/lcd_driver/`: keep as-is but adjust imports after moving `lcd.rs` into `display/`.
- `tests/`: update integration tests to target new modules; keep unit tests colocated.

## Execution Steps

1. Create directory skeletons (`app`, `display`, `payload`, `serial`, `config`) and move existing files into the closest match without altering logic; fix `mod` declarations and imports.
2. Slim `main.rs` so it only handles CLI parsing and forwards to an `AppRunner::run` entry in `app::mod`.
3. Split `app.rs` into the new app submodules:
   - Move serial connect/retry/backoff logic into `app/connection.rs` using a struct like `SerialReconnectStrategy` with human-readable field names.
   - Extract the main `while` loop into `app/render_loop.rs` with small helpers for heartbeat, paging, scrolling, and blink/backlight toggles.
   - Isolate ctrlc handler and boot/shutdown rendering into `app/lifecycle.rs`.
   - Replace ad-hoc tuples like `scroll_offsets` with structs such as `ScrollOffsets { top, bottom }`.
4. Move rendering helpers (`render_frame_with_scroll`, `render_bar`, `overlay_*`, scrolling helpers) into `display/overlays.rs`; keep `Lcd` facade in `display/lcd.rs` and adjust public exports through `display::Display`.
5. As files move, scan for duplicated helpers (scroll windows, heartbeat overlay, checksum/backoff math, button handling) and consolidate them into single reusable modules.
6. In `payload.rs`, split into `parser.rs` (serde structs, checksum, `RenderFrame` builder) and `icons.rs` (icon mapping, display mode helpers). Keep defaults in `mod.rs`.
7. In `serial.rs`, rename to `serial/sync.rs` and rename methods to descriptive verbs (`send_command_line`, `read_message_line`, `borrow_reader`). Add `backoff.rs` for exponential backoff parameters and timers reused by the connection module.
8. In `config.rs`, move filesystem paths and TOML parsing into `config/loader.rs`; keep defaults and types in `config/mod.rs`. Ensure error messages remain stable.
9. Standardize naming for readability across modules (examples: `buf` -> `incoming_line`, `lcd` -> `display`, `port` -> `serial_connection`, `now` -> `current_time`, `btn` -> `button_input`).
10. Add concise block comments before complex logic (render loop stages, reconnect decisions, scroll calculations, checksum validation) explaining intent and edge cases; avoid restating code.
11. Run unit tests and any integration tests; fix borrow/import issues from the move; keep behaviour unchanged. Update docs/examples if public APIs move.

## Notes for Implementation

- Maintain backward-compatible CLI flags and config file format.
- Keep feature flag `async-serial` working after the move.
- Preserve Linux vs non-Linux conditional code paths when relocating files.
- Prefer small pure functions over inline logic in the main loop to keep files short and readable.
