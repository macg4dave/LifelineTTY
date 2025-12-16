# LifelineTTY Copilot Charter
Purpose: keep every AI-assisted change aligned with the active roadmap under `docs/Roadmaps/` (current: `docs/Roadmaps/v0.2.0/roadmap.md`). All guidance here is binding—stay within scope, finish the blockers first, and move through priorities only when explicitly scheduled.

## One-line mission
Ship a single, ultra-light Rust daemon that reads newline-delimited JSON (and key=value fallbacks) from `/dev/ttyUSB0` at 9600 baud (8N1), renders to an HD44780 LCD via a PCF8574 I²C backpack, and runs for months without exceeding 5 MB RSS.

## Roadmap alignment (read before coding)
1. **Blockers (B1–B6)** — rename fallout, charter sync, cache-policy audit, CLI docs/tests, prompt refresh, and release tooling. Nothing else lands until these are closed.
2. **Priority queue (P1–P20)** — once blockers are done, tackle P1–P4 (rename lint, baud audit, config hardening, LCD regression tests) before touching telemetry, tunnels, or protocol work.
3. **Milestones (A–G)** — every large feature (command tunnel, negotiation, file push/pull, polling+heartbeat, display expansion, strict JSON+compression, serial shell) builds on specific priorities. Reference the milestone workflows in `docs/Roadmaps/v0.2.0/roadmap.md` (or the active version under `docs/Roadmaps/`) when planning.
4. Always annotate changes with the roadmap item they advance (e.g., “P3: Config loader hardening”) so we can trace progress.

## Core behavior (never change without approval)
- **IO**: UART input via `/dev/ttyUSB0` (9600 8N1) by default; config/CLI overrides may point to `/dev/ttyAMA0`, `/dev/ttyS*`, or USB adapters as long as they speak the same framing. LCD output via HD44780 + PCF8574 @ 0x27. No Wi-Fi, Bluetooth, sockets, HTTP, USB HID, or other transports.
- **CLI**: binary is invoked as `lifelinetty`. The supported CLI surface is whatever `lifelinetty --help` prints (implemented in `src/cli.rs`). Do **not** add new flags or modes unless the roadmap explicitly calls for it.
- **Protocols**: newline-terminated JSON or `key=value` pairs. JSON payloads must include `schema_version` (see `src/payload/parser.rs`). Exit code 0 on success, non-zero on fatal errors.
- **Display geometry**: Primary target is a 16×2 HD44780 LCD. Columns/rows are still configurable via config/CLI (defaults are 16×2); avoid hard-coding 20×4 assumptions.

## Storage + RAM-disk policy (mandatory)
- Persistent writes are limited to `~/.serial_lcd/config.toml`.
- Everything else (logs, temp payloads, LCD caches, tunnel buffers) must live under `/run/serial_lcd_cache`.
- The application must never call `mount`, create tmpfs, require sudo, or write outside the RAM disk.
- Hard-code `const CACHE_DIR: &str = "/run/serial_lcd_cache";` and treat it as ephemeral (clean up after yourself, expect wipe on reboot).
- All logging goes to stderr or files inside the RAM disk.

