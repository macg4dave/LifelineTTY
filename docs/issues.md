Payload format (most critical)
Send newline-terminated JSON matching the LCD schema. Minimal valid frame:
{"schema_version":1,"line1":"Hello","line2":"World"}
Allowed fields: line1, line2, schema_version, bar, bar_value, bar_max, bar_label, bar_line1, bar_line2, backlight, blink, scroll, scroll_speed_ms, duration_ms, page_timeout_ms, clear, test, mode, icons, checksum, config_reload.
Remove the type field—your sender is likely using a different schema (maybe a command/tunnel frame). Ensure each frame ends with \n, and avoid leading/trailing whitespace or mixed protocols.

Stop empty/garbage frames
The “expected value” errors mean the device is receiving blank lines or non-JSON bytes. Check the upstream producer for noise, stray CR-only lines, or partial writes.

Negotiation log permission
The daemon tries to write to negotiation.log. Make that path writable (requires root on most systems). For example, create logs with owner/perm that the service user can write to.

Serial device permission (if issues recur)
Ensure the running user has access to ttyS0 (e.g., add to dialout or adjust udev rules). Right now the connect succeeds, so this is secondary.

config.toml does not have server / client sections to store keys



