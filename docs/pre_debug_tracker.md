Pre-Debugging Improvement Tracker
=================================

Goal: finish polish items before full debug pass. Each section lists rationale, tasks, and test ideas.

Serial I/O Hardening
--------------------
- Rationale: make reconnects predictable and visible; avoid silent stalls.
- Tasks:
  - Add backoff jitter and cap retries with a “cooldown” log line when saturated.
  - Raise serial timeouts slightly; ensure read/write errors surface distinct messages.
  - Render a clear “SERIAL OFFLINE” page (blink/backlight) when the port drops and clear it on recover.
  - Log every reconnect attempt with timestamp and backoff delay.
- Tests: fake serial transport that forces timeouts/errors; assert reconnect path renders offline page and logs retries.

Input Validation & Errors
-------------------------
- Rationale: fast feedback for payload producers and LCD safety.
- Tasks:
  - Enforce LCD width/height: truncate or ellipsis long lines before render; guard custom glyph placeholders.
  - Improve parse errors with field names and limits (e.g., “bar_value must be <= bar_max”).
  - Add stricter checks on bar_max >= 1 and page_timeout_ms > 0 with clear messages.
- Tests: golden error strings per field; line-length clamp test for 16x2 and 20x4; placeholder parsing bounds test.

Observability
-------------
- Rationale: easier repro and field diagnostics.
- Tasks:
  - Add `--log-level` and optional `--log-file` (env overrides ok); default INFO.
  - Track counters (frames accepted/rejected, checksum failures, reconnects) and print summary on shutdown.
  - Include frame CRC or hash in debug logs to correlate duplicates.
- Tests: CLI parsing for new flags; unit test that counters increment on parse errors and dump on shutdown hook.

Config Lifecycle
----------------
- Rationale: predictable runtime changes.
- Tasks:
  - On `config_reload`, refresh scroll/page timeouts, backoff settings, and (when safe) device/baud with a reconnect.
  - Document which fields are reloadable vs require restart.
  - Emit a log line for reload success/failure with diff of changed fields.
- Tests: mock config file swap; assert defaults update and reconnect scheduled when device/baud change.

Tests & Simulation
------------------
- Rationale: catch regressions without hardware.
- Tasks:
  - Build a fake serial transport to inject frames, timeouts, and disconnects.
  - Integration tests for: reconnect loop, TTL expiry, scroll advancement, blink/backlight toggling, dashboard bar row placement, and icon placeholder rendering.
  - Golden tests for banner mode clearing line2 and for checksum mismatch handling.
- Tests: run under `cargo test` gated; ensure fake transport is the default for tests to avoid real hardware.

Developer Tooling
-----------------
- Rationale: faster feedback cycle.
- Tasks:
  - Add `make lint` (fmt + clippy) and ensure CI uses it.
  - README: short “sending test payloads” section with PTY feeder + checksum example; note 512-byte limit.
  - Optional: `justfile` or `make` targets for “run with sample payload file” and “spawn PTYs + feeder”.
- Tests: CI jobs green; sample commands verified locally.
