//! # artbox
//!
//! A Rust library for rendering FIGlet/ASCII art text into bounded rectangles.
//!
//! artbox provides a simple API for rendering text using FIGlet fonts with automatic
//! font selection based on available space, alignment options, and letter spacing control.
//!
//! ## Quick Start
//!
//! ```rust
//! use artbox::{render, Renderer, Alignment};
//!
//! // Simple rendering with defaults
//! let result = render("Hello", 40, 10).unwrap();
//! println!("{}", result.text);
//!
//! // Custom renderer with alignment
//! let renderer = Renderer::default()
//!     .with_alignment(Alignment::Center);
//! let result = renderer.render("Hi", 20, 5).unwrap();
//! ```
//!
//! ## Font Selection
//!
//! The renderer tries fonts in order until one fits within the specified bounds.
//! The default font stack progresses from large to small: `big` → `standard` → `small` → `mini`.
//!
//! ```rust
//! use artbox::{Renderer, fonts};
//!
//! // Use a specific font family
//! let renderer = Renderer::new(fonts::family("cyber").unwrap());
//!
//! // Or build a custom stack
//! let renderer = Renderer::new(fonts::stack(&["slant", "small_slant"]));
//! ```
//!
//! ## Features
//!
//! - **`ratatui`** - Enables the [`integrations::ratatui::ArtBox`] widget for TUI applications.

use std::sync::Arc;

use figlet_rs::FIGfont;
use unicode_width::UnicodeWidthStr;

pub mod fonts;
pub mod integrations;

/// A font that can be used to render text as ASCII art.
///
/// Fonts can be loaded from FIGlet font files (`.flf`) or created as plain text.
/// Multiple fonts can be combined into a stack for automatic fallback rendering.
///
/// # Examples
///
/// ```rust
/// use artbox::Font;
///
/// // Load from embedded fonts
/// let font = artbox::fonts::font("slant").unwrap();
///
/// // Create a plain text font (no ASCII art)
/// let plain = Font::plain();
/// ```
#[derive(Clone)]
pub struct Font {
    kind: FontKind,
}

#[derive(Clone)]
enum FontKind {
    Figlet(Arc<FIGfont>),
    Plain,
}

impl Font {
    /// Creates a font from a parsed FIGfont.
    ///
    /// This is a low-level constructor. Prefer [`Font::from_file`], [`Font::from_content`],
    /// or the [`fonts`] module for loading fonts.
    pub fn figlet(font: FIGfont) -> Self {
        Self {
            kind: FontKind::Figlet(Arc::new(font)),
        }
    }

    /// Loads a font from a FIGlet font file (`.flf`).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains invalid FIGlet data.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use artbox::Font;
    ///
    /// let font = Font::from_file("/path/to/font.flf")?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = std::fs::read(path).map_err(|e| format!("{e:?}"))?;
        Self::from_bytes_latin1(&contents)
    }

    /// Creates a font from FIGlet font content as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the content is not valid FIGlet format.
    pub fn from_content(contents: &str) -> Result<Self, String> {
        parse_figlet_content(contents).map(Self::figlet)
    }

    /// Creates a font from raw bytes encoded as Latin-1.
    ///
    /// This is useful for loading embedded font data. Each byte is interpreted
    /// as a Latin-1 character code point.
    ///
    /// # Errors
    ///
    /// Returns an error if the content is not valid FIGlet format.
    pub fn from_bytes_latin1(bytes: &[u8]) -> Result<Self, String> {
        let contents = latin1_to_string(bytes);
        parse_figlet_content(&contents).map(Self::figlet)
    }

    /// Loads the standard FIGlet font bundled with `figlet-rs`.
    ///
    /// Returns `None` if the standard font cannot be loaded.
    pub fn standard() -> Option<Self> {
        FIGfont::standard().ok().map(Self::figlet)
    }

    /// Creates a plain text font that renders text without ASCII art styling.
    ///
    /// This is useful as a fallback when FIGlet fonts are too large for the bounds.
    pub fn plain() -> Self {
        Self {
            kind: FontKind::Plain,
        }
    }

    fn is_plain(&self) -> bool {
        matches!(self.kind, FontKind::Plain)
    }

    fn render_with_spacing(&self, content: &str, letter_spacing: i16) -> Option<String> {
        match &self.kind {
            FontKind::Figlet(font) => {
                if letter_spacing == 0 {
                    font.convert(content).map(|figure| figure.to_string())
                } else {
                    render_figlet_with_spacing(font, content, letter_spacing)
                }
            }
            FontKind::Plain => Some(apply_letter_spacing_plain(content, letter_spacing)),
        }
    }
}

