use artbox::{Alignment, Artbox, Color, ColorStop, Fill, LinearGradient, RenderTarget};
use crossterm::terminal;

fn terminal_target() -> RenderTarget {
    let (w, h) = terminal::size().unwrap_or((80, 24));
    RenderTarget::new(w.max(1), h.max(1))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = terminal_target();
    let gradient = LinearGradient::new(
        45.0,
        vec![
            ColorStop::new(0.0, Color::rgb(255, 0, 128)),
            ColorStop::new(1.0, Color::rgb(0, 128, 255)),
        ],
    );

    let art = Artbox::default()
        .with_alignment(Alignment::Center)
        .with_fill(Fill::Linear(gradient));

    let rendered = art.render_text("artbox", target)?;
    print!("{}", rendered.to_ansi_string());
    Ok(())
}
