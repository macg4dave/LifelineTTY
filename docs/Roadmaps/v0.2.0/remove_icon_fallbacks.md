Roadmap: Remove implicit LCD icon/display fallbacks
=================================================

Purpose
-------

This note lists where the codebase implements fallbacks for LCD icons and display types and proposes a small plan to remove the implicit behavior safely.

Why remove
----------

- Fallbacks mask real failures in the rendering or glyph-loading path.
- They complicate deterministic tests and field trial verification.
- Removing them leads to clearer errors, simpler logic, and easier acceptance tests.

Current state (fallbacks removed)
---------------------------------

- `src/display/icon_bank.rs`: palettes return optional glyphs without ASCII substitutes; missing glyphs are recorded in `missing_icons` for callers to observe.
- `src/display/overlays.rs`: icon overlays rely solely on palette entries and leave cells untouched when glyphs are absent (no `ascii_fallback` path).
- `src/payload/icons.rs`: `Icon::ascii_fallback()` has been removed; icons expose only bitmaps and optional parsing helpers.
- `src/display/lcd.rs`: on Linux, `Lcd::new()` now fails fast instead of silently falling back to a stub when hardware init fails. Non-Linux platforms still use the stub-only path for host-mode tests.

Docs & tests referencing fallback behaviour
------------------------------------------

- README.md (mentions fallback behaviour for demo/wizard)
- docs/lcd_patterns.md (keep visuals aligned with the no-fallback behavior)
- docs/demo_playbook.md
- tests/ (unit tests in icon_bank.rs and overlays.rs that assert missing_icons behavior)

Removal status and guardrails (alpha)
-------------------------------------

- Fallback APIs and code paths have been removed; keep regression coverage in place.
- Renderers must leave cells blank or rely on explicit `missing_icons` reporting rather than substituting ASCII characters.
- Linux builds should surface LCD init failures as errors; stub behavior is for non-Linux host-mode tests only.

Regression checks to keep warm
------------------------------

- Unit coverage that overlays do not substitute ASCII when glyphs are absent (see `overlay_icons_does_not_substitute_when_missing`).
- Integration or smoke tests for icon-heavy payloads should assert `missing_icons` reporting without rendered ASCII substitutions.
- Linux initialization errors must bubble up (no silent stub fallback); host-mode tests continue to use the stub constructor explicitly.

Deliverables
------------

- Updated `docs/Roadmaps/v0.2.0/roadmap.md` describing the plan and acceptance criteria.
- This file (developer note) listing files to change, tests to add, and a safe migration path.
- Follow-up PR(s) implementing the gated behavior and tests, then a removal PR after field trials.
