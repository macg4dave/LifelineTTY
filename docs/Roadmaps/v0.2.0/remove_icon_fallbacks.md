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

Locations where fallbacks are currently implemented
--------------------------------------------------
- src/display/icon_bank.rs
  - BAR_FALLBACK_CHARS and HEARTBEAT_FALLBACK_CHAR provide ASCII fallbacks
  - missing_icons vector records icons which were not loaded into CGRAM
- src/display/overlays.rs
  - overlay_icons uses palette.icon_char(icon).or_else(|| icon.ascii_fallback())
    to silently use ascii characters when glyphs aren't available
- src/payload/icons.rs
  - Icon::ascii_fallback() returns an ASCII character for several icons
  - DisplayMode parsing may leave room for subtle fallback behavior in UI flows
- src/display/lcd.rs
  - `Lcd::new()` prints a warning and falls back to a stub driver when hardware
    init fails (this is convenient for development but hides OS/hardware issues)

Docs & tests referencing fallback behaviour
------------------------------------------
- README.md (mentions fallback behaviour for demo/wizard)
- docs/lcd_patterns.md (contains: 'falls back to ASCII when the eight-slot CGRAM budget is exceeded')
- docs/demo_playbook.md
- tests/ (unit tests in icon_bank.rs and overlays.rs that assert missing_icons behavior)

Immediate removal plan (alpha)
------------------------------
This repository is currently in alpha. There will be no transitional flag in this release —
we'll remove implicit ASCII fallbacks and silent driver/stub fallback on Linux immediately.

1. Remove the `Icon::ascii_fallback()` API and any codepaths that call it (e.g. overlays).
2. Make `IconPalette` return optional glyphs and ensure render paths do not substitute ASCII
  characters silently; renderers should either leave blank cells or record missing icons.
3. Stop silently falling back to a stub on Linux during `Lcd::new()` initialization — return
  an error so callers/operators observe hardware issues clearly.
4. Update unit & integration tests, fix expectations, and refresh docs and samples to match
  the new deterministic behavior.

Acceptance tests to add/update
------------------------------
- Unit tests that assert `Icon::ascii_fallback()` has been removed and render paths do not
  substitute ASCII characters when glyphs are missing; overlay logic should simply not place
  substitute characters in the display.
- Integration tests that run icon-heavy payloads and assert `missing_icons` is populated and
  that no ASCII substitutions are rendered on the display.
- Smoke tests that verify daemon behavior when glyphs are missing (e.g., visible logs, no
  silent substitution) and that hardware init failures on Linux surface as errors instead of
  silently falling back to a stub.

Deliverables
------------
- Updated `docs/Roadmaps/v0.2.0/roadmap.md` describing the plan and acceptance criteria.
- This file (developer note) listing files to change, tests to add, and a safe migration path.
- Follow-up PR(s) implementing the gated behavior and tests, then a removal PR after field trials.
