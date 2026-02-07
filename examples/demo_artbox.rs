//! Unified API demo using the `Artbox` entrypoint.
//!
//! Shows render_text, render_sprite, and the Rendered output type.
//!
//! Run: cargo run --example demo_artbox

use artbox::sprites::{SpriteLayer, SpriteSelection, SpriteVariant};
use artbox::{Alignment, Artbox, Color, ColorStop, Fill, LinearGradient, RenderTarget, Sprite};
use crossterm::terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let target = RenderTarget::new(cols, rows / 2);

    let art = Artbox::default()
        .with_alignment(Alignment::Center)
        .with_fill(Fill::Linear(LinearGradient::new(
            90.0,
            vec![
                ColorStop::new(0.0, Color::rgb(0, 200, 255)),
                ColorStop::new(1.0, Color::rgb(255, 0, 128)),
            ],
        )));

    // --- Text ---
    let text_result = art.render_text("artbox", target)?;
    println!("{}", text_result.to_ansi_string());

    if let Some(metrics) = text_result.metrics() {
        println!(
            "  text: {}x{}, font #{}",
            metrics.width,
            metrics.height,
            metrics.font_index.unwrap_or(0),
        );
    }

    // --- Sprite ---
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

    let sprite_target = RenderTarget::new(cols, 5);
    let sprite_result = art.render_sprite(&sprite, sprite_target, SpriteSelection::Auto)?;
    println!("{}", sprite_result.to_ansi_string());

    if let Some(metrics) = sprite_result.metrics() {
        println!("  sprite: {}x{}", metrics.width, metrics.height);
    }

    // --- Rendered variant inspection ---
    println!();
    match &text_result {
        artbox::Rendered::Text(_) => println!("  text_result is Rendered::Text (lightweight)"),
        artbox::Rendered::Grid(_) => println!("  text_result is Rendered::Grid (has fill)"),
        #[cfg(feature = "images")]
        artbox::Rendered::TerminalImage { .. } => {}
    }

    // --- Renderer access ---
    let _renderer = art.renderer();
    println!("  renderer has fill: {}", art.renderer().has_fill());

    Ok(())
}

/// Demonstrate that without fill, render() returns the lightweight Text variant.
#[allow(dead_code)]
fn show_text_variant() {
    let art = Artbox::default().with_alignment(Alignment::Center);
    let target = RenderTarget::new(40, 10);
    let result = art.render_text("Hi", target).unwrap();

    // No fill → Rendered::Text (no grid allocation)
    match &result {
        artbox::Rendered::Text(text) => {
            println!("plain: {}", text.text);
            println!("metrics: {}x{}", text.width, text.height);
        }
        _ => println!("got grid (fill was set)"),
    }
}
