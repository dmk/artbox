//! Sprite rendering for ASCII art images with layered colors and size variants.
//!
//! Sprites are collections of layered ASCII art frames. Each layer can have its
//! own color, and spaces are treated as transparent when compositing layers.
//! Multiple size variants can be provided so sprites can auto-select the best
//! fit for a given bounding box.
//!
//! # Examples
//!
//! ```rust
//! use artbox::sprites::{Sprite, SpriteLayer, SpriteSelection, SpriteSize, SpriteVariant};
//! use artbox::{Alignment, Color};
//!
//! let small = SpriteVariant::new(
//!     "small",
//!     vec![SpriteLayer::colored(":-)", Color::rgb(255, 200, 0))],
//! );
//! let large = SpriteVariant::new(
//!     "large",
//!     vec![SpriteLayer::colored("( ^_^ )", Color::rgb(255, 200, 0))],
//! );
//!
//! let sprite = Sprite::new(vec![large, small]).with_alignment(Alignment::Center);
//! let rendered = sprite.render(20, 5).unwrap();
//! let forced = sprite
//!     .render_with(20, 5, SpriteSelection::Size(SpriteSize::Small))
//!     .unwrap();
//! ```

use std::borrow::Cow;

use unicode_width::UnicodeWidthStr;

use crate::styled::{grid_to_ansi_string, grid_to_plain_string, StyledChar};
use crate::{alignment_parts, Alignment, Color, Fill, GridRendered, HAlign, VAlign};

/// A single sprite layer with optional fill (solid color or gradient).
#[derive(Debug, Clone)]
pub struct SpriteLayer<'a> {
    /// ASCII art content for this layer.
    pub content: Cow<'a, str>,
    /// Optional layer fill (solid color, gradient, etc.).
    pub fill: Option<Fill>,
}

impl<'a> SpriteLayer<'a> {
    /// Creates a new uncolored sprite layer.
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Self {
            content: content.into(),
            fill: None,
        }
    }

    /// Creates a new sprite layer with a solid color.
    pub fn colored(content: impl Into<Cow<'a, str>>, color: Color) -> Self {
        Self {
            content: content.into(),
            fill: Some(Fill::solid(color)),
        }
    }

    /// Sets a solid color for this layer.
    pub fn with_color(mut self, color: Color) -> Self {
        self.fill = Some(Fill::solid(color));
        self
    }

    /// Sets a fill (solid, gradient, etc.) for this layer.
    pub fn with_fill(mut self, fill: Fill) -> Self {
        self.fill = Some(fill);
        self
    }
}

/// Named sprite size categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSize {
    Small,
    Medium,
    Large,
}

impl SpriteSize {
    /// Returns the canonical string id for this size.
    pub fn as_str(self) -> &'static str {
        match self {
            SpriteSize::Small => "small",
            SpriteSize::Medium => "medium",
            SpriteSize::Large => "large",
        }
    }
}

/// Sprite variant selection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSelection<'a> {
    /// Auto-select the first variant that fits the bounds.
    Auto,
    /// Select by the standard size enum (`small`, `medium`, `large`).
    Size(SpriteSize),
    /// Select by variant id (case-insensitive).
    Id(&'a str),
}

/// A single sprite variant (one size) with layered content.
#[derive(Debug, Clone)]
pub struct SpriteVariant<'a> {
    /// Variant id (e.g., "small", "medium", "large").
    pub id: Cow<'a, str>,
    /// Layers are composited in order; last layer is the topmost.
    pub layers: Vec<SpriteLayer<'a>>,
}

impl<'a> SpriteVariant<'a> {
    /// Creates a new sprite variant with the given id and layers.
    pub fn new(id: impl Into<Cow<'a, str>>, layers: Vec<SpriteLayer<'a>>) -> Self {
        Self {
            id: id.into(),
            layers,
        }
    }

    /// Creates a single-layer sprite variant.
    pub fn single(
        id: impl Into<Cow<'a, str>>,
        content: impl Into<Cow<'a, str>>,
        color: Option<Color>,
    ) -> Self {
        let layer = SpriteLayer {
            content: content.into(),
            fill: color.map(Fill::solid),
        };
        Self::new(id, vec![layer])
    }
}

/// A sprite made of multiple size variants.
#[derive(Debug, Clone)]
pub struct Sprite<'a> {
    variants: Vec<SpriteVariant<'a>>,
    alignment: Alignment,
}

impl<'a> Sprite<'a> {
    /// Creates a new sprite from a list of variants.
    ///
    /// Variants are evaluated in order when using [`SpriteSelection::Auto`].
    pub fn new(variants: Vec<SpriteVariant<'a>>) -> Self {
        Self {
            variants,
            alignment: Alignment::TopLeft,
        }
    }

    /// Sets the alignment within the bounding box.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Returns the configured alignment.
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    /// Returns the sprite variants.
    pub fn variants(&self) -> &[SpriteVariant<'a>] {
        &self.variants
    }

