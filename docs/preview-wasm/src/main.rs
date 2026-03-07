use artbox::{
    fonts, Alignment, Artbox, Color, ColorStop, Fill, LinearGradient, RadialGradient, RenderTarget,
    Renderer,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FillMode {
    None,
    Solid,
    Linear,
    Radial,
}

#[derive(Clone, Debug)]
struct Config {
    text: String,
    cols: u16,
    rows: u16,
    family: String,
    alignment: Alignment,
    letter_spacing: i16,
    plain_fallback: bool,
    fill_mode: FillMode,
    color_a: Color,
    color_b: Color,
    angle: f32,
    radius: f32,
    stops: Vec<(f32, Color)>,
    center_x: f32,
    center_y: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            text: "artbox".to_string(),
            cols: 72,
            rows: 16,
            family: "default".to_string(),
            alignment: Alignment::Center,
            letter_spacing: 0,
            plain_fallback: true,
            fill_mode: FillMode::Linear,
            color_a: Color::rgb(0, 200, 255),
            color_b: Color::rgb(255, 90, 120),
            angle: 90.0,
            radius: 0.95,
            stops: Vec::new(),
            center_x: 0.5,
            center_y: 0.5,
        }
    }
}

fn parse_u16(value: Option<String>, fallback: u16) -> u16 {
    value
        .and_then(|v| v.parse::<u16>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(fallback)
}

fn parse_i16(value: Option<String>, fallback: i16) -> i16 {
    value
        .and_then(|v| v.parse::<i16>().ok())
        .unwrap_or(fallback)
}

fn parse_f32(value: Option<String>, fallback: f32) -> f32 {
    value
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(fallback)
}

fn parse_rgb_hex(value: &str) -> Option<Color> {
    let raw = value.trim();
    let hex = raw.strip_prefix('#').unwrap_or(raw);
    let expanded = match hex.len() {
        3 => {
            let mut s = String::with_capacity(6);
            for ch in hex.chars() {
                s.push(ch);
                s.push(ch);
            }
            s
        }
        6 => hex.to_string(),
        _ => return None,
    };

    let r = u8::from_str_radix(&expanded[0..2], 16).ok()?;
    let g = u8::from_str_radix(&expanded[2..4], 16).ok()?;
    let b = u8::from_str_radix(&expanded[4..6], 16).ok()?;
    Some(Color::rgb(r, g, b))
}

fn parse_alignment(value: &str) -> Option<Alignment> {
    match value.trim().to_ascii_lowercase().as_str() {
        "top-left" => Some(Alignment::TopLeft),
        "top" => Some(Alignment::Top),
        "top-right" => Some(Alignment::TopRight),
        "left" => Some(Alignment::Left),
        "center" => Some(Alignment::Center),
        "right" => Some(Alignment::Right),
        "bottom-left" => Some(Alignment::BottomLeft),
        "bottom" => Some(Alignment::Bottom),
        "bottom-right" => Some(Alignment::BottomRight),
        _ => None,
    }
}

fn parse_fill_mode(value: &str) -> Option<FillMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(FillMode::None),
        "solid" => Some(FillMode::Solid),
        "linear" => Some(FillMode::Linear),
        "radial" => Some(FillMode::Radial),
        _ => None,
    }
}

fn normalize_family(value: &str) -> String {
    let family = value.trim().to_ascii_lowercase();
    match family.as_str() {
        "default" | "banner" | "blocky" | "script" | "slant" => family,
        _ => "default".to_string(),
    }
}

fn parse_args() -> Config {
    let mut args = std::env::args().skip(1);
    let mut config = Config::default();

    config.text = args.next().unwrap_or_else(|| config.text.clone());
    config.cols = parse_u16(args.next(), config.cols);
    config.rows = parse_u16(args.next(), config.rows);

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--family" => {
                if let Some(value) = args.next() {
                    config.family = normalize_family(&value);
                }
            }
            "--align" => {
                if let Some(value) = args.next().and_then(|v| parse_alignment(&v)) {
                    config.alignment = value;
                }
            }
            "--letter-spacing" => {
                config.letter_spacing = parse_i16(args.next(), config.letter_spacing);
            }
            "--plain-fallback" => config.plain_fallback = true,
            "--no-plain-fallback" => config.plain_fallback = false,
            "--fill" => {
                if let Some(value) = args.next().and_then(|v| parse_fill_mode(&v)) {
                    config.fill_mode = value;
                }
            }
            "--color-a" => {
                if let Some(value) = args.next().and_then(|v| parse_rgb_hex(&v)) {
                    config.color_a = value;
                }
            }
            "--color-b" => {
                if let Some(value) = args.next().and_then(|v| parse_rgb_hex(&v)) {
                    config.color_b = value;
                }
            }
            "--angle" => {
                config.angle = parse_f32(args.next(), config.angle);
            }
            "--radius" => {
                config.radius = parse_f32(args.next(), config.radius).clamp(0.05, 2.0);
            }
            "--stop" => {
                if let Some(val) = args.next() {
                    if let Some((pos_str, color_str)) = val.split_once(':') {
                        if let (Ok(pos), Some(color)) =
                            (pos_str.parse::<f32>(), parse_rgb_hex(color_str))
                        {
                            config.stops.push((pos, color));
                        }
                    }
                }
            }
            "--center-x" => {
                config.center_x = parse_f32(args.next(), config.center_x).clamp(0.0, 1.0);
            }
            "--center-y" => {
                config.center_y = parse_f32(args.next(), config.center_y).clamp(0.0, 1.0);
            }
            _ => {}
        }
    }

    config
}

fn build_renderer(config: &Config) -> Renderer {
    let fonts = if config.family == "default" {
        fonts::default()
    } else {
        fonts::family(&config.family).unwrap_or_else(fonts::default)
    };

    let mut renderer = Renderer::new(fonts)
        .with_alignment(config.alignment)
        .with_letter_spacing(config.letter_spacing);

    if config.plain_fallback {
        renderer = renderer.with_plain_fallback();
    }

    let stops: Vec<ColorStop> = if config.stops.is_empty() {
        vec![
            ColorStop::new(0.0, config.color_a),
            ColorStop::new(1.0, config.color_b),
        ]
    } else {
        config
            .stops
            .iter()
            .map(|(pos, color)| ColorStop::new(*pos, *color))
            .collect()
    };

    match config.fill_mode {
        FillMode::None => {}
        FillMode::Solid => {
            renderer = renderer.with_fill(Fill::solid(
                stops.first().map(|s| s.color).unwrap_or(config.color_a),
            ));
        }
        FillMode::Linear => {
            renderer =
                renderer.with_fill(Fill::Linear(LinearGradient::new(config.angle, stops)));
        }
        FillMode::Radial => {
            let center = (config.center_x, config.center_y);
            renderer = renderer.with_fill(Fill::Radial(RadialGradient::new(
                center,
                center,
                config.radius,
                stops,
            )));
        }
    }

    renderer
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();
    let renderer = build_renderer(&config);
    let art = Artbox::from_renderer(renderer);
    let target = RenderTarget::new(config.cols, config.rows);

    match art.render_text(&config.text, target) {
        Ok(rendered) => {
            print!("{}", rendered.to_ansi_string());
        }
        Err(err) => {
            eprintln!("render error: {err}");
        }
    }

    Ok(())
}
