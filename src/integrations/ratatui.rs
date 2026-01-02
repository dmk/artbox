//! Ratatui widget integration for artbox.
//!
//! This module provides an [`ArtBox`] widget that can render ASCII art
//! directly into a ratatui terminal buffer.
//!
//! # Examples
//!
//! ```rust,no_run
//! use artbox::{Renderer, Alignment};
//! use artbox::integrations::ratatui::ArtBox;
//! use ratatui::prelude::*;
//!
//! fn render(frame: &mut Frame) {
//!     let renderer = Renderer::default()
//!         .with_alignment(Alignment::Center);
//!
//!     let widget = ArtBox::new(&renderer, "Hello");
//!     frame.render_widget(widget, frame.area());
//! }
//! ```

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::Renderer;

/// A ratatui widget that renders ASCII art text.
///
/// This widget uses a [`Renderer`] to generate ASCII art that fits within
/// the widget's allocated area. If no font fits, nothing is rendered.
///
/// # Examples
///
/// ```rust,no_run
/// use artbox::{Renderer, Alignment};
/// use artbox::integrations::ratatui::ArtBox;
///
/// let renderer = Renderer::default()
///     .with_alignment(Alignment::Center);
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
    /// to render the given text.
    pub fn new(renderer: &'a Renderer, text: &'a str) -> Self {
        Self { renderer, text }
    }
}

impl Widget for ArtBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
