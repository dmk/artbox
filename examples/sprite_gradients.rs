//! Sprite rendering with per-layer gradients.
//!
//! Loads weather sprite assets and applies different gradient fills
//! to each layer (sun + cloud), auto-sizing to the terminal.
//!
//! Run: cargo run --example sprite_gradients

use std::fs;

use artbox::sprites::{SpriteLayer, SpriteVariant};
use artbox::{Alignment, Color, ColorStop, Fill, LinearGradient, RenderTarget, Sprite};
use crossterm::terminal;

fn main() {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let target = RenderTarget::new(cols / 2, rows / 2);

    let sun_fill = Fill::Linear(LinearGradient::new(
        45.0,
        vec![
            ColorStop::new(0.0, Color::rgb(255, 240, 50)),
            ColorStop::new(0.5, Color::rgb(255, 160, 0)),
            ColorStop::new(1.0, Color::rgb(255, 60, 0)),
        ],
    ));

    let cloud_fill = Fill::Linear(LinearGradient::new(
        135.0,
        vec![
            ColorStop::new(0.0, Color::rgb(180, 210, 255)),
            ColorStop::new(0.5, Color::rgb(140, 160, 200)),
            ColorStop::new(1.0, Color::rgb(80, 100, 140)),
        ],
    ));

    let variants: Vec<SpriteVariant<'_>> = ["large", "medium", "small"]
        .iter()
        .map(|size| {
            let sun_path = format!("examples/assets/weather/partly_cloudy/{size}_yellow.txt");
            let cloud_path = format!("examples/assets/weather/partly_cloudy/{size}_gray.txt");

            let sun_layer = SpriteLayer::new(load_file(&sun_path)).with_fill(sun_fill.clone());
            let cloud_layer =
                SpriteLayer::new(load_file(&cloud_path)).with_fill(cloud_fill.clone());

            SpriteVariant::new(*size, vec![sun_layer, cloud_layer])
        })
        .collect();

    let sprite = Sprite::new(variants).with_alignment(Alignment::Center);

    match sprite.render(target.width, target.height) {
        Ok(rendered) => print!("{}", rendered.to_ansi_string()),
        Err(err) => eprintln!("Error: {err}"),
    }
}

fn load_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("Failed to read {path}: {err}");
        std::process::exit(1);
    })
}
