# v0.2.0 — Change log

Date: 2025-12-05

## Summary

This entry captures the changes made as part of the v0.2.0 milestone to remove implicit LCD icon/display fallback behavior. Because the project is still in alpha, the removal is immediate (no transitional flag). The goal is clearer failure modes, simpler testing, and deterministic render behaviour.

- ## Key changes

- Remove implicit ASCII fallback substitutions for icons and bar/heartbeat glyphs. Missing glyphs are now explicitly recorded (no silent replacement).
- Remove `Icon::ascii_fallback()` and all call-sites that used it.
- IconPalette getters now return `Option<char>` so render paths take explicit action when glyphs are absent.
- `overlay_icons` and heart/ bar rendering paths no longer substitute ASCII; they either leave blanks or use CGRAM glyphs when available.
- `Lcd::new()` on Linux now surfaces hardware/init errors instead of silently falling back to a stub display — this makes hardware initialization issues visible to operators and tests.

## Files touched (representative)

- src/display/icon_bank.rs — removed ASCII fallback constants and adjusted API to return optional glyphs
- src/display/overlays.rs — stopped substituting ASCII fallback characters and adjusted bar/heartbeat overlays
- src/payload/icons.rs — dropped `ascii_fallback()` API and adjusted tests
- src/display/lcd.rs — removed silent stub fallback on hardware init failure
- src/app/demo.rs — updated demo logging to reflect 'recorded missing glyphs' rather than ASCII substitution
- docs/** — README.md, docs/lcd_patterns.md, docs/demo_playbook.md, docs/icon_library.md updated to state missing requests are recorded and not automatically substituted

- ## Tests & verification

- Full unit test suite run locally after changes: all tests passed (145 passed, 2 ignored in the main suite; other test groups passed).
- New/updated tests were added or updated where necessary to remove assumptions about ASCII substitution.

## Why the immediate removal

Because this repository is in alpha, we remove the legacy fallback behaviour now to reduce surface area for bugs, simplify testing, and make render/hardware problems directly observable during field trials.

## Next steps

1. Run field trials (icon-heavy payloads) on a Pi, capture logs to `/run/serial_lcd_cache` and confirm operator-facing behavior.
2. Update any external demos or payload samples that implicitly relied on ASCII substitution; prefer explicit ASCII-only payloads if necessary.
3. Add any additional tests discovered during field trials and iterate on the render UX for missing glyphs if operators request a clearer visual fallback.
