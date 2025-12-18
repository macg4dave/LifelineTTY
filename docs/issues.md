Known Issues (v0.2.0)
=====================

This doc tracks the current operational issues and workarounds. Keep updates scoped to the active roadmap (v0.2.0) and the charter guardrails.

Issue log
---------

| ID | Title | Symptoms | Workaround / Notes | Status |
| --- | ----- | -------- | ------------------ | ------ |
| I1 | Payload format rejections | `expected value` parse errors; LCD shows parse error; cache logs show malformed JSON. | Send newline-terminated JSON matching the LCD payload schema (e.g., `{ "schema_version":1,"line1":"Hello","line2":"World" }`). Allowed fields: `schema_version`, `line1`, `line2`, `bar`, `bar_value`, `bar_max`, `bar_label`, `bar_line1`, `bar_line2`, `backlight`, `blink`, `scroll`, `scroll_speed_ms`, `duration_ms`, `page_timeout_ms`, `clear`, `test`, `mode`, `icons`, `checksum`, `config_reload`. Frames may include an extra top-level `type` field (it is tolerated/ignored by the payload parser), but **do not** mix in non-payload frames (tunnel/command frames) on the same channel. Ensure each frame ends with `\n`; CRLF is fine. For debugging, `/run/serial_lcd_cache/protocol_errors.log` records JSON-lines with a short `preview`, frame `len`, and a `crc32` to help correlate bad frames back to the producer (regression: `src/app/render_loop.rs` test `protocol_error_log_records_len_crc32_preview_and_payload`). | Mitigated |
| I2 | Garbage/blank frames from producer | Daemon logs show parse errors; LCD intermittently clears; integration mock passes. | The daemon ignores blank lines and obvious non-payload chatter (e.g., `INIT`, non-JSON / non-`key=value` frames). If you still see parse errors, your producer is likely sending *valid UTF-8* that isn't a JSON object or `key=value` payload, or it's sending truncated/malformed JSON. Enforce full line writes ending in `\n` and flush after each line. | Mitigated |
| I3 | Negotiation log permission | `negotiation.log` fails to open/write under certain users; warnings in stderr. | Negotiation logging is best-effort: the daemon will continue if the log can't be created. The log path is `/run/serial_lcd_cache/logs/negotiation.log`; ensure `/run/serial_lcd_cache` (and `logs/`) is writable by the service user (ownership/permissions), and keep logs inside cache per charter. | Mitigated |
| I4 | Serial device permission | Serial connect fails when user lacks access to the TTY; may see `Permission denied` or silent open failures. | Add the service user to `dialout` (or matching group) or adjust udev rules; keep default device `/dev/ttyUSB0` unless overridden. Verify with `ls -l /dev/tty*` before startup. The daemon logs `permission_denied` failures with an explicit dialout/udev hint (regression: `src/app/connection.rs` test `connect_failure_hint_only_for_permission_denied`). | Mitigated |
| I5 | Missing server/client sections in config | `config.toml` lacks explicit server/client key storage; operators unsure where to place credentials. | Current v0.2.0 config schema does not include any server/client credential sections. Only global/runtime settings are supported; do not attempt to add keys to `~/.serial_lcd/config.toml` until a roadmap item explicitly adds schema support. | Mitigated |

How to update this list
-----------------------

- Scope: only runtime issues that fit the v0.2.0 roadmap and charter (single daemon, UART+LCD only, cache rules enforced).
- For each new issue, add an ID, concise title, reproducible symptoms, workaround/notes, and status (`Open`, `Mitigated`, or `Resolved`).
- Link to regression tests when added, and ensure any new logs stay under `/run/serial_lcd_cache`.
