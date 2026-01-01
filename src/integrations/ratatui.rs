use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::Renderer;

pub struct ArtBox<'a> {
    renderer: &'a Renderer,
    text: &'a str,
}

impl<'a> ArtBox<'a> {
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
