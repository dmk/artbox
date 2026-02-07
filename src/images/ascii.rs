//! Image-to-ASCII rendering utilities.
//!
//! Converts raster (and SVG) images into ASCII art using multiple rendering
//! modes. Outputs are provided as a styled character grid that can be
//! converted to ANSI or plain text.

use std::path::Path;

use image::{DynamicImage, GenericImageView};

use crate::color::ANSI_RESET;
use crate::styled::StyledChar;
use crate::Rgb;

const ASCII_CHARS: &str =
    " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";
const BLOCK_CHARS: [char; 16] = [
    ' ', '▗', '▖', '▄', '▝', '▐', '▞', '▟', '▘', '▚', '▌', '▙', '▀', '▜', '▛', '█',
];

/// Rendering modes for ASCII conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiMode {
    /// Full blocks (`█`) or spaces based on a threshold.
    Full,
    /// 2x2 block patterns (▀▄▌▐) with 16 possible shapes.
    Block,
    /// Shading blocks (░▒▓█) for smooth gradients.
    Shade,
    /// ASCII character ramp.
    Ascii,
}

/// Options for ASCII conversion.
#[derive(Debug, Clone)]
pub struct AsciiOptions {
    /// Output width in characters.
    pub width: u32,
    /// Horizontal scaling factor.
    pub h_scale: f32,
    /// Vertical scaling factor.
    pub v_scale: f32,
    /// Brightness adjustment (-255 to 255).
    pub brightness: i32,
    /// Contrast factor (0.0 to 3.0, where 1.0 is neutral).
    pub contrast: f32,
    /// Sharpness factor (0.0 to 3.0, where 1.0 is neutral).
    pub sharpness: f32,
    /// Rendering mode.
    pub mode: AsciiMode,
    /// Whether to include ANSI colors.
    pub color: bool,
    /// Brightness threshold (0-255) for full/block modes.
    pub threshold: u8,
    /// Invert brightness.
    pub invert: bool,
    /// Alpha threshold for transparency (0-255).
    pub alpha_threshold: u8,
}

impl Default for AsciiOptions {
    fn default() -> Self {
        Self {
            width: 100,
            h_scale: 1.0,
            v_scale: 1.0,
            brightness: 0,
            contrast: 1.0,
            sharpness: 1.0,
            mode: AsciiMode::Full,
            color: true,
            threshold: 128,
            invert: false,
            alpha_threshold: 128,
        }
    }
}

impl AsciiOptions {
    /// Sets output width in characters.
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the rendering mode.
    pub fn with_mode(mut self, mode: AsciiMode) -> Self {
        self.mode = mode;
        self
    }

    /// Enables or disables ANSI colors.
    pub fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }
}

/// The result of an ASCII render operation.
#[derive(Debug, Clone)]
pub struct AsciiRendered {
    /// 2D grid of styled characters (rows of columns).
    pub chars: Vec<Vec<StyledChar>>,
    /// Width of the output.
    pub width: u16,
    /// Height of the output.
    pub height: u16,
}

impl AsciiRendered {
    /// Converts to an ANSI-colored string.
    pub fn to_ansi_string(&self) -> String {
        let mut out = String::new();
        let mut last_color: Option<Rgb> = None;

        for (row_idx, row) in self.chars.iter().enumerate() {
            for sc in row {
                if sc.fg != last_color {
                    if let Some(rgb) = sc.fg {
                        out.push_str(&rgb.to_ansi_fg());
                    } else {
                        out.push_str(ANSI_RESET);
                    }
                    last_color = sc.fg;
                }
                out.push(sc.ch);
            }

            if row_idx < self.chars.len() - 1 {
                out.push('\n');
            }
        }

        if last_color.is_some() {
            out.push_str(ANSI_RESET);
        }

        out
    }

    /// Converts to a plain string without color codes.
    pub fn to_plain_string(&self) -> String {
        let mut out = String::new();
        for (row_idx, row) in self.chars.iter().enumerate() {
            for sc in row {
                out.push(sc.ch);
            }
            if row_idx < self.chars.len() - 1 {
                out.push('\n');
            }
        }
        out
    }

    /// Returns metrics about the rendered output.
    pub fn metrics(&self) -> AsciiMetrics {
        AsciiMetrics {
            width: self.width,
            height: self.height,
        }
    }
}

/// Metrics about an ASCII render.
#[derive(Debug, Clone, Copy)]
pub struct AsciiMetrics {
    pub width: u16,
    pub height: u16,
}