/// Specifies how rendered text is aligned within the bounding box.
///
/// Alignment is a combination of horizontal (left, center, right) and
/// vertical (top, middle, bottom) positioning.
///
/// # Examples
///
/// ```rust
/// use artbox::{Renderer, Alignment};
///
/// let renderer = Renderer::default()
///     .with_alignment(Alignment::Center);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// Align to top-left corner.
    TopLeft,
    /// Align to top edge, horizontally centered.
    Top,
    /// Align to top-right corner.
    TopRight,
    /// Align to left edge, vertically centered.
    Left,
    /// Center both horizontally and vertically.
    Center,
    /// Align to right edge, vertically centered.
    Right,
    /// Align to bottom-left corner.
    BottomLeft,
    /// Align to bottom edge, horizontally centered.
    Bottom,
    /// Align to bottom-right corner.
    BottomRight,
}

/// Renders text as ASCII art within specified bounds.
///
/// The renderer maintains a stack of fonts and tries each in order until one fits
/// within the specified dimensions. Configuration options include alignment and
/// letter spacing.
///
/// # Examples
///
/// ```rust
/// use artbox::{Renderer, Alignment, fonts};
///
/// // Default renderer with built-in font stack
/// let renderer = Renderer::default();
///
/// // Custom configuration
/// let renderer = Renderer::new(fonts::family("cyber").unwrap())
///     .with_alignment(Alignment::Center)
///     .with_letter_spacing(-1)
///     .with_plain_fallback();
///
/// let result = renderer.render("Hello", 40, 10)?;
/// # Ok::<(), artbox::RenderError>(())
/// ```
#[derive(Clone)]
pub struct Renderer {
    fonts: Vec<Font>,
    alignment: Alignment,
    letter_spacing: i16,
}

impl Renderer {
    /// Creates a new renderer with the specified font stack.
    ///
    /// Fonts are tried in order during rendering. The first font whose output
    /// fits within the bounds is used.
    pub fn new(fonts: Vec<Font>) -> Self {
        Self {
            fonts,
            alignment: Alignment::TopLeft,
            letter_spacing: 0,
        }
    }

    /// Adds a plain text fallback font if one doesn't already exist.
    ///
    /// This ensures rendering always succeeds (assuming bounds allow at least
    /// the plain text to fit) by falling back to unstyled text.
    pub fn with_plain_fallback(mut self) -> Self {
        if !self.fonts.iter().any(|font| font.is_plain()) {
            self.fonts.push(Font::plain());
        }
        self
    }

    /// Sets the alignment for rendered text within the bounding box.
    ///
    /// Default is [`Alignment::TopLeft`].
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets the letter spacing adjustment.
    ///
    /// Positive values add space between characters. Negative values create
    /// overlap, which can produce interesting visual effects with some fonts.
    ///
    /// Default is `0` (normal spacing).
    pub fn with_letter_spacing(mut self, letter_spacing: i16) -> Self {
        self.letter_spacing = letter_spacing;
        self
    }

    /// Renders text into a new string buffer.
    ///
    /// Returns the rendered text with alignment padding applied, along with
    /// metrics about the content dimensions and which font was used.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `width` or `height` is zero ([`RenderError::EmptyBounds`])
    /// - No fonts are configured ([`RenderError::EmptyFonts`])
    /// - No font produces output that fits ([`RenderError::NoFit`])
    pub fn render(&self, text: &str, width: u16, height: u16) -> Result<Rendered, RenderError> {
        let mut buffer = String::new();
        let metrics = self.render_into(text, width, height, &mut buffer)?;
        Ok(Rendered {
            text: buffer,
            width: metrics.width,
            height: metrics.height,
            font_index: metrics.font_index,
        })
    }