    /// Renders using auto-selection (first variant that fits).
    pub fn render(&self, width: u16, height: u16) -> Result<SpriteRendered, SpriteError> {
        self.render_with(width, height, SpriteSelection::Auto)
    }

    /// Renders using an explicit selection mode.
    pub fn render_with(
        &self,
        width: u16,
        height: u16,
        selection: SpriteSelection<'_>,
    ) -> Result<SpriteRendered, SpriteError> {
        if width == 0 || height == 0 {
            return Err(SpriteError::EmptyBounds);
        }

        if self.variants.is_empty() {
            return Err(SpriteError::EmptyVariants);
        }

        let variant = self.select_variant(width, height, selection)?;
        let (content_width, content_height) = measure_variant(variant);
        if content_width > width as usize || content_height > height as usize {
            return Err(SpriteError::NoFit);
        }

        let composed = composite_variant(variant, content_width, content_height);
        let chars = align_sprite(
            &composed,
            width as usize,
            height as usize,
            content_width,
            content_height,
            self.alignment,
        );

        Ok(SpriteRendered {
            chars,
            width: content_width as u16,
            height: content_height as u16,
        })
    }

    fn select_variant(
        &self,
        width: u16,
        height: u16,
        selection: SpriteSelection<'_>,
    ) -> Result<&SpriteVariant<'a>, SpriteError> {
        match selection {
            SpriteSelection::Auto => {
                for variant in &self.variants {
                    let (w, h) = measure_variant(variant);
                    if w <= width as usize && h <= height as usize {
                        return Ok(variant);
                    }
                }
                Err(SpriteError::NoFit)
            }
            SpriteSelection::Size(size) => self
                .variants
                .iter()
                .find(|variant| variant.id.eq_ignore_ascii_case(size.as_str()))
                .ok_or_else(|| SpriteError::UnknownVariant(size.as_str().to_string())),
            SpriteSelection::Id(id) => self
                .variants
                .iter()
                .find(|variant| variant.id.eq_ignore_ascii_case(id))
                .ok_or_else(|| SpriteError::UnknownVariant(id.to_string())),
        }
    }
}

/// The result of a sprite render operation.
#[derive(Debug, Clone)]
pub struct SpriteRendered {
    /// 2D grid of styled characters (rows of columns).
    pub chars: Vec<Vec<StyledChar>>,
    /// Width of the sprite content before alignment padding.
    pub width: u16,
    /// Height of the sprite content before alignment padding.
    pub height: u16,
}

impl SpriteRendered {
    /// Converts to an ANSI-colored string for terminal output.
    pub fn to_ansi_string(&self) -> String {
        grid_to_ansi_string(&self.chars)
    }

    /// Converts to a plain string without color codes.
    pub fn to_plain_string(&self) -> String {
        grid_to_plain_string(&self.chars)
    }

    /// Returns metrics about the sprite content.
    pub fn metrics(&self) -> SpriteMetrics {
        SpriteMetrics {
            width: self.width,
            height: self.height,
        }
    }
}

impl From<SpriteRendered> for GridRendered {
    fn from(rendered: SpriteRendered) -> Self {
        GridRendered {
            chars: rendered.chars,
            width: rendered.width,
            height: rendered.height,
            font_index: None,
        }
    }
}

/// Metrics about a rendered sprite.
#[derive(Debug, Clone, Copy)]
pub struct SpriteMetrics {
    pub width: u16,
    pub height: u16,
}

/// Errors that can occur during sprite rendering.
#[derive(Debug, Clone)]
pub enum SpriteError {
    /// The specified width or height was zero.
    EmptyBounds,
    /// No variants were configured in the sprite.
    EmptyVariants,
    /// No variant fits within the requested bounds.
    NoFit,
    /// The requested variant id or size was not found.
    UnknownVariant(String),
}

impl std::fmt::Display for SpriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpriteError::EmptyBounds => write!(f, "width and height must be non-zero"),
            SpriteError::EmptyVariants => write!(f, "no sprite variants provided"),
            SpriteError::NoFit => write!(f, "no sprite variant fits within the bounds"),
            SpriteError::UnknownVariant(id) => write!(f, "unknown sprite variant: {id}"),
        }
    }
}

impl std::error::Error for SpriteError {}

#[derive(Debug)]
struct LayerGrid {
    lines: Vec<Vec<char>>,
    height: usize,
    fill: Option<Fill>,
}

fn measure_variant(variant: &SpriteVariant<'_>) -> (usize, usize) {
    let mut max_width = 0;
    let mut max_height = 0;

    for layer in &variant.layers {
        let (width, height) = measure_content(&layer.content);
        max_width = max_width.max(width);
        max_height = max_height.max(height);
    }

    (max_width, max_height)
}

fn measure_content(content: &str) -> (usize, usize) {
    let mut max_width = 0;
    let mut lines = 0;

    for line in content.trim_end_matches('\n').lines() {
        let trimmed = line.trim_end_matches(' ');
        max_width = max_width.max(UnicodeWidthStr::width(trimmed));
        lines += 1;
    }

    (max_width, lines)
}

fn build_layer_grid(layer: &SpriteLayer<'_>) -> LayerGrid {
    let mut lines = Vec::new();

    for line in layer.content.trim_end_matches('\n').lines() {
        let trimmed = line.trim_end_matches(' ');
        let chars: Vec<char> = trimmed.chars().collect();
        lines.push(chars);
    }

    LayerGrid {
        height: lines.len(),
        lines,
        fill: layer.fill.clone(),
    }
}

fn composite_variant(
    variant: &SpriteVariant<'_>,
    content_width: usize,
    content_height: usize,
) -> Vec<Vec<StyledChar>> {
    let grids: Vec<LayerGrid> = variant.layers.iter().map(build_layer_grid).collect();
    let mut out = vec![vec![StyledChar::plain(' '); content_width]; content_height];

    let w = if content_width > 1 {
        (content_width - 1) as f32
    } else {
        1.0
    };
    let h = if content_height > 1 {
        (content_height - 1) as f32
    } else {
        1.0
    };

    for (row_idx, row) in out.iter_mut().enumerate() {
        for (col_idx, cell) in row.iter_mut().enumerate() {
            let mut ch = ' ';
            let mut fg = None;

            for grid in grids.iter().rev() {
                if row_idx < grid.height {
                    if let Some(&candidate) = grid.lines[row_idx].get(col_idx) {
                        if candidate != ' ' {
                            ch = candidate;
                            fg = grid.fill.as_ref().map(|fill| {
                                let nx = col_idx as f32 / w;
                                let ny = row_idx as f32 / h;
                                fill.color_at(nx, ny)
                            });
                            break;
                        }
                    }
                }
            }

            *cell = StyledChar::new(ch, fg);
        }
    }

    out
}

fn align_sprite(
    content: &[Vec<StyledChar>],
    box_width: usize,
    box_height: usize,
    content_width: usize,
    content_height: usize,
    alignment: Alignment,
) -> Vec<Vec<StyledChar>> {
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

    let mut out = vec![vec![StyledChar::plain(' '); box_width]; box_height];

    for (row_idx, row) in content.iter().enumerate().take(content_height) {
        let dst_row = top_pad + row_idx;
        if dst_row >= box_height {
            break;
        }

        for (col_idx, cell) in row.iter().enumerate().take(content_width) {
            let dst_col = left_pad + col_idx;
            if dst_col >= box_width {
                break;
            }
            out[dst_row][dst_col] = *cell;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rgb;

    #[test]
    fn auto_selects_first_fit() {
        let large = SpriteVariant::single("large", "XXXX\nXXXX", None);
        let small = SpriteVariant::single("small", "X", None);
        let sprite = Sprite::new(vec![large, small]);

        let rendered = sprite.render(2, 1).unwrap();
        assert_eq!(rendered.width, 1);
        assert_eq!(rendered.height, 1);
        assert_eq!(rendered.to_plain_string().trim(), "X");
    }

    #[test]
    fn selection_by_size() {
        let small = SpriteVariant::single("small", "S", None);
        let medium = SpriteVariant::single("medium", "MM\nMM", None);
        let sprite = Sprite::new(vec![medium, small]);

        let rendered = sprite
            .render_with(10, 10, SpriteSelection::Size(SpriteSize::Medium))
            .unwrap();
        assert_eq!(rendered.width, 2);
        assert_eq!(rendered.height, 2);
    }

    #[test]
    fn layer_composite_uses_topmost_char_and_color() {
        let bottom = SpriteLayer::colored("A", Color::rgb(0, 0, 255));
        let top = SpriteLayer::colored("B", Color::rgb(255, 0, 0));
        let variant = SpriteVariant::new("small", vec![bottom, top]);
        let sprite = Sprite::new(vec![variant]);

        let rendered = sprite.render(1, 1).unwrap();
        assert_eq!(rendered.chars[0][0].ch, 'B');
        assert_eq!(rendered.chars[0][0].fg, Some(Rgb::new(255, 0, 0)));
    }

    #[test]
    fn aligns_into_box() {
        let variant = SpriteVariant::single("small", "X", None);
        let sprite = Sprite::new(vec![variant]).with_alignment(Alignment::Center);

        let rendered = sprite.render(3, 3).unwrap();
        assert_eq!(rendered.chars.len(), 3);
        assert_eq!(rendered.chars[0].len(), 3);
    }

    #[test]
    fn measure_content_uses_display_width() {
        // CJK character "世" has display width 2 but chars().count() == 1
        let (width, height) = measure_content("\u{4e16}");
        assert_eq!(width, 2);
        assert_eq!(height, 1);
    }
}
