//! Styled rendering with per-character colors.
//!
//! This module provides types for rendering ASCII art with colors applied
//! to each character, supporting solid colors and gradients.

use crate::color::{Fill, Rgb, ANSI_RESET};

/// A single character with optional foreground color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyledChar {
    /// The character.
    pub ch: char,
    /// Optional foreground color (None for default terminal color).
    pub fg: Option<Rgb>,
}

impl StyledChar {
    /// Creates a new styled character.
    pub fn new(ch: char, fg: Option<Rgb>) -> Self {
        Self { ch, fg }
    }

    /// Creates a styled character with no color.
    pub fn plain(ch: char) -> Self {
        Self { ch, fg: None }
    }

    /// Creates a styled character with a foreground color.
    pub fn colored(ch: char, fg: Rgb) -> Self {
        Self { ch, fg: Some(fg) }
    }
}

/// The result of a render operation that yields a styled character grid.
///
/// Contains a 2D grid of styled characters with per-character color information.
#[derive(Debug, Clone)]
pub struct GridRendered {
    /// 2D grid of styled characters (rows of columns).
    pub chars: Vec<Vec<StyledChar>>,
    /// Width of the content before alignment padding.
    pub width: u16,
    /// Height of the content before alignment padding.
    pub height: u16,
    /// Index of the font that was used from the renderer's font stack.
    pub font_index: Option<usize>,
}

impl GridRendered {
    /// Converts to an ANSI-colored string for terminal output.
    ///
    /// Uses 24-bit true color escape sequences (`\x1b[38;2;R;G;Bm`).
    pub fn to_ansi_string(&self) -> String {
        grid_to_ansi_string(&self.chars)
    }

    /// Converts to a plain string without any color codes.
    pub fn to_plain_string(&self) -> String {
        grid_to_plain_string(&self.chars)
    }

    /// Returns the metrics for this rendered result.
    pub fn metrics(&self) -> crate::RenderMetrics {
        crate::RenderMetrics {
            width: self.width,
            height: self.height,
            font_index: self.font_index,
        }
    }
}

/// Converts a 2D grid of styled characters to an ANSI-colored string.
pub(crate) fn grid_to_ansi_string(chars: &[Vec<StyledChar>]) -> String {
    let mut out = String::new();
    let mut last_color: Option<Rgb> = None;

    for (row_idx, row) in chars.iter().enumerate() {
        for sc in row {
            if sc.fg != last_color {
                if let Some(rgb) = sc.fg {
                    out.push_str(&rgb.to_ansi_fg());
                } else {
                    out.push_str(ANSI_RESET);
                }
                last_color = sc.fg;
            }
            out.push(sc.ch);
        }

        if row_idx < chars.len() - 1 {
            out.push('\n');
        }
    }

    if last_color.is_some() {
        out.push_str(ANSI_RESET);
    }

    out
}

/// Converts a 2D grid of styled characters to a plain string.
pub(crate) fn grid_to_plain_string(chars: &[Vec<StyledChar>]) -> String {
    let mut out = String::new();
    for (row_idx, row) in chars.iter().enumerate() {
        for sc in row {
            out.push(sc.ch);
        }
        if row_idx < chars.len() - 1 {
            out.push('\n');
        }
    }
    out
}

/// Creates a plain styled character grid with no colors.
///
/// All characters have `fg: None`, using the terminal's default color.
pub fn apply_plain(
    text: &str,
    box_width: u16,
    box_height: u16,
    content_width: u16,
    content_height: u16,
    font_index: usize,
) -> GridRendered {
    let lines: Vec<&str> = text.lines().collect();
    let mut chars = Vec::with_capacity(lines.len());

    for line in lines.iter() {
        let mut row: Vec<StyledChar> = line.chars().map(StyledChar::plain).collect();

        // Pad to box width
        while row.len() < box_width as usize {
            row.push(StyledChar::plain(' '));
        }

        chars.push(row);
    }

    // Pad to box height
    while chars.len() < box_height as usize {
        chars.push(vec![StyledChar::plain(' '); box_width as usize]);
    }

    GridRendered {
        chars,
        width: content_width,
        height: content_height,
        font_index: Some(font_index),
    }
}

