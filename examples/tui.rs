//! TUI example using the Artbox struct with ratatui.
//!
//! Shows a gradient title and a weather sprite that auto-sizes to the terminal.
//!
//! Run: cargo run --example tui --features ratatui
//!
//! Press 'q' or Esc to quit.

#[cfg(feature = "ratatui")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io;

    use artbox::integrations::ratatui::{ArtBox, SpriteBox};
    use artbox::sprites::{SpriteLayer, SpriteVariant};
    use artbox::{Alignment, Color, ColorStop, Fill, LinearGradient, Renderer, Sprite};
    use crossterm::event::{self, Event, KeyCode};
    use crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    };
    use crossterm::ExecutableCommand;
    use ratatui::layout::{Constraint, Layout};
    use ratatui::prelude::CrosstermBackend;
    use ratatui::Terminal;

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let renderer = Renderer::default()
        .with_alignment(Alignment::Center)
        .with_fill(Fill::Linear(LinearGradient::new(
            45.0,
            vec![
                ColorStop::new(0.0, Color::rgb(255, 100, 0)),
                ColorStop::new(1.0, Color::rgb(0, 200, 255)),
            ],
        )));

    let sun_small = SpriteVariant::new(
        "small",
        vec![SpriteLayer::colored("\\o/", Color::rgb(255, 200, 0))],
    );
    let sun_large = SpriteVariant::new(
        "large",
        vec![SpriteLayer::colored(
            " \\ | / \n--( )--\n / | \\ ",
            Color::rgb(255, 200, 0),
        )],
    );
    let sprite = Sprite::new(vec![sun_large, sun_small]).with_alignment(Alignment::Center);

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let [title_area, sprite_area] =
                Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .areas(area);

            // Render title using the ratatui ArtBox widget
            let widget = ArtBox::new(&renderer, "artbox");
            frame.render_widget(widget, title_area);

            // Render sprite using the SpriteBox widget
            frame.render_widget(SpriteBox::new(&sprite), sprite_area);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

#[cfg(not(feature = "ratatui"))]
fn main() {
    eprintln!("This example requires the `ratatui` feature.");
    eprintln!("Run: cargo run --example tui --features ratatui");
    std::process::exit(1);
}