/// Errors that can occur during ASCII rendering.
#[derive(Debug)]
pub enum AsciiError {
    EmptyWidth,
    Io(std::io::Error),
    Decode(image::ImageError),
    Svg(String),
}

impl std::fmt::Display for AsciiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsciiError::EmptyWidth => write!(f, "width must be greater than zero"),
            AsciiError::Io(err) => write!(f, "failed to read image: {err}"),
            AsciiError::Decode(err) => write!(f, "failed to decode image: {err}"),
            AsciiError::Svg(err) => write!(f, "failed to render svg: {err}"),
        }
    }
}

impl std::error::Error for AsciiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AsciiError::Io(err) => Some(err),
            AsciiError::Decode(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AsciiError {
    fn from(err: std::io::Error) -> Self {
        AsciiError::Io(err)
    }
}

/// Render an image from disk to ASCII.
pub fn render_path(
    path: impl AsRef<Path>,
    options: &AsciiOptions,
) -> Result<AsciiRendered, AsciiError> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    if ext.eq_ignore_ascii_case("svg") {
        let bytes = std::fs::read(path)?;
        render_svg_bytes(&bytes, options)
    } else {
        let image = image::open(path).map_err(AsciiError::Decode)?;
        render_image(image, options)
    }
}

/// Render an image from memory to ASCII.
pub fn render_bytes(bytes: &[u8], options: &AsciiOptions) -> Result<AsciiRendered, AsciiError> {
    let image = image::load_from_memory(bytes).map_err(AsciiError::Decode)?;
    render_image(image, options)
}

/// Render a decoded image to ASCII.
pub fn render_image(
    image: DynamicImage,
    options: &AsciiOptions,
) -> Result<AsciiRendered, AsciiError> {
    if options.width == 0 {
        return Err(AsciiError::EmptyWidth);
    }

    let (target_width, target_height) = target_dimensions(&image, options);
    let (rgb, alpha) = split_alpha(&image);

    let adjusted = apply_adjustments(DynamicImage::ImageRgb8(rgb), options);
    let resized_rgb = image::imageops::resize(
        &adjusted.to_rgb8(),
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    );
    let resized_alpha = image::imageops::resize(
        &alpha,
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    );

    let rgba = merge_alpha(&resized_rgb, &resized_alpha);
    let rendered = match options.mode {
        AsciiMode::Full => render_full(&rgba, options),
        AsciiMode::Shade => render_shade(&rgba, options),
        AsciiMode::Ascii => render_ascii(&rgba, options),
        AsciiMode::Block => render_block(&rgba, options),
    };

    Ok(rendered)
}

fn render_svg_bytes(bytes: &[u8], options: &AsciiOptions) -> Result<AsciiRendered, AsciiError> {
    let svg_str = std::str::from_utf8(bytes)
        .map_err(|err| AsciiError::Svg(format!("invalid utf-8 svg: {err}")))?;
    let svg_str = svg_str.replace("currentColor", "black");
    let bytes = svg_str.as_bytes();

    let target_width = (options.width as f32 * 4.0).round().max(1.0) as u32;
    let image = render_svg_to_image(bytes, target_width)?;
    render_image(image, options)
}

fn render_svg_to_image(bytes: &[u8], target_width: u32) -> Result<DynamicImage, AsciiError> {
    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let tree = resvg::usvg::Tree::from_data(bytes, &opt)
        .map_err(|err| AsciiError::Svg(err.to_string()))?;

    let size = tree.size();
    let scale = target_width as f32 / size.width();
    let target_height = (size.height() * scale).round().max(1.0) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(target_width, target_height)
        .ok_or_else(|| AsciiError::Svg("failed to allocate pixmap for svg".to_string()))?;

    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap_mut,
    );

    let image = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .ok_or_else(|| AsciiError::Svg("failed to build rgba buffer from svg".to_string()))?;

    Ok(DynamicImage::ImageRgba8(image))
}

fn target_dimensions(image: &DynamicImage, options: &AsciiOptions) -> (u32, u32) {
    let (w, h) = image.dimensions();
    let aspect_ratio = h as f32 / w as f32;

    let base_width = options.width as f32;
    let target_width = (base_width * options.h_scale).round().max(1.0) as u32;
    let target_height = (aspect_ratio * base_width * 0.5 * options.v_scale)
        .round()
        .max(1.0) as u32;

    (target_width, target_height)
}