    /// Renders text into an existing string buffer.
    ///
    /// This is useful in hot render loops to avoid repeated allocations.
    /// The buffer is cleared before rendering.
    ///
    /// # Errors
    ///
    /// Same error conditions as [`Renderer::render`].
    pub fn render_into(
        &self,
        text: &str,
        width: u16,
        height: u16,
        out: &mut String,
    ) -> Result<RenderMetrics, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::EmptyBounds);
        }

        if self.fonts.is_empty() {
            return Err(RenderError::EmptyFonts);
        }

        for (index, font) in self.fonts.iter().enumerate() {
            if let Some(rendered) = font.render_with_spacing(text, self.letter_spacing) {
                let (content_width, content_height) = measure_rendered(&rendered);
                if content_width <= width as usize && content_height <= height as usize {
                    align_rendered_into(&rendered, width, height, self.alignment, out);
                    return Ok(RenderMetrics {
                        width: content_width as u16,
                        height: content_height as u16,
                        font_index: index,
                    });
                }
            }
        }

        Err(RenderError::NoFit)
    }
}

impl Default for Renderer {
    /// Creates a renderer with the default font stack (`big`, `standard`, `small`, `mini`)
    /// and a plain text fallback.
    fn default() -> Self {
        Self::new(fonts::default()).with_plain_fallback()
    }
}

/// Renders text using the default renderer.
///
/// This is a convenience function equivalent to `Renderer::default().render(text, width, height)`.
///
/// # Examples
///
/// ```rust
/// let result = artbox::render("Hi", 20, 5)?;
/// println!("{}", result.text);
/// # Ok::<(), artbox::RenderError>(())
/// ```
pub fn render(text: &str, width: u16, height: u16) -> Result<Rendered, RenderError> {
    Renderer::default().render(text, width, height)
}

/// The result of a successful render operation.
///
/// Contains the rendered text with alignment padding applied, along with
/// metrics about the original content dimensions.
#[derive(Debug, Clone)]
pub struct Rendered {
    /// The rendered ASCII art text with alignment padding.
    pub text: String,
    /// Width of the content before alignment padding was applied.
    pub width: u16,
    /// Height of the content before alignment padding was applied.
    pub height: u16,
    /// Index of the font that was used from the renderer's font stack.
    pub font_index: usize,
}

impl Rendered {
    /// Extracts just the metrics from this rendered result.
    pub fn metrics(&self) -> RenderMetrics {
        RenderMetrics {
            width: self.width,
            height: self.height,
            font_index: self.font_index,
        }
    }
}

/// Metrics about a rendered result without the text content.
///
/// Useful when you only need dimension information, such as with
/// [`Renderer::render_into`] which returns metrics separately from the buffer.
#[derive(Debug, Clone, Copy)]
pub struct RenderMetrics {
    /// Width of the rendered content (before alignment padding).
    pub width: u16,
    /// Height of the rendered content (before alignment padding).
    pub height: u16,
    /// Index of the font that was used from the renderer's font stack.
    pub font_index: usize,
}

/// Errors that can occur during rendering.
#[derive(Debug, Clone)]
pub enum RenderError {
    /// The specified width or height was zero.
    EmptyBounds,
    /// No fonts were configured in the renderer.
    EmptyFonts,
    /// No font in the stack produced output that fits within the bounds.
    NoFit,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::EmptyBounds => write!(f, "width and height must be non-zero"),
            RenderError::EmptyFonts => write!(f, "no fonts provided"),
            RenderError::NoFit => write!(f, "no font fits within the requested bounds"),
        }
    }
}

impl std::error::Error for RenderError {}

fn render_figlet_with_spacing(font: &FIGfont, content: &str, spacing: i16) -> Option<String> {
    if content.is_empty() {
        return None;
    }

    let mut characters = Vec::new();
    for ch in content.chars() {
        if let Some(character) = font.fonts.get(&(ch as u32)) {
            characters.push(character);
        }
    }

    if characters.is_empty() {
        return None;
    }

    let height = font.header_line.height as usize;
    let mut lines: Vec<Vec<char>> = vec![Vec::new(); height];
    let mut cursor: isize = 0;

    for character in characters {
        for (line_idx, line) in character.characters.iter().enumerate() {
            let row = &mut lines[line_idx];
            if cursor > 0 {
                let needed = cursor as usize;
                if row.len() < needed {
                    row.resize(needed, ' ');
                }
            }

            let mut pos = cursor.max(0) as usize;
            for ch in line.chars() {
                if pos >= row.len() {
                    row.push(ch);
                } else if ch != ' ' {
                    row[pos] = ch;
                }
                pos += 1;
            }
        }

        let glyph_width = character
            .characters
            .first()
            .map(|line| line.chars().count())
            .unwrap_or(0) as isize;
        cursor += glyph_width + spacing as isize;
        if cursor < 0 {
            cursor = 0;
        }
    }

    let mut out_lines = Vec::with_capacity(height);
    for mut row in lines {
        while row.last() == Some(&' ') {
            row.pop();
        }
        out_lines.push(row.into_iter().collect::<String>());
    }

    Some(out_lines.join("\n"))
}

fn apply_letter_spacing_plain(text: &str, spacing: i16) -> String {
    if spacing == 0 || text.is_empty() {
        return text.to_string();
    }

    let mut out_lines = Vec::new();

    for line in text.lines() {
        let mut row: Vec<char> = Vec::new();
        let mut cursor: isize = 0;
        for ch in line.chars() {
            if cursor > 0 {
                let needed = cursor as usize;
                if row.len() < needed {
                    row.resize(needed, ' ');
                }
            }
            let pos = cursor.max(0) as usize;
            if pos >= row.len() {
                row.push(ch);
            } else {
                row[pos] = ch;
            }
            cursor += 1 + spacing as isize;
            if cursor < 0 {
                cursor = 0;
            }
        }

        while row.last() == Some(&' ') {
            row.pop();
        }

        out_lines.push(row.into_iter().collect::<String>());
    }

    out_lines.join("\n")
}

fn latin1_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| *byte as char).collect()
}

fn parse_figlet_content(contents: &str) -> Result<FIGfont, String> {
    match FIGfont::from_content(contents) {
        Ok(font) => Ok(font),
        Err(err) => {
            let Some(sanitized) = sanitize_figlet_content(contents) else {
                return Err(err);
            };
            FIGfont::from_content(&sanitized)
        }
    }
}

fn sanitize_figlet_content(contents: &str) -> Option<String> {
    let lines: Vec<&str> = contents.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let mut header_parts = lines[0].split_whitespace();
    header_parts.next()?;
    let height: usize = header_parts.next()?.parse().ok()?;
    header_parts.next()?;
    header_parts.next()?;
    header_parts.next()?;
    let comment_lines: usize = header_parts.next()?.parse().ok()?;

    let offset = 1 + comment_lines + 102 * height;
    if lines.len() <= offset {
        return None;
    }

    let block_size = height + 1;
    let codetag_lines = lines.len().saturating_sub(offset);
    if codetag_lines == 0 || !codetag_lines.is_multiple_of(block_size) {
        return None;
    }

    let mut removed_any = false;
    let mut sanitized: Vec<&str> = Vec::with_capacity(lines.len());
    sanitized.extend_from_slice(&lines[..offset]);

    let blocks = codetag_lines / block_size;
    for i in 0..blocks {
        let start = offset + i * block_size;
        let tag_line = *lines.get(start)?;
        let code_token = tag_line.split_whitespace().next();
        let code = code_token.and_then(parse_codetag_code);
        if code.is_none() || code.is_some_and(|value| value < 0) {
            removed_any = true;
            continue;
        }

        sanitized.extend_from_slice(&lines[start..start + block_size]);
    }

    if !removed_any {
        return None;
    }

    let mut output = sanitized.join("\n");
    if contents.ends_with('\n') {
        output.push('\n');
    }
    Some(output)
}

fn parse_codetag_code(token: &str) -> Option<i32> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }

    let (sign, digits) = if let Some(stripped) = token.strip_prefix('-') {
        (-1, stripped)
    } else {
        (1, token)
    };

    let value = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        i32::from_str_radix(hex, 16).ok()?
    } else if digits.starts_with('0') && digits != "0" {
        i32::from_str_radix(digits, 8).ok()?
    } else {
        digits.parse::<i32>().ok()?
    };

    Some(sign * value)
}

fn measure_rendered(rendered: &str) -> (usize, usize) {
    let mut max_width = 0;
    let mut lines = 0;

    for line in rendered.trim_end_matches('\n').lines() {
        lines += 1;
        max_width = max_width.max(UnicodeWidthStr::width(line));
    }

    (max_width, lines)
}

fn align_rendered_into(
    rendered: &str,
    width: u16,
    height: u16,
    alignment: Alignment,
    out: &mut String,
) {
    out.clear();

    let box_width = width as usize;
    let box_height = height as usize;
    let lines: Vec<&str> = rendered.trim_end_matches('\n').lines().collect();
    let content_height = lines.len();
    let content_width = lines
        .iter()
        .map(|line| UnicodeWidthStr::width(*line))
        .max()
        .unwrap_or(0);

    let (h_align, v_align) = alignment_parts(alignment);

    let left_pad = match h_align {
        HAlign::Left => 0,
        HAlign::Center => (box_width - content_width) / 2,
        HAlign::Right => box_width - content_width,
    };

    let top_pad = match v_align {
        VAlign::Top => 0,
        VAlign::Middle => (box_height - content_height) / 2,
        VAlign::Bottom => box_height - content_height,
    };

    let blank_line = " ".repeat(box_width);
    if box_height > 0 {
        out.reserve((box_width + 1) * box_height - 1);
    }

    let mut line_count = 0;
    for _ in 0..top_pad {
        out.push_str(&blank_line);
        line_count += 1;
        if line_count < box_height {
            out.push('\n');
        }
    }

    for line in lines {
        let line_width = UnicodeWidthStr::width(line);
        let right_pad = box_width.saturating_sub(left_pad + line_width);

        push_spaces(out, left_pad);
        out.push_str(line);
        push_spaces(out, right_pad);

        line_count += 1;
        if line_count < box_height {
            out.push('\n');
        }
    }

    while line_count < box_height {
        out.push_str(&blank_line);
        line_count += 1;
        if line_count < box_height {
            out.push('\n');
        }
    }
}

