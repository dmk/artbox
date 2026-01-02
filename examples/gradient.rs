use artbox::{
    fonts, Alignment, Color, ColorStop, Fill, Font, LinearGradient, RadialGradient, Renderer,
};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(
    name = "gradient",
    about = "Render ASCII text with colors and gradients."
)]
struct Args {
    /// Text to render.
    text: String,
    /// Width of the output area.
    width: u16,
    /// Height of the output area.
    height: u16,

    // Color options
    /// Solid color as R,G,B (e.g., "255,0,128").
    #[arg(short, long, value_parser = parse_rgb, conflicts_with_all = ["gradient", "from", "to"])]
    color: Option<(u8, u8, u8)>,

    /// Gradient type: horizontal, vertical, diagonal, radial.
    #[arg(short, long, value_enum)]
    gradient: Option<GradientType>,

    /// Start color for gradient as R,G,B (or H,S,L with --hsl).
    #[arg(long, value_parser = parse_color_tuple, requires = "gradient")]
    from: Option<(f32, f32, f32)>,

    /// End color for gradient as R,G,B (or H,S,L with --hsl).
    #[arg(long, value_parser = parse_color_tuple, requires = "gradient")]
    to: Option<(f32, f32, f32)>,

    /// Custom angle for linear gradients (0-360 degrees).
    #[arg(long, requires = "gradient")]
    angle: Option<f32>,

    /// Interpret --from and --to as H,S,L instead of R,G,B.
    #[arg(long)]
    hsl: bool,

    // Existing rendering options
    /// Alignment: tl, t, tr, l, c, r, bl, b, br.
    #[arg(short, long, default_value = "c", value_parser = parse_alignment)]
    alignment: Alignment,

    /// Spaces between letters, supports negatives.
    #[arg(short = 's', long = "spacing", default_value_t = 0)]
    spacing: i16,

    /// Font name to use.
    #[arg(short, long, value_parser = parse_font_name, conflicts_with = "family")]
    font: Option<String>,

    /// Named font family to use (e.g., slant, script).
    #[arg(long, value_parser = parse_family_name, conflicts_with = "font")]
    family: Option<String>,

    /// Don't print border around the output.
    #[arg(long)]
    no_border: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum GradientType {
    Horizontal,
    Vertical,
    Diagonal,
    Radial,
}

fn main() {
    let args = Args::parse();

    let (inner_width, inner_height) = if args.no_border {
        (args.width, args.height)
    } else {
        if args.width < 2 || args.height < 2 {
            eprintln!("Width and height must be at least 2 for borders.");
            std::process::exit(2);
        }
        (args.width - 2, args.height - 2)
    };

    let fonts = resolve_fonts(&args);
    let mut renderer = Renderer::new(fonts)
        .with_plain_fallback()
        .with_alignment(args.alignment)
        .with_letter_spacing(args.spacing);

    // Apply fill if specified
    if let Some(fill) = resolve_fill(&args) {
        renderer = renderer.with_fill(fill);
    }

    // Render
    let result = if renderer.has_fill() {
        renderer
            .render_styled(&args.text, inner_width, inner_height)
            .map(|styled| styled.to_ansi_string())
    } else {
        renderer
            .render(&args.text, inner_width, inner_height)
            .map(|rendered| rendered.text)
    };

    match result {
        Ok(output) => {
            if args.no_border {
                println!("{}", output);
            } else {
                print_with_border(&output, inner_width, inner_height);
            }
        }
        Err(err) => {
            eprintln!("Render error: {err}");
            std::process::exit(1);
        }
    }
}

fn resolve_fill(args: &Args) -> Option<Fill> {
    // Solid color
    if let Some((r, g, b)) = args.color {
        return Some(Fill::solid(Color::rgb(r, g, b)));
    }

    // Gradient
    let gradient_type = args.gradient?;

    let from = args.from.unwrap_or((255.0, 0.0, 128.0));
    let to = args.to.unwrap_or((0.0, 128.0, 255.0));

    let from_color = tuple_to_color(from, args.hsl);
    let to_color = tuple_to_color(to, args.hsl);

    let stops = vec![
        ColorStop::new(0.0, from_color),
        ColorStop::new(1.0, to_color),
    ];

    match gradient_type {
        GradientType::Horizontal => {
            let angle = args.angle.unwrap_or(0.0);
            Some(Fill::Linear(LinearGradient::new(angle, stops)))
        }
        GradientType::Vertical => {
            let angle = args.angle.unwrap_or(90.0);
            Some(Fill::Linear(LinearGradient::new(angle, stops)))
        }
        GradientType::Diagonal => {
            let angle = args.angle.unwrap_or(45.0);
            Some(Fill::Linear(LinearGradient::new(angle, stops)))
        }
        GradientType::Radial => Some(Fill::Radial(RadialGradient::new(
            (0.5, 0.5),
            (0.5, 0.5),
            1.0,
            stops,
        ))),
    }
}

fn tuple_to_color(tuple: (f32, f32, f32), is_hsl: bool) -> Color {
    if is_hsl {
        Color::hsl(tuple.0, tuple.1, tuple.2)
    } else {
        Color::rgb(tuple.0 as u8, tuple.1 as u8, tuple.2 as u8)
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

fn parse_rgb(value: &str) -> Result<(u8, u8, u8), String> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 3 {
        return Err(format!("Expected R,G,B format, got: {value}"));
    }
    let r = parts[0]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid R value: {}", parts[0]))?;
    let g = parts[1]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid G value: {}", parts[1]))?;
    let b = parts[2]
        .trim()
        .parse::<u8>()
        .map_err(|_| format!("Invalid B value: {}", parts[2]))?;
    Ok((r, g, b))
}

fn parse_color_tuple(value: &str) -> Result<(f32, f32, f32), String> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 3 {
        return Err(format!("Expected 3 comma-separated values, got: {value}"));
    }
    let a = parts[0]
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("Invalid value: {}", parts[0]))?;
    let b = parts[1]
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("Invalid value: {}", parts[1]))?;
    let c = parts[2]
        .trim()
        .parse::<f32>()
        .map_err(|_| format!("Invalid value: {}", parts[2]))?;
    Ok((a, b, c))
}

fn parse_font_name(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("Font name cannot be empty.".to_string());
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
    use unicode_width::UnicodeWidthStr;

    let inner_width = width as usize;
    let border = format!("+{}+", "-".repeat(inner_width));
    println!("{border}");

    let mut lines = rendered.lines();
    for _ in 0..height {
        let line = lines.next().unwrap_or("");
        // Strip ANSI codes for width calculation
        let visible = strip_ansi(line);
        let line_width = UnicodeWidthStr::width(visible.as_str());
        let pad = inner_width.saturating_sub(line_width);
        if pad == 0 {
            println!("|{}|", line);
        } else {
            // Reset at end before padding to avoid color bleeding
            println!("|{}\x1b[0m{}|", line, " ".repeat(pad));
        }
    }

    println!("{border}");
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            result.push(ch);
        }
    }
    result
}
