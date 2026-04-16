# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- **Width-aware text wrapping**: `TextBuffer` now wraps text to the terminal width (auto-detected at each `flush()` call). Use `TextBuffer::with_width(n)` for a fixed width. Adapts to terminal resizes automatically.

## [0.1.0] - 2025-02-01

Initial release.

### Added

- **Tool execution formatting**: `format_tool_executing`, `format_tool_result`, `format_tool_args`, `format_error_detail` for consistent tool output display
- **Status formatters**: `format_context_warning`, `format_retry`, `format_error_message`, `format_ctrl_c`, `format_cancelled`
- **Token estimation**: `estimate_tokens` for rough token counts from JSON values
- **Streaming markdown**: `TextBuffer` for accumulating streaming text chunks with markdown rendering on flush
- **Logging infrastructure**: `OutputSink` trait with `log_event` / `log_event_line` for pluggable output destinations
- Comprehensive test suite: unit tests, ACP simulation tests, and PTY-based E2E tests