fn push_spaces(out: &mut String, count: usize) {
    for _ in 0..count {
        out.push(' ');
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VAlign {
    Top,
    Middle,
    Bottom,
}

fn alignment_parts(alignment: Alignment) -> (HAlign, VAlign) {
    match alignment {
        Alignment::TopLeft => (HAlign::Left, VAlign::Top),
        Alignment::Top => (HAlign::Center, VAlign::Top),
        Alignment::TopRight => (HAlign::Right, VAlign::Top),
        Alignment::Left => (HAlign::Left, VAlign::Middle),
        Alignment::Center => (HAlign::Center, VAlign::Middle),
        Alignment::Right => (HAlign::Right, VAlign::Middle),
        Alignment::BottomLeft => (HAlign::Left, VAlign::Bottom),
        Alignment::Bottom => (HAlign::Center, VAlign::Bottom),
        Alignment::BottomRight => (HAlign::Right, VAlign::Bottom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Font Tests ====================

    #[test]
    fn font_plain_creation() {
        let font = Font::plain();
        assert!(font.is_plain());
    }

    #[test]
    fn font_standard_loads() {
        let font = Font::standard();
        assert!(font.is_some());
        assert!(!font.unwrap().is_plain());
    }

    #[test]
    fn font_from_embedded_loads() {
        let font = fonts::font("small");
        assert!(font.is_some());
    }

    #[test]
    fn font_cloning_works() {
        let font = fonts::font("standard").unwrap();
        let cloned = font.clone();
        // Both should render the same
        let original = font.render_with_spacing("A", 0);
        let cloned_render = cloned.render_with_spacing("A", 0);
        assert_eq!(original, cloned_render);
    }

    // ==================== Renderer Tests ====================

    #[test]
    fn renderer_default_creates_with_fonts() {
        let renderer = Renderer::default();
        let result = renderer.render("A", 100, 20);
        assert!(result.is_ok());
    }

    #[test]
    fn renderer_empty_fonts_error() {
        let renderer = Renderer::new(vec![]);
        let result = renderer.render("A", 100, 20);
        assert!(matches!(result, Err(RenderError::EmptyFonts)));
    }

    #[test]
    fn renderer_empty_bounds_error() {
        let renderer = Renderer::default();
        assert!(matches!(
            renderer.render("A", 0, 10),
            Err(RenderError::EmptyBounds)
        ));
        assert!(matches!(
            renderer.render("A", 10, 0),
            Err(RenderError::EmptyBounds)
        ));
    }

    #[test]
    fn renderer_no_fit_error() {
        // Very small bounds that can't fit anything
        let renderer = Renderer::new(vec![fonts::font("big").unwrap()]);
        let result = renderer.render("HELLO WORLD", 5, 2);
        assert!(matches!(result, Err(RenderError::NoFit)));
    }

    #[test]
    fn renderer_with_plain_fallback() {
        let renderer = Renderer::new(vec![fonts::font("big").unwrap()]).with_plain_fallback();
        // Should succeed with plain fallback even in small bounds
        let result = renderer.render("Hi", 10, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn renderer_plain_fallback_not_duplicated() {
        let renderer = Renderer::new(vec![Font::plain()])
            .with_plain_fallback()
            .with_plain_fallback();
        // Should still work, not add multiple plain fonts
        let result = renderer.render("Hi", 10, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn renderer_render_into_reuses_buffer() {
        let renderer = Renderer::default();
        let mut buffer = String::new();

        renderer.render_into("A", 50, 10, &mut buffer).unwrap();
        assert!(!buffer.is_empty());

        renderer.render_into("B", 50, 10, &mut buffer).unwrap();
        // Buffer should be cleared and reused
        assert!(!buffer.is_empty());
    }

    #[test]
    fn renderer_font_fallback_order() {
        // Create renderer with large font first, then small
        let renderer = Renderer::new(vec![
            fonts::font("big").unwrap(),
            fonts::font("mini").unwrap(),
        ]);

        // Large bounds should use first font (index 0)
        let result = renderer.render("A", 100, 20).unwrap();
        assert_eq!(result.font_index, 0);

        // Smaller bounds should fall back to mini (index 1)
        // mini font renders "A" in about 4x2
        let result = renderer.render("A", 15, 5).unwrap();
        assert_eq!(result.font_index, 1);
    }

    // ==================== Alignment Tests ====================

    #[test]
    fn alignment_top_left() {
        let renderer = Renderer::new(vec![Font::plain()]).with_alignment(Alignment::TopLeft);
        let result = renderer.render("X", 5, 3).unwrap();
        let lines: Vec<&str> = result.text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("X"));
    }

    #[test]
    fn alignment_bottom_right() {
        let renderer = Renderer::new(vec![Font::plain()]).with_alignment(Alignment::BottomRight);
        let result = renderer.render("X", 5, 3).unwrap();
        let lines: Vec<&str> = result.text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[2].ends_with("X"));
    }

    #[test]
    fn alignment_center() {
        let renderer = Renderer::new(vec![Font::plain()]).with_alignment(Alignment::Center);
        let result = renderer.render("X", 5, 3).unwrap();
        let lines: Vec<&str> = result.text.lines().collect();
        // Middle line (index 1) should contain X
        assert!(lines[1].contains("X"));
    }

    #[test]
    fn alignment_parts_mapping() {
        assert_eq!(
            alignment_parts(Alignment::TopLeft),
            (HAlign::Left, VAlign::Top)
        );
        assert_eq!(
            alignment_parts(Alignment::Center),
            (HAlign::Center, VAlign::Middle)
        );
        assert_eq!(
            alignment_parts(Alignment::BottomRight),
            (HAlign::Right, VAlign::Bottom)
        );
        assert_eq!(
            alignment_parts(Alignment::Top),
            (HAlign::Center, VAlign::Top)
        );
        assert_eq!(
            alignment_parts(Alignment::Left),
            (HAlign::Left, VAlign::Middle)
        );
    }

    // ==================== Letter Spacing Tests ====================

    #[test]
    fn letter_spacing_plain_zero() {
        let result = apply_letter_spacing_plain("ABC", 0);
        assert_eq!(result, "ABC");
    }

    #[test]
    fn letter_spacing_plain_positive() {
        let result = apply_letter_spacing_plain("AB", 1);
        assert_eq!(result, "A B");
    }

    #[test]
    fn letter_spacing_plain_negative() {
        // Negative spacing causes overlap
        let result = apply_letter_spacing_plain("AB", -1);
        // With -1 spacing, characters overlap completely
        assert_eq!(result, "B");
    }

    #[test]
    fn letter_spacing_plain_empty() {
        let result = apply_letter_spacing_plain("", 5);
        assert_eq!(result, "");
    }

    #[test]
    fn letter_spacing_multiline() {
        let result = apply_letter_spacing_plain("AB\nCD", 1);
        assert_eq!(result, "A B\nC D");
    }

    #[test]
    fn renderer_with_letter_spacing() {
        let renderer = Renderer::new(vec![Font::plain()]).with_letter_spacing(2);
        let result = renderer.render("AB", 20, 1).unwrap();
        // Should have extra spacing
        assert!(result.text.contains("  "));
    }

    // ==================== Measure Rendered Tests ====================

    #[test]
    fn measure_rendered_single_line() {
        let (width, height) = measure_rendered("Hello");
        assert_eq!(width, 5);
        assert_eq!(height, 1);
    }

    #[test]
    fn measure_rendered_multiline() {
        let (width, height) = measure_rendered("Hello\nWorld!");
        assert_eq!(width, 6); // "World!" is longest
        assert_eq!(height, 2);
    }

    #[test]
    fn measure_rendered_trailing_newline() {
        let (width, height) = measure_rendered("Hello\n");
        assert_eq!(width, 5);
        assert_eq!(height, 1);
    }

    #[test]
    fn measure_rendered_empty() {
        let (width, height) = measure_rendered("");
        assert_eq!(width, 0);
        assert_eq!(height, 0);
    }

    // ==================== Parse Codetag Tests ====================

    #[test]
    fn parse_codetag_decimal() {
        assert_eq!(parse_codetag_code("42"), Some(42));
        assert_eq!(parse_codetag_code("0"), Some(0));
        assert_eq!(parse_codetag_code("-42"), Some(-42));
    }

    #[test]
    fn parse_codetag_hex() {
        assert_eq!(parse_codetag_code("0x2A"), Some(42));
        assert_eq!(parse_codetag_code("0X2a"), Some(42));
        assert_eq!(parse_codetag_code("-0x2A"), Some(-42));
    }

    #[test]
    fn parse_codetag_octal() {
        assert_eq!(parse_codetag_code("052"), Some(42));
        assert_eq!(parse_codetag_code("-052"), Some(-42));
    }

    #[test]
    fn parse_codetag_invalid() {
        assert_eq!(parse_codetag_code(""), None);
        assert_eq!(parse_codetag_code("abc"), None);
        assert_eq!(parse_codetag_code("0xZZ"), None);
    }

    // ==================== Latin1 Conversion Tests ====================

    #[test]
    fn latin1_to_string_ascii() {
        let bytes = b"Hello";
        assert_eq!(latin1_to_string(bytes), "Hello");
    }

    #[test]
    fn latin1_to_string_extended() {
        // Latin-1 extended characters (128-255)
        let bytes = &[0xE9u8]; // é in Latin-1
        let result = latin1_to_string(bytes);
        assert_eq!(result, "é");
    }

    // ==================== Rendered Struct Tests ====================

    #[test]
    fn rendered_metrics() {
        let rendered = Rendered {
            text: String::from("test"),
            width: 10,
            height: 5,
            font_index: 2,
        };
        let metrics = rendered.metrics();
        assert_eq!(metrics.width, 10);
        assert_eq!(metrics.height, 5);
        assert_eq!(metrics.font_index, 2);
    }

    // ==================== Error Display Tests ====================

    #[test]
    fn render_error_display() {
        assert_eq!(
            format!("{}", RenderError::EmptyBounds),
            "width and height must be non-zero"
        );
        assert_eq!(format!("{}", RenderError::EmptyFonts), "no fonts provided");
        assert_eq!(
            format!("{}", RenderError::NoFit),
            "no font fits within the requested bounds"
        );
    }

    // ==================== Convenience Function Tests ====================

    #[test]
    fn render_convenience_function() {
        let result = render("Hi", 50, 10);
        assert!(result.is_ok());
    }

    // ==================== Alignment Output Dimensions ====================

    #[test]
    fn aligned_output_fills_bounds() {
        let renderer = Renderer::new(vec![Font::plain()]).with_alignment(Alignment::Center);
        let result = renderer.render("X", 10, 5).unwrap();
        let lines: Vec<&str> = result.text.lines().collect();
        assert_eq!(lines.len(), 5);
        for line in &lines {
            assert_eq!(line.len(), 10);
        }
    }
}
