//! Ratatui widget integration for artbox.
//!
//! This module provides an [`ArtBox`] widget that can render ASCII art
//! directly into a ratatui terminal buffer with optional color support.
//!
//! # Examples
//!
//! ```rust,no_run
//! use artbox::{Renderer, Alignment, Fill, Color};
//! use artbox::integrations::ratatui::ArtBox;
//! use ratatui::prelude::*;
//!
//! fn render(frame: &mut Frame) {
//!     let renderer = Renderer::default()
//!         .with_alignment(Alignment::Center)
//!         .with_fill(Fill::solid(Color::rgb(255, 100, 0)));
//!
//!     let widget = ArtBox::new(&renderer, "Hello");
//!     frame.render_widget(widget, frame.area());
//! }
//! ```

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::color::Rgb;
use crate::sprites::{Sprite, SpriteSelection};
use crate::styled::StyledChar;
use crate::Renderer;

/// A ratatui widget that renders ASCII art text.
///
/// This widget uses a [`Renderer`] to generate ASCII art that fits within
/// the widget's allocated area. If no font fits, nothing is rendered.
///
/// When a fill is configured on the renderer, the widget will render
/// each character with its computed color from the fill.
///
/// # Examples
///
/// ```rust,no_run
/// use artbox::{Renderer, Alignment, Fill, LinearGradient, Color};
/// use artbox::integrations::ratatui::ArtBox;
///
/// let renderer = Renderer::default()
///     .with_alignment(Alignment::Center)
///     .with_fill(Fill::Linear(LinearGradient::horizontal(
///         Color::rgb(255, 0, 0),
///         Color::rgb(0, 0, 255),
///     )));
///
/// // Use with frame.render_widget(ArtBox::new(&renderer, "Hi"), area);
/// ```
pub struct ArtBox<'a> {
    renderer: &'a Renderer,
    text: &'a str,
}

impl<'a> ArtBox<'a> {
    /// Creates a new ArtBox widget.
    ///
    /// The widget will use the provided renderer's font stack and settings
    /// to render the given text. If a fill is configured, colors will be
    /// applied per-character.
    pub fn new(renderer: &'a Renderer, text: &'a str) -> Self {
        Self { renderer, text }
    }
}

impl Widget for ArtBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Ok(rendered) = self
            .renderer
            .render_grid(self.text, area.width, area.height)
        else {
            return;
        };

        render_grid_to_buffer(&rendered.chars, area, buf);
    }
}

/// A ratatui widget that renders sprites.
///
/// The sprite will auto-select a variant that fits the allocated area unless
/// a selection override is provided.
pub struct SpriteBox<'a> {
    sprite: &'a Sprite<'a>,
    selection: SpriteSelection<'a>,
}

impl<'a> SpriteBox<'a> {
    /// Creates a new SpriteBox widget.
    pub fn new(sprite: &'a Sprite<'a>) -> Self {
        Self {
            sprite,
            selection: SpriteSelection::Auto,
        }
    }

    /// Sets the selection mode (auto, size, or id).
    pub fn with_selection(mut self, selection: SpriteSelection<'a>) -> Self {
        self.selection = selection;
        self
    }
}

impl Widget for SpriteBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Ok(rendered) = self
            .sprite
            .render_with(area.width, area.height, self.selection)
        else {
            return;
        };

        render_grid_to_buffer(&rendered.chars, area, buf);
    }
}

fn render_grid_to_buffer(chars: &[Vec<StyledChar>], area: Rect, buf: &mut Buffer) {
    for (row_idx, row) in chars.iter().enumerate() {
        let y = area.y + row_idx as u16;
        if y >= area.y + area.height {
            break;
        }

        for (col_idx, sc) in row.iter().enumerate() {
            let x = area.x + col_idx as u16;
            if x >= area.x + area.width {
                break;
            }

            let style = match sc.fg {
                Some(rgb) => Style::default().fg(to_ratatui_color(rgb)),
                None => Style::default(),
            };

            buf.set_string(x, y, sc.ch.to_string(), style);
        }
    }
}

fn to_ratatui_color(rgb: Rgb) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(rgb.r, rgb.g, rgb.b)
}
