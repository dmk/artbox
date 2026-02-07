use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "artbox", about = "ASCII art utilities for terminals.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert images to ASCII art.
    Image(ImageArgs),
}

#[derive(Parser)]
struct ImageArgs {
    /// Input image file (PNG, JPG, SVG, etc.)
    image: PathBuf,
    /// Output width in characters.
    #[arg(short, long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(1..))]
    width: u32,
    /// Horizontal scale factor.
    #[arg(long = "h-scale", default_value_t = 1.0)]
    h_scale: f32,
    /// Vertical scale factor.
    #[arg(long = "v-scale", default_value_t = 1.0)]
    v_scale: f32,
    /// Brightness adjustment (-255 to 255).
    #[arg(
        short,
        long,
        default_value_t = 0,
        value_parser = clap::value_parser!(i32).range(-255..=255)
    )]
    brightness: i32,
    /// Contrast factor (0.0 to 3.0).
    #[arg(short, long, default_value_t = 1.0)]
    contrast: f32,
    /// Sharpness factor (0.0 to 3.0).
    #[arg(short, long, default_value_t = 1.0)]
    sharpness: f32,
    /// Rendering mode: full, block, shade, ascii.
    #[arg(short, long, value_enum, default_value_t = ModeArg::Full)]
    mode: ModeArg,
    /// Output file (optional).
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Disable ANSI color output.
    #[arg(long = "no-color", aliases = ["monochrome", "mono"])]
    no_color: bool,
    /// Brightness threshold (0-255) for full/block modes.
    #[arg(short, long, default_value_t = 128, value_parser = clap::value_parser!(u8))]
    threshold: u8,
    /// Invert brightness (dark pixels become blocks).
    #[arg(short, long)]
    invert: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ModeArg {
    Full,
    Block,
    Shade,
    Ascii,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Image(args) => run_image(args),
    }
}

#[cfg(feature = "images")]
fn run_image(args: ImageArgs) {
    use artbox::images::ascii::{render_path, AsciiMode, AsciiOptions};

    let mode = match args.mode {
        ModeArg::Full => AsciiMode::Full,
        ModeArg::Block => AsciiMode::Block,
        ModeArg::Shade => AsciiMode::Shade,
        ModeArg::Ascii => AsciiMode::Ascii,
    };

    if !(0.0..=3.0).contains(&args.contrast) {
        eprintln!("Error: contrast must be between 0.0 and 3.0");
        std::process::exit(2);
    }
    if !(0.0..=3.0).contains(&args.sharpness) {
        eprintln!("Error: sharpness must be between 0.0 and 3.0");
        std::process::exit(2);
    }

    let options = AsciiOptions {
        width: args.width,
        h_scale: args.h_scale,
        v_scale: args.v_scale,
        brightness: args.brightness,
        contrast: args.contrast,
        sharpness: args.sharpness,
        mode,
        color: !args.no_color,
        threshold: args.threshold,
        invert: args.invert,
        alpha_threshold: 128,
    };

    let rendered = match render_path(&args.image, &options) {
        Ok(rendered) => rendered,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    let output = if args.no_color {
        rendered.to_plain_string()
    } else {
        rendered.to_ansi_string()
    };
    print!("{output}");

    if let Some(output_path) = args.output.as_ref() {
        if let Err(err) = std::fs::write(output_path, output) {
            eprintln!("Error writing output: {err}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(feature = "images"))]
fn run_image(_args: ImageArgs) {
    eprintln!("Error: image support requires building with --features images");
    std::process::exit(2);
}
