//! Text buffer for accumulating streaming text with markdown rendering.
//!
//! The [`TextBuffer`] collects text chunks from streaming responses and
//! renders them with markdown formatting when flushed. Text is wrapped
//! to the terminal width (auto-detected) or a caller-specified width.

use std::sync::LazyLock;
use termimad::MadSkin;

// ============================================================================
// Markdown Rendering
// ============================================================================

/// Termimad skin for markdown rendering. Left-aligns headers.
pub(crate) static SKIN: LazyLock<MadSkin> = LazyLock::new(|| {
    let mut skin = MadSkin::default();
    for h in &mut skin.headers {
        h.align = termimad::Alignment::Left;
    }
    skin
});

/// Default width when terminal size cannot be detected (e.g., piped output).
const DEFAULT_WIDTH: usize = 120;

/// Detect the current terminal width, falling back to [`DEFAULT_WIDTH`].
pub(crate) fn detect_terminal_width() -> usize {
    let (width, _) = termimad::terminal_size();
    let width = width as usize;
    if width == 0 { DEFAULT_WIDTH } else { width }
}

/// Render text with markdown formatting, wrapped to `width` columns.
pub(crate) fn render_markdown(text: &str, width: usize) -> String {
    use termimad::FmtText;
    FmtText::from(&SKIN, text, Some(width)).to_string()
}

// ============================================================================
// Text Buffer
// ============================================================================

/// Buffer for accumulating streaming text until event boundaries.
///
/// Text is buffered via `push()` during streaming, then flushed with markdown
/// rendering at logical boundaries (e.g., before tool execution, on completion).
///
/// By default, text is wrapped to the terminal width (auto-detected at each
/// `flush()` call, so it adapts to terminal resizes). Use [`TextBuffer::with_width`]
/// to force a specific width.
///
/// The `flush()` method normalizes trailing newlines to exactly `\n\n`.
///
/// # Example
///
/// ```
/// use clemitui::TextBuffer;
///
/// let mut buffer = TextBuffer::new();
/// buffer.push("Hello ");
/// buffer.push("world!");
///
/// let rendered = buffer.flush();
/// assert!(rendered.is_some());
/// assert!(rendered.unwrap().contains("Hello world!"));
///
/// // Buffer is now empty
/// assert!(buffer.is_empty());
/// ```
#[derive(Debug)]
pub struct TextBuffer {
    text: String,
    width: Option<usize>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    /// Create a new empty text buffer with auto-detected terminal width.
    ///
    /// Width is detected from the terminal at each `flush()` call, so it
    /// adapts to terminal resizes automatically.
    pub fn new() -> Self {
        Self {
            text: String::new(),
            width: None,
        }
    }

    /// Create a new empty text buffer with a fixed width.
    ///
    /// Text will be wrapped to exactly `width` columns on every flush,
    /// regardless of the actual terminal size.
    ///
    /// # Panics
    ///
    /// Panics if `width` is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use clemitui::TextBuffer;
    ///
    /// let mut buffer = TextBuffer::with_width(80);
    /// buffer.push("Hello world!");
    /// let rendered = buffer.flush();
    /// assert!(rendered.is_some());
    /// ```
    #[must_use]
    pub fn with_width(width: usize) -> Self {
        assert!(width > 0, "width must be positive");
        Self {
            text: String::new(),
            width: Some(width),
        }
    }