fn split_alpha(image: &DynamicImage) -> (image::RgbImage, image::GrayImage) {
    let rgba = image.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut rgb = image::RgbImage::new(w, h);
    let mut alpha = image::GrayImage::new(w, h);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        rgb.put_pixel(x, y, image::Rgb([pixel[0], pixel[1], pixel[2]]));
        alpha.put_pixel(x, y, image::Luma([pixel[3]]));
    }

    (rgb, alpha)
}

fn merge_alpha(rgb: &image::RgbImage, alpha: &image::GrayImage) -> image::RgbaImage {
    let (w, h) = rgb.dimensions();
    let mut rgba = image::RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let p = rgb.get_pixel(x, y);
            let a = alpha.get_pixel(x, y)[0];
            rgba.put_pixel(x, y, image::Rgba([p[0], p[1], p[2], a]));
        }
    }

    rgba
}

fn apply_adjustments(image: DynamicImage, options: &AsciiOptions) -> DynamicImage {
    let mut img = image;

    if options.brightness != 0 {
        img = img.brighten(options.brightness);
    }

    if (options.contrast - 1.0).abs() > f32::EPSILON {
        let contrast = (options.contrast - 1.0) * 100.0;
        img = img.adjust_contrast(contrast);
    }

    if (options.sharpness - 1.0).abs() > f32::EPSILON {
        if options.sharpness < 1.0 {
            let sigma = (1.0 - options.sharpness) * 2.0;
            if sigma > 0.0 {
                img = img.blur(sigma);
            }
        } else {
            let sigma = (options.sharpness - 1.0) * 2.0;
            if sigma > 0.0 {
                img = img.unsharpen(sigma, 1);
            }
        }
    }

    img
}

fn render_full(rgba: &image::RgbaImage, options: &AsciiOptions) -> AsciiRendered {
    let (w, h) = rgba.dimensions();
    let mut chars = Vec::with_capacity(h as usize);

    for y in 0..h {
        let mut row = Vec::with_capacity(w as usize);
        for x in 0..w {
            let pixel = rgba.get_pixel(x, y);
            if is_transparent(pixel, options.alpha_threshold) {
                row.push(StyledChar::plain(' '));
                continue;
            }

            let mut brightness = pixel_brightness(pixel);
            if options.invert {
                brightness = 255 - brightness;
            }

            let ch = if brightness >= options.threshold {
                '█'
            } else {
                ' '
            };
            row.push(styled_char(ch, pixel, options));
        }
        chars.push(row);
    }

    AsciiRendered {
        chars,
        width: w as u16,
        height: h as u16,
    }
}

fn render_shade(rgba: &image::RgbaImage, options: &AsciiOptions) -> AsciiRendered {
    let (w, h) = rgba.dimensions();
    let mut chars = Vec::with_capacity(h as usize);

    for y in 0..h {
        let mut row = Vec::with_capacity(w as usize);
        for x in 0..w {
            let pixel = rgba.get_pixel(x, y);
            if is_transparent(pixel, options.alpha_threshold) {
                row.push(StyledChar::plain(' '));
                continue;
            }

            let mut brightness = pixel_brightness(pixel);
            if options.invert {
                brightness = 255 - brightness;
            }

            let ch = match brightness {
                0..=31 => ' ',
                32..=63 => '░',
                64..=127 => '▒',
                128..=191 => '▓',
                _ => '█',
            };
            row.push(styled_char(ch, pixel, options));
        }
        chars.push(row);
    }

    AsciiRendered {
        chars,
        width: w as u16,
        height: h as u16,
    }
}

fn render_ascii(rgba: &image::RgbaImage, options: &AsciiOptions) -> AsciiRendered {
    let (w, h) = rgba.dimensions();
    let mut chars = Vec::with_capacity(h as usize);
    let ramp = ASCII_CHARS.as_bytes();
    let ramp_len = ramp.len() as u32;

    for y in 0..h {
        let mut row = Vec::with_capacity(w as usize);
        for x in 0..w {
            let pixel = rgba.get_pixel(x, y);
            if is_transparent(pixel, options.alpha_threshold) {
                row.push(StyledChar::plain(' '));
                continue;
            }

            let mut brightness = pixel_brightness(pixel);
            if options.invert {
                brightness = 255 - brightness;
            }
            let idx = (brightness as u32 * ramp_len / 256) as usize;
            let ch = ramp[idx] as char;
            row.push(styled_char(ch, pixel, options));
        }
        chars.push(row);
    }

    AsciiRendered {
        chars,
        width: w as u16,
        height: h as u16,
    }
}