## Tech + dependencies
- **Language**: Rust 2021, `lifelinetty` crate only.
- **Allowed crates**: std, `hd44780-driver`, `linux-embedded-hal`, `rppal`, `serialport`, `tokio-serial` (feature `async-serial`), `tokio` (only for async serial), `serde`, `serde_json`, `crc32fast`, `ctrlc`, optional `anyhow`, `thiserror`, `log`, `tracing`, plus the roadmap-aligned helpers: `calloop`, `async-io`, `syslog-rs`, `os_info`, `crossbeam`, `rustix`, `sysinfo`, `futures`, `directories`, `humantime`, `serde_bytes`, `bincode`, `clap_complete`, `indicatif`, and `tokio-util`. If a crate already exists for the function you need, add it from this list and use it rather than re-implementing the feature. Do **not** remove existing crate dependencies without explicit permission. Every new crate that lands must be documented in `docs/lifelinetty_creates.md`; treat that file as a living reference, not a limit on future roadmap-approved crates.
- **Banned crates**: anything pulling in a network stack, heavyweight runtime, database, or filesystem abstraction that writes outside allowed paths.
- **Build/test commands**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --release` when needed. All must pass on x86_64 **and** ARMv6.

## Interfaces that must stay stable
- CLI name + flags: `lifelinetty` binary with documented flags.
- Config schema at `~/.serial_lcd/config.toml` and payload contracts in `src/payload/`.
- LCD command set (HD44780) and I²C wiring (PCF8574 @ 0x27).
- Serial framing: newline JSON / key=value.

## Quality bar & testing
- Every behavioral change gets matching tests (unit + integration under `tests/`). All CLI flags must have regression coverage.
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and the relevant `cargo test` targets before submitting. Include full output in reviews/PRs.
- Avoid `unsafe` and unchecked `unwrap()` in production code.
- Maintain <5 MB RSS, no busy loops. Add backoff/retry handling for serial and LCD errors.
- Document user-facing changes (README, docs/*.md, Rustdoc). Public functions and types require Rustdoc comments.
- Never silence lints globally (`#[allow(dead_code)]`, etc.) without explicit approval and clear justification.

## Task request template (use verbatim)
Task:
"""
<One-line summary of the change>

Details:
- Roadmap link: <B#/P#/Milestone reference>
- What to change: <short description + acceptance criteria>
- Files to consider: <list or leave blank>
- Tests: <which tests to add/update or leave blank>
- Constraints / do not modify: <guardrails>
"""

## Agent rules (apply to every change)
1. If the request conflicts with this charter or the roadmap, clarify before coding.
2. Make the smallest change that satisfies the acceptance criteria and roadmap intent.
3. Preserve stable interfaces unless the roadmap explicitly authorizes modifications (and provide migration notes when it does).
4. Update tests, docs, and roadmap cross-references together in the same PR.
5. Include `cargo test` output and note any platform-specific considerations (x86_64 vs ARMv6).
6. Resist feature creep—no speculative refactors or new capabilities beyond the roadmap milestones.

## Development + review environment
- Target client hardware: Raspberry Pi 1 Model A (ARMv6, Debian/systemd). Target server hardware: crossplatform Linux (Debian/systemd). Cross-compile or use QEMU/docker images in `docker/` and `scripts/local-release.sh` for packaging.
- Services: `lifelinetty.service` must remain systemd-friendly (no extra daemons).

## Documentation expectations
- Keep README, `docs/architecture.md`, `docs/Roadmaps/[VERSION]/roadmap.md`, `docs/lcd_patterns.md`, and `samples/` payloads synchronized with functionality.
- When adding protocol/CLI changes, update `spec.txt` (or create it if missing) and annotate roadmap items with the new state.
- Comment non-obvious state machines (render loop, serial backoff, payload parser) so future contributors can reason about them.
- All doc updates must ship with their accompanying code changes.

# Bash Scripting Rules for This Repository

The project is primarily Rust, but occasionally uses Bash scripts for tests and tooling.
Whenever the user requests Bash, follow these strict rules:

1. Shell: bash only. Do not use sh, zsh, or POSIX-generic syntax.
2. Platform: Linux only (Fedora/Debian). Avoid macOS-specific commands or flags.
3. Output must be real, runnable Bash. Do not invent flags or commands.
4. Start scripts with:
       set -euo pipefail
5. Use strict quoting: "$var"
6. Reading files: use 'while IFS= read -r line'; avoid 'for x in $(...)'.
7. For paths: prefer explicit variables or absolute paths. Avoid guessing.
8. Error output: use >&2 and non-zero exit codes.
9. No pseudo-code blocks and no wrapping everything in functions unless asked.
10. Keep scripts minimal, predictable, and aligned with normal Linux tooling
    (cp, rsync, scp, ssh, grep, sed, awk, rg, jq).
12. Never change the project layout or add new files unless requested.
13. Respect cross-env scripting: scripts may run locally or on a remote Pi via SSH.
14. Use shellcheck annotations for exceptions.


Always prioritise stability and compatibility over cleverness. 