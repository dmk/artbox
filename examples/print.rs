use artbox::{fonts, Alignment, Font, Renderer};
use clap::Parser;
use unicode_width::UnicodeWidthStr;

#[derive(Parser)]
#[command(name = "print", about = "Render ASCII text inside a bordered box.")]
struct Args {
    /// Text to render.
    text: String,
    /// Total width, including border.
    width: u16,
    /// Total height, including border.
    height: u16,
    /// Alignment: tl, t, tr, l, c, r, bl, b, br.
    #[arg(short, long, default_value = "c", value_parser = parse_alignment)]
    alignment: Alignment,
    /// Spaces between letters, supports negatives.
    #[arg(short = 's', long = "spacing", default_value_t = 0)]
    spacing: i16,
    /// Font name to use (see --family for font families).
    #[arg(short, long, value_parser = parse_font_name, conflicts_with = "family")]
    font: Option<String>,
    /// Named font family to use (e.g. slant, script).
    #[arg(long, value_parser = parse_family_name, conflicts_with = "font")]
    family: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.width < 2 || args.height < 2 {
        eprintln!("Width and height must be at least 2 to draw a border.");
        std::process::exit(2);
    }

    let inner_width = args.width - 2;
    let inner_height = args.height - 2;
    let fonts = resolve_fonts(&args);
    let renderer = Renderer::new(fonts)
        .with_plain_fallback()
        .with_alignment(args.alignment)
        .with_letter_spacing(args.spacing);

    let art = renderer.render(&args.text, inner_width, inner_height);

    match art {
        Ok(rendered) => print_with_border(&rendered.text, inner_width, inner_height),
        Err(err) => {
            eprintln!("Render error: {err}");
            std::process::exit(1);
        }
    }
}

fn resolve_fonts(args: &Args) -> Vec<Font> {
    if let Some(font_name) = args.font.as_deref() {
        let font = fonts::font(font_name).unwrap_or_else(|| {
            eprintln!("Failed to load font: {font_name}");
            std::process::exit(2);
        });
        return vec![font];
    }

    if let Some(family_name) = args.family.as_deref() {
        let family = fonts::family(family_name).unwrap_or_else(|| {
            eprintln!("Failed to load font family: {family_name}");
            std::process::exit(2);
        });
        return family;
    }

    fonts::default()
}

fn parse_alignment(value: &str) -> Result<Alignment, String> {
    match value.to_ascii_lowercase().as_str() {
        "tl" | "top-left" => Ok(Alignment::TopLeft),
        "t" | "top" => Ok(Alignment::Top),
        "tr" | "top-right" => Ok(Alignment::TopRight),
        "l" | "left" => Ok(Alignment::Left),
        "c" | "center" | "middle" => Ok(Alignment::Center),
        "r" | "right" => Ok(Alignment::Right),
        "bl" | "bottom-left" => Ok(Alignment::BottomLeft),
        "b" | "bottom" => Ok(Alignment::Bottom),
        "br" | "bottom-right" => Ok(Alignment::BottomRight),
        _ => Err(format!(
            "Invalid alignment: {value}. Use tl, t, tr, l, c, r, bl, b, br."
        )),
    }
}

fn parse_font_name(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Font name cannot be empty.".to_string());
    }
    if trimmed.contains(',') {
        return Err("Font name cannot contain ','.".to_string());
    }

    let names = fonts::names();
    if !names.iter().any(|name| name.eq_ignore_ascii_case(trimmed)) {
        return Err(format!(
            "Unknown font: {trimmed}. Available fonts: {}",
            names.join(", ")
        ));
    }

    Ok(trimmed.to_string())
}

fn parse_family_name(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Font family cannot be empty.".to_string());
    }

    let names = fonts::family_names();
    if !names.iter().any(|name| name.eq_ignore_ascii_case(trimmed)) {
        return Err(format!(
            "Unknown font family: {trimmed}. Available families: {}",
            names.join(", ")
        ));
    }

    Ok(trimmed.to_string())
}

fn print_with_border(rendered: &str, width: u16, height: u16) {
    let inner_width = width as usize;
    let border = format!("+{}+", "-".repeat(inner_width));
    println!("{border}");

    let mut lines = rendered.lines();
    for _ in 0..height {
        let line = lines.next().unwrap_or("");
        let line_width = UnicodeWidthStr::width(line);
        let pad = inner_width.saturating_sub(line_width);
        if pad == 0 {
            println!("|{}|", line);
        } else {
            println!("|{}{}|", line, " ".repeat(pad));
        }
    }

    println!("{border}");
}