/// Applies a fill to rendered text, producing a styled character grid.
///
/// The fill is applied based on each character's position within the
/// bounding box. Spaces are not colored.
pub fn apply_fill(
    text: &str,
    fill: &Fill,
    box_width: u16,
    box_height: u16,
    content_width: u16,
    content_height: u16,
    font_index: usize,
) -> GridRendered {
    let lines: Vec<&str> = text.lines().collect();
    let mut chars = Vec::with_capacity(lines.len());

    let box_w = box_width as f32;
    let box_h = box_height as f32;

    for (row_idx, line) in lines.iter().enumerate() {
        let mut row = Vec::new();
        let y = if box_h > 1.0 {
            row_idx as f32 / (box_h - 1.0)
        } else {
            0.5
        };

        let mut col_idx = 0usize;
        for ch in line.chars() {
            let x = if box_w > 1.0 {
                col_idx as f32 / (box_w - 1.0)
            } else {
                0.5
            };

            let fg = if ch == ' ' {
                None // Don't color spaces
            } else {
                Some(fill.color_at(x, y))
            };

            row.push(StyledChar { ch, fg });
            col_idx += 1;
        }

        // Pad to box width with uncolored spaces
        while col_idx < box_width as usize {
            row.push(StyledChar::plain(' '));
            col_idx += 1;
        }

        chars.push(row);
    }

    // Pad to box height with empty rows
    while chars.len() < box_height as usize {
        let row = vec![StyledChar::plain(' '); box_width as usize];
        chars.push(row);
    }

    GridRendered {
        chars,
        width: content_width,
        height: content_height,
        font_index: Some(font_index),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::LinearGradient;

    #[test]
    fn styled_char_plain() {
        let sc = StyledChar::plain('X');
        assert_eq!(sc.ch, 'X');
        assert!(sc.fg.is_none());
    }

    #[test]
    fn styled_char_colored() {
        let sc = StyledChar::colored('A', Rgb::new(255, 0, 0));
        assert_eq!(sc.ch, 'A');
        assert_eq!(sc.fg, Some(Rgb::new(255, 0, 0)));
    }

    #[test]
    fn to_plain_string_strips_colors() {
        let chars = vec![
            vec![
                StyledChar::colored('H', Rgb::new(255, 0, 0)),
                StyledChar::colored('i', Rgb::new(0, 255, 0)),
            ],
            vec![StyledChar::plain(' '), StyledChar::plain(' ')],
        ];
        let rendered = GridRendered {
            chars,
            width: 2,
            height: 2,
            font_index: Some(0),
        };
        assert_eq!(rendered.to_plain_string(), "Hi\n  ");
    }

    #[test]
    fn to_ansi_string_has_color_codes() {
        let chars = vec![vec![StyledChar::colored('X', Rgb::new(255, 0, 0))]];
        let rendered = GridRendered {
            chars,
            width: 1,
            height: 1,
            font_index: Some(0),
        };
        let ansi = rendered.to_ansi_string();
        assert!(ansi.contains("\x1b[38;2;255;0;0m"));
        assert!(ansi.contains('X'));
        assert!(ansi.contains(ANSI_RESET));
    }

    #[test]
    fn apply_fill_solid() {
        let fill = Fill::solid(Rgb::new(100, 150, 200));
        let result = apply_fill("AB\nCD", &fill, 2, 2, 2, 2, 0);

        assert_eq!(result.chars.len(), 2);
        assert_eq!(result.chars[0].len(), 2);

        // All non-space chars should have the same color
        assert_eq!(result.chars[0][0].fg, Some(Rgb::new(100, 150, 200)));
        assert_eq!(result.chars[1][1].fg, Some(Rgb::new(100, 150, 200)));
    }

    #[test]
    fn apply_fill_gradient_varies() {
        let fill = Fill::Linear(LinearGradient::horizontal(
            Rgb::new(0, 0, 0),
            Rgb::new(255, 255, 255),
        ));
        let result = apply_fill("ABCD", &fill, 4, 1, 4, 1, 0);

        // Colors should vary across the row
        let left = result.chars[0][0].fg.unwrap();
        let right = result.chars[0][3].fg.unwrap();
        assert!(left.r < right.r); // Left should be darker
    }

    #[test]
    fn apply_fill_spaces_not_colored() {
        let fill = Fill::solid(Rgb::new(255, 0, 0));
        let result = apply_fill("A B", &fill, 3, 1, 3, 1, 0);

        assert!(result.chars[0][0].fg.is_some()); // 'A' colored
        assert!(result.chars[0][1].fg.is_none()); // ' ' not colored
        assert!(result.chars[0][2].fg.is_some()); // 'B' colored
    }

    #[test]
    fn apply_fill_pads_to_box_size() {
        let fill = Fill::solid(Rgb::new(0, 0, 0));
        let result = apply_fill("X", &fill, 5, 3, 1, 1, 0);

        assert_eq!(result.chars.len(), 3); // 3 rows
        assert_eq!(result.chars[0].len(), 5); // 5 cols
    }
}
