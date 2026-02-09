# clemitui

Terminal UI library for [ACP](https://github.com/anthropics/acp)-compatible AI agents.

clemitui provides formatting and logging primitives for AI coding assistants that speak the Agent Client Protocol. It handles streaming text rendering with markdown support, tool execution display, and session logging infrastructure.

## Installation

```toml
[dependencies]
clemitui = "0.1"
```

Requires Rust 1.88+.

## Usage

### Tool execution display

Format tool start/result lines with durations and token counts:

```rust
use clemitui::{format_tool_executing, format_tool_result};
use serde_json::json;
use std::time::Duration;

// ┌─ read_file path="src/main.rs"
let start = format_tool_executing("read_file", &json!({"path": "src/main.rs"}));

// └─ read_file 0.25s ~100 tok
let result = format_tool_result("read_file", Duration::from_millis(250), 100, false);
```

### Streaming text with markdown

Buffer streaming text chunks and render with markdown formatting on flush:

```rust
use clemitui::TextBuffer;

// Auto-detects terminal width at each flush (adapts to resizes)
let mut buffer = TextBuffer::new();
buffer.push("Here's the **fix**:\n\n");
buffer.push("```rust\nfn main() {}\n```");

if let Some(rendered) = buffer.flush() {
    print!("{}", rendered);
}

// Or use a fixed width
let mut fixed = TextBuffer::with_width(80);
fixed.push("Text wrapped to exactly 80 columns.");
```

### Logging infrastructure

Plug in your own output sink to control where formatted output goes:

```rust
use clemitui::{OutputSink, set_output_sink, log_event, log_event_line};
use std::sync::Arc;

struct StdoutSink;

impl OutputSink for StdoutSink {
    fn emit(&self, message: &str) {
        println!("{}\n", message);
    }
    fn emit_line(&self, message: &str) {
        println!("{}", message);
    }
}

set_output_sink(Arc::new(StdoutSink));
log_event("Tool completed successfully");
```

## Design

clemitui takes primitive types (strings, durations, token counts) rather than model-specific types. This keeps it usable by any ACP-compatible agent without coupling to a particular AI SDK.

All formatting functions are pure -- no side effects, no global state. Color output, file I/O, and logging happen in callers, not formatters.

## API

| Function | Purpose |
|----------|---------|
| `format_tool_executing` | Tool start line (`┌─ name args`) |
| `format_tool_result` | Tool completion line (`└─ name 0.25s ~100 tok`) |
| `format_tool_args` | Format arguments as `key=value` pairs |
| `format_error_detail` | Indented error detail line |
| `format_context_warning` | Context window usage warning |
| `format_retry` | API retry message |
| `format_error_message` | Red error text |
| `format_ctrl_c` | Ctrl-C received message |
| `format_cancelled` | Task cancelled message |
| `estimate_tokens` | Rough token count from JSON value |
| `TextBuffer` | Streaming markdown text accumulator |
| `OutputSink` | Trait for pluggable output destinations |
| `log_event` / `log_event_line` | Global logging through the configured sink |

## License

MIT
