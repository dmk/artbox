use std::fs;

use artbox::{Color, ColorStop, Fill, GridRendered, LinearGradient, RenderTarget, StyledChar};
use crossterm::terminal;

fn main() {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    let target = RenderTarget::new(cols, rows);

    let variants = vec![variant("large"), variant("medium"), variant("small")];
    let mut chosen = variants.last().expect("at least one variant should exist");
    for variant in &variants {
        if variant.width <= target.width as usize && variant.height <= target.height as usize {
            chosen = variant;
            break;
        }
    }

    let sun_fill = Fill::Linear(LinearGradient::new(
        45.0,
        vec![
            ColorStop::new(0.0, Color::rgb(255, 200, 0)),
            ColorStop::new(1.0, Color::rgb(255, 128, 0)),
        ],
    ));

    let cloud_fill = Fill::Linear(LinearGradient::new(
        135.0,
        vec![
            ColorStop::new(0.0, Color::rgb(200, 200, 200)),
            ColorStop::new(1.0, Color::rgb(120, 120, 120)),
        ],
    ));

    let sun_layer = apply_gradient(&chosen.sun, sun_fill);
    let cloud_layer = apply_gradient(&chosen.cloud, cloud_fill);

    let mut composite = composite_layers(&[sun_layer, cloud_layer]);
    composite = pad_grid(composite, target.width as usize, target.height as usize);

    let rendered = GridRendered {
        chars: composite,
        width: target.width,
        height: target.height,
        font_index: None,
    };

    print!("{}", rendered.to_ansi_string());
}

struct Variant {
    sun: Vec<Vec<char>>,
    cloud: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

fn variant(size: &str) -> Variant {
    let sun_path = format!("examples/assets/weather/partly_cloudy/{}_yellow.txt", size);
    let cloud_path = format!("examples/assets/weather/partly_cloudy/{}_gray.txt", size);

    let sun = load_grid(&sun_path);
    let cloud = load_grid(&cloud_path);

    let width = sun
        .iter()
        .chain(cloud.iter())
        .map(|row| row.len())
        .max()
        .unwrap_or(0);
    let height = sun.len().max(cloud.len());

    Variant {
        sun,
        cloud,
        width,
        height,
    }
}

fn load_grid(path: &str) -> Vec<Vec<char>> {
    fs::read_to_string(path)
        .unwrap_or_else(|err| {
            eprintln!("Failed to read {path}: {err}");
            std::process::exit(1);
        })
        .lines()
        .map(|line| line.chars().collect())
        .collect()
}

fn apply_gradient(grid: &[Vec<char>], fill: Fill) -> Vec<Vec<StyledChar>> {
    let height = grid.len().max(1) as f32;
    let width = grid.iter().map(|row| row.len()).max().unwrap_or(1) as f32;

    grid.iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(|(x, &ch)| {
                    if ch == ' ' {
                        return StyledChar::plain(' ');
                    }

                    let nx = if width > 1.0 {
                        x as f32 / (width - 1.0)
                    } else {
                        0.5
                    };
                    let ny = if height > 1.0 {
                        y as f32 / (height - 1.0)
                    } else {
                        0.5
                    };
                    let color = fill.color_at(nx, ny);
                    StyledChar::new(ch, Some(color))
                })
                .collect()
        })
        .collect()
}

fn composite_layers(layers: &[Vec<Vec<StyledChar>>]) -> Vec<Vec<StyledChar>> {
    let height = layers.iter().map(|layer| layer.len()).max().unwrap_or(0);
    let width = layers
        .iter()
        .flat_map(|layer| layer.iter().map(|row| row.len()))
        .max()
        .unwrap_or(0);

    let mut out = vec![vec![StyledChar::plain(' '); width]; height];

    for layer in layers {
        for (y, row) in layer.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if cell.ch != ' ' {
                    out[y][x] = *cell;
                }
            }
        }
    }

    out
}

fn pad_grid(
    grid: Vec<Vec<StyledChar>>,
    target_width: usize,
    target_height: usize,
) -> Vec<Vec<StyledChar>> {
    if grid.is_empty() {
        return vec![vec![StyledChar::plain(' '); target_width]; target_height];
    }

    let content_height = grid.len();
    let content_width = grid.iter().map(|row| row.len()).max().unwrap_or(0);

    let left_pad = target_width.saturating_sub(content_width) / 2;
    let top_pad = target_height.saturating_sub(content_height) / 2;

    let mut out = Vec::with_capacity(target_height);

    for _ in 0..top_pad {
        out.push(vec![StyledChar::plain(' '); target_width]);
    }

    for row in grid {
        let mut padded = Vec::with_capacity(target_width);
        padded.extend(std::iter::repeat_n(StyledChar::plain(' '), left_pad));
        padded.extend(row.into_iter());
        while padded.len() < target_width {
            padded.push(StyledChar::plain(' '));
        }
        out.push(padded);
    }

    while out.len() < target_height {
        out.push(vec![StyledChar::plain(' '); target_width]);
    }

    out
}
