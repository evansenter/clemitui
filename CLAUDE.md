# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

clemitui is a standalone terminal UI library for ACP-compatible AI agents. It provides pure formatting functions, streaming markdown rendering, and a pluggable logging infrastructure. It's designed to take primitive types (strings, durations, token counts) so any agent can use it without model-specific dependencies.

## Build & Test

```bash
cargo check                                    # Fast type checking
cargo test                                     # All tests (unit + integration + doctests)
cargo nextest run                              # Parallel test runner (used in CI)
cargo clippy --all-targets -- -D warnings      # Lint with warnings as errors
cargo fmt -- --check                           # Check formatting
cargo doc --no-deps --document-private-items   # Build docs (RUSTDOCFLAGS="-D warnings" in CI)
```

No API keys or environment variables required. All tests are self-contained.

## Architecture

```
src/
├── lib.rs           # Re-exports
├── format.rs        # Pure formatting functions (tool output, warnings, errors)
├── logging.rs       # OutputSink trait, global logging infrastructure
├── text_buffer.rs   # TextBuffer for streaming markdown accumulation
└── bin/demo.rs      # Demo binary for E2E testing
```

### Design Principles

| Principle | Meaning |
|-----------|---------|
| **Pure rendering** | Format functions have no side effects or global state. Colors, I/O, and logging are caller concerns. |
| **Primitive types only** | Takes strings, durations, token counts -- not model-specific types. Any ACP agent can use this. |
| **Format helpers for all output** | All styled output uses `format_*` functions. No inline `.cyan()` or `.bold()` in business logic. |

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `format.rs` | Pure formatting: tool executing/result lines, args, warnings, errors |
| `logging.rs` | `OutputSink` trait, `set_output_sink()`, `log_event()` / `log_event_line()` |
| `text_buffer.rs` | `TextBuffer` for buffering streaming text and flushing with markdown rendering |
| `bin/demo.rs` | CLI exercising the public API, used by PTY-based E2E tests |

## Testing

Tests are organized in three layers:

| Layer | Location | Count | Purpose |
|-------|----------|-------|---------|
| Unit tests | `src/*.rs` | ~30 | Format function contracts, edge cases, logging behavior |
| ACP simulation | `tests/acp_simulation_tests.rs` | ~30 | Realistic agent workflow scenarios |
| E2E (PTY) | `tests/e2e_tests.rs` | ~15 | Terminal output verification via the demo binary |

Shared test helpers live in `tests/common/mod.rs` and include:
- `strip_ansi()` for comparing output without ANSI escape codes
- `DisableColors` RAII guard for deterministic format assertions
- `LoggingGuard` RAII guard to prevent test pollution of shared logging state
- `CaptureSink` mock for capturing `log_event` / `log_event_line` output

E2E tests gracefully skip if the demo binary isn't built.

## Conventions

- Rust 2024 edition, MSRV 1.88
- All public API items have doc comments with examples
- Doctests are executable and verified in CI
- Clippy with `-D warnings` (zero tolerance for warnings)
- `colored::control::set_override(false)` in unit tests that assert on formatted output

## Quality Gates (before pushing)

All of these must pass:
1. `cargo clippy --all-targets -- -D warnings`
2. `cargo fmt -- --check`
3. `cargo nextest run` (or `cargo test`)
4. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items`
