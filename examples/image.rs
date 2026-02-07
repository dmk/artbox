//! Image-to-ASCII example.
//!
//! Renders an image file as colored ASCII art. Defaults to the bundled
//! `ruby.svg` asset, or pass a custom path as the first argument.
//!
//! Run: cargo run --example image --features images

#[cfg(feature = "images")]
fn main() {
    use artbox::images::ascii::{AsciiMode, AsciiOptions};
    use artbox::{Artbox, RenderTarget};
    use crossterm::terminal;

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/assets/ruby.svg".to_string());

    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let target = RenderTarget::new(cols / 2, rows / 2);

    let art = Artbox::default()
        .with_image_ascii_options(AsciiOptions {
            mode: AsciiMode::Block,
            color: true,
            ..AsciiOptions::default()
        })
        .with_image_output(artbox::images::ImageOutput::Ascii);

    match art.render_image_path(&path, target) {
        Ok(rendered) => print!("{}", rendered.to_ansi_string()),
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "images"))]
fn main() {
    eprintln!("This example requires the `images` feature.");
    eprintln!("Run: cargo run --example image --features images");
    std::process::exit(1);
}