    /// Append text to the buffer.
    pub fn push(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// Flush buffered text with markdown rendering, normalized to `\n\n`.
    ///
    /// Width is resolved at flush time: either the fixed width from
    /// [`TextBuffer::with_width`], or the current terminal width.
    ///
    /// Returns rendered text, or None if buffer was empty or whitespace-only.
    pub fn flush(&mut self) -> Option<String> {
        if self.text.is_empty() {
            return None;
        }

        let text = std::mem::take(&mut self.text);
        let width = self.width.unwrap_or_else(detect_terminal_width);
        let rendered = render_markdown(&text, width);

        // Normalize trailing newlines to exactly \n\n
        let trimmed = rendered.trim_end_matches('\n');
        if trimmed.is_empty() {
            None
        } else {
            Some(format!("{}\n\n", trimmed))
        }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_accumulates() {
        let mut buffer = TextBuffer::new();

        // Buffer text chunks
        buffer.push("Hello ");
        buffer.push("world!");

        // Flush returns rendered content
        let out = buffer.flush();
        assert!(out.is_some());
        assert!(out.unwrap().contains("Hello world!"));

        // Buffer is now empty
        assert!(buffer.flush().is_none());
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_text_buffer_flush_empty() {
        let mut buffer = TextBuffer::new();
        assert!(buffer.is_empty());
        let out = buffer.flush();
        assert!(out.is_none());
    }

    #[test]
    fn test_text_buffer_flush_normalizes_to_double_newline() {
        // flush() should normalize output to end with exactly \n\n
        // This is critical for consistent spacing before tool calls

        // Case 1: Text with no trailing newline -> normalized to \n\n
        let mut buffer = TextBuffer::with_width(120);
        buffer.push("Hello world");
        let out = buffer.flush().unwrap();
        assert!(
            out.ends_with("\n\n"),
            "Should end with \\n\\n, got: {:?}",
            out
        );
        assert!(!out.ends_with("\n\n\n"), "Should not have triple newline");

        // Case 2: Text with single trailing newline -> normalized to \n\n
        let mut buffer = TextBuffer::with_width(120);
        buffer.push("Hello world\n");
        let out = buffer.flush().unwrap();
        assert!(
            out.ends_with("\n\n"),
            "Should end with \\n\\n, got: {:?}",
            out
        );

        // Case 3: Text with double trailing newline -> stays \n\n
        let mut buffer = TextBuffer::with_width(120);
        buffer.push("Hello world\n\n");
        let out = buffer.flush().unwrap();
        assert!(
            out.ends_with("\n\n"),
            "Should end with \\n\\n, got: {:?}",
            out
        );
        assert!(!out.ends_with("\n\n\n"), "Should not have triple newline");
    }

    #[test]
    fn test_text_buffer_flush_returns_none_for_whitespace_only() {
        // If buffer only contains whitespace/newlines, flush should return None
        let mut buffer = TextBuffer::with_width(120);
        buffer.push("\n\n");
        let out = buffer.flush();
        assert!(out.is_none(), "Whitespace-only buffer should return None");
    }

    #[test]
    fn test_text_buffer_default() {
        // TextBuffer implements Default
        let buffer = TextBuffer::default();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_text_buffer_with_width() {
        let buffer = TextBuffer::with_width(80);
        assert!(buffer.is_empty());
        assert_eq!(buffer.width, Some(80));
    }

    #[test]
    #[should_panic(expected = "width must be positive")]
    fn test_text_buffer_with_width_zero_panics() {
        let _ = TextBuffer::with_width(0);
    }

    #[test]
    fn test_render_markdown() {
        let rendered = render_markdown("**bold** and *italic*", 120);
        // Verify markdown is processed - should contain ANSI escape codes
        assert!(!rendered.is_empty());
        assert!(
            rendered.contains("\x1b["),
            "Expected ANSI codes for bold/italic formatting, got: {:?}",
            rendered
        );
        assert!(rendered.contains("bold"), "Should contain 'bold' text");
        assert!(rendered.contains("italic"), "Should contain 'italic' text");
    }

    #[test]
    fn test_render_markdown_plain_text() {
        let rendered = render_markdown("plain text", 120);
        assert!(rendered.contains("plain text"));
    }

    #[test]
    fn test_render_markdown_headers() {
        let rendered = render_markdown("# Header", 120);
        assert!(!rendered.is_empty());
        assert!(rendered.contains("Header"));
    }

    #[test]
    fn test_detect_terminal_width() {
        // Should return a non-zero width (either from terminal or fallback)
        let width = detect_terminal_width();
        assert!(width > 0, "Width should be positive");
    }

    // =========================================
    // Width-aware wrapping tests
    // =========================================

    #[test]
    fn test_wrapping_at_narrow_width() {
        // A long line should be wrapped when width is narrow
        let long_line = "This is a fairly long line that should definitely be wrapped when rendered at a narrow terminal width of forty columns.";
        let rendered = render_markdown(long_line, 40);
        let stripped = strip_ansi_for_test(&rendered);

        // The rendered output should contain multiple lines
        let lines: Vec<&str> = stripped.lines().collect();
        assert!(
            lines.len() > 1,
            "Long text should wrap at width 40, got {} line(s): {:?}",
            lines.len(),
            stripped
        );

        // No line should exceed the width
        for line in &lines {
            assert!(
                line.len() <= 40,
                "Line exceeds width 40 ({} chars): {:?}",
                line.len(),
                line
            );
        }
    }

    #[test]
    fn test_wrapping_preserves_content() {
        let text = "The quick brown fox jumps over the lazy dog. This sentence has enough words to wrap at various widths.";

        for width in [40, 60, 80, 120] {
            let rendered = render_markdown(text, width);
            let stripped = strip_ansi_for_test(&rendered);
            let joined = stripped
                .lines()
                .map(|l| l.trim_end())
                .collect::<Vec<_>>()
                .join(" ");

            assert!(
                joined.contains("quick brown fox"),
                "Content should be preserved at width {}: {:?}",
                width,
                stripped
            );
            assert!(
                joined.contains("lazy dog"),
                "Content should be preserved at width {}: {:?}",
                width,
                stripped
            );
        }
    }

    #[test]
    fn test_wrapping_at_different_widths() {
        let text = "This is a sentence that is long enough to be wrapped at eighty columns but not at one hundred and twenty columns for sure.";

        let narrow = render_markdown(text, 40);
        let wide = render_markdown(text, 200);

        let narrow_lines = strip_ansi_for_test(&narrow).lines().count();
        let wide_lines = strip_ansi_for_test(&wide).lines().count();

        assert!(
            narrow_lines > wide_lines,
            "Narrow width ({} lines) should produce more lines than wide ({} lines)",
            narrow_lines,
            wide_lines
        );
    }

    #[test]
    fn test_width_affects_flush_output() {
        let text = "This is a fairly long paragraph that should wrap differently depending on the configured width of the text buffer.";

        let mut narrow_buf = TextBuffer::with_width(40);
        narrow_buf.push(text);
        let narrow_out = narrow_buf.flush().unwrap();

        let mut wide_buf = TextBuffer::with_width(200);
        wide_buf.push(text);
        let wide_out = wide_buf.flush().unwrap();

        let narrow_lines = strip_ansi_for_test(&narrow_out).lines().count();
        let wide_lines = strip_ansi_for_test(&wide_out).lines().count();

        assert!(
            narrow_lines > wide_lines,
            "Narrow buffer ({} lines) should produce more lines than wide ({} lines)",
            narrow_lines,
            wide_lines
        );
    }

    #[test]
    fn test_markdown_code_block_at_narrow_width() {
        // Code blocks should not be word-wrapped (they preserve formatting)
        let text = "```\nfn main() { println!(\"hello\"); }\n```";
        let rendered = render_markdown(text, 40);
        let stripped = strip_ansi_for_test(&rendered);

        assert!(
            stripped.contains("fn main()"),
            "Code block content should be preserved: {:?}",
            stripped
        );
    }

    #[test]
    fn test_markdown_list_at_narrow_width() {
        let text = "- First item with some extra text that might wrap\n- Second item\n- Third item with even more text to test wrapping behavior at narrow widths";
        let rendered = render_markdown(text, 40);
        let stripped = strip_ansi_for_test(&rendered);

        assert!(stripped.contains("First item"), "Should contain first item");
        assert!(
            stripped.contains("Second item"),
            "Should contain second item"
        );
        assert!(stripped.contains("Third item"), "Should contain third item");
    }

    /// Simple ANSI stripping for unit tests (avoids dep on tests/common).
    fn strip_ansi_for_test(s: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                }
            } else {
                result.push(c);
            }
        }
        result
    }
}
