//! Simple image-to-ASCII example.
//!
//! Renders an image file as ASCII art and prints it to the terminal.
//!
//! Run: cargo run --example demo_image --features images -- <path>

#[cfg(feature = "images")]
fn main() {
    use artbox::images::ascii::{AsciiMode, AsciiOptions};
    use artbox::{Artbox, RenderTarget};
    use crossterm::terminal;

    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("Usage: demo_image <image-path>");
            std::process::exit(2);
        }
    };

    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let target = RenderTarget::new(cols, rows);

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
    eprintln!("Run: cargo run --example demo_image --features images -- <path>");
    std::process::exit(1);
}