fn render_block(rgba: &image::RgbaImage, options: &AsciiOptions) -> AsciiRendered {
    let (w, h) = rgba.dimensions();
    let out_w = w.div_ceil(2);
    let out_h = h.div_ceil(2);
    let mut chars = Vec::with_capacity(out_h as usize);

    let mut y = 0;
    while y < h {
        let mut row = Vec::with_capacity(out_w as usize);
        let mut x = 0;
        while x < w {
            let pixels = [
                get_pixel(rgba, x, y, options.alpha_threshold),
                get_pixel(rgba, x + 1, y, options.alpha_threshold),
                get_pixel(rgba, x, y + 1, options.alpha_threshold),
                get_pixel(rgba, x + 1, y + 1, options.alpha_threshold),
            ];

            let mut mask = 0u8;
            for (idx, pixel) in pixels.iter().enumerate() {
                if let Some(pixel) = pixel {
                    let mut brightness = pixel_brightness(pixel);
                    if options.invert {
                        brightness = 255 - brightness;
                    }
                    if brightness >= options.threshold {
                        mask |= 1 << (3 - idx);
                    }
                }
            }

            let ch = BLOCK_CHARS[mask as usize];
            let color = if options.color {
                average_color(&pixels)
            } else {
                None
            };

            let cell = if ch == ' ' {
                StyledChar::plain(' ')
            } else {
                StyledChar::new(ch, color)
            };

            row.push(cell);
            x += 2;
        }
        chars.push(row);
        y += 2;
    }

    AsciiRendered {
        chars,
        width: out_w as u16,
        height: out_h as u16,
    }
}

fn get_pixel(
    rgba: &image::RgbaImage,
    x: u32,
    y: u32,
    alpha_threshold: u8,
) -> Option<image::Rgba<u8>> {
    if x < rgba.width() && y < rgba.height() {
        let pixel = *rgba.get_pixel(x, y);
        if pixel[3] < alpha_threshold {
            None
        } else {
            Some(pixel)
        }
    } else {
        None
    }
}

fn is_transparent(pixel: &image::Rgba<u8>, threshold: u8) -> bool {
    pixel[3] < threshold
}

fn pixel_brightness(pixel: &image::Rgba<u8>) -> u8 {
    let r = pixel[0] as u32;
    let g = pixel[1] as u32;
    let b = pixel[2] as u32;
    let value = 299 * r + 587 * g + 114 * b;
    (value / 1000) as u8
}

fn styled_char(ch: char, pixel: &image::Rgba<u8>, options: &AsciiOptions) -> StyledChar {
    if ch == ' ' {
        return StyledChar::plain(' ');
    }
    if !options.color {
        return StyledChar::plain(ch);
    }

    let rgb = Rgb::new(pixel[0], pixel[1], pixel[2]);
    StyledChar::new(ch, Some(rgb))
}

fn average_color(pixels: &[Option<image::Rgba<u8>>; 4]) -> Option<Rgb> {
    let mut r_sum = 0u32;
    let mut g_sum = 0u32;
    let mut b_sum = 0u32;
    let mut count = 0u32;

    for pixel in pixels.iter().flatten() {
        r_sum += pixel[0] as u32;
        g_sum += pixel[1] as u32;
        b_sum += pixel[2] as u32;
        count += 1;
    }

    if count == 0 {
        None
    } else {
        Some(Rgb::new(
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_image(r: u8, g: u8, b: u8) -> DynamicImage {
        let mut img = image::RgbImage::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                img.put_pixel(x, y, image::Rgb([r, g, b]));
            }
        }
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn ascii_render_dimensions() {
        let img = solid_image(255, 0, 0);
        let options = AsciiOptions::default()
            .with_width(4)
            .with_mode(AsciiMode::Ascii);
        let rendered = render_image(img, &options).unwrap();
        assert!(rendered.width > 0);
        assert!(rendered.height > 0);
    }

    #[test]
    fn block_mode_uses_quadrant_chars() {
        let mut img = image::RgbaImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
        img.put_pixel(1, 0, image::Rgba([0, 0, 0, 0]));
        img.put_pixel(0, 1, image::Rgba([0, 0, 0, 0]));
        img.put_pixel(1, 1, image::Rgba([0, 0, 0, 0]));
        let img = DynamicImage::ImageRgba8(img);

        let options = AsciiOptions {
            mode: AsciiMode::Block,
            threshold: 128,
            color: false,
            width: 2,
            v_scale: 2.0,
            ..AsciiOptions::default()
        };

        let rendered = render_image(img, &options).unwrap();
        let output = rendered.to_plain_string();
        assert_eq!(output.trim(), "▘");
    }
}
