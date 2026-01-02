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
        if self.renderer.has_fill() {
            // Use styled rendering for colored output
            let Ok(styled) = self
                .renderer
                .render_styled(self.text, area.width, area.height)
            else {
                return;
            };

            for (row_idx, row) in styled.chars.iter().enumerate() {
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
        } else {
            // Use plain rendering (no colors)
            let Ok(rendered) = self.renderer.render(self.text, area.width, area.height) else {
                return;
            };

            let mut y = area.y;
            for line in rendered.text.lines() {
                if y >= area.y + area.height {
                    break;
                }
                buf.set_stringn(area.x, y, line, area.width as usize, Style::default());
                y += 1;
            }
        }
    }
}

/// Converts an artbox RGB color to a ratatui Color.
fn to_ratatui_color(rgb: Rgb) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(rgb.r, rgb.g, rgb.b)
}
