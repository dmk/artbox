//! Terminal image support for kitty and iTerm2-style inline images.
//!
//! This module provides best-effort terminal capability detection along with
//! helpers for rendering images into terminal escape sequences. The API is
//! designed to allow explicit overrides when detection is insufficient.
//!
//! ## Example
//! ```rust,no_run
//! use artbox::images::{TerminalImageConfig, TerminalImageMode, render_image_path};
//!
//! let config = TerminalImageConfig::default()
//!     .with_mode(TerminalImageMode::Auto)
//!     .with_size(Some(24), Some(12));
//!
//! let img = render_image_path("image.png", config)?;
//! print!("{}", img.as_str());
//! # Ok::<(), artbox::images::ImageError>(())
//! ```

use std::env;
use std::io::Cursor;
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::DynamicImage;

use crate::{GridRendered, RenderTarget, Rendered};

const KITTY_CHUNK_SIZE: usize = 4096;

pub mod ascii;

/// Supported terminal image protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageProtocol {
    Kitty,
    Iterm2,
}

/// How to choose image output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageOutput {
    /// Prefer terminal image output when supported, otherwise ASCII.
    #[default]
    Auto,
    /// Force terminal image output (kitty/iTerm2).
    Terminal,
    /// Force ASCII output.
    Ascii,
}

/// Result of terminal capability detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalImageSupport {
    Unsupported,
    Kitty,
    Iterm2,
}

impl TerminalImageSupport {
    /// Returns the matching image protocol, if supported.
    pub fn protocol(self) -> Option<ImageProtocol> {
        match self {
            TerminalImageSupport::Kitty => Some(ImageProtocol::Kitty),
            TerminalImageSupport::Iterm2 => Some(ImageProtocol::Iterm2),
            TerminalImageSupport::Unsupported => None,
        }
    }
}

/// How to choose the image protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalImageMode {
    /// Auto-detect support (best effort).
    Auto,
    /// Disable image output.
    Disabled,
    /// Force kitty graphics protocol.
    Kitty,
    /// Force iTerm2 inline images.
    Iterm2,
}

impl TerminalImageMode {
    fn resolve(self, support: TerminalImageSupport) -> Option<ImageProtocol> {
        match self {
            TerminalImageMode::Auto => support.protocol(),
            TerminalImageMode::Disabled => None,
            TerminalImageMode::Kitty => Some(ImageProtocol::Kitty),
            TerminalImageMode::Iterm2 => Some(ImageProtocol::Iterm2),
        }
    }
}

/// Configuration for terminal image rendering.
#[derive(Debug, Clone, Copy)]
pub struct TerminalImageConfig {
    /// Protocol selection mode.
    pub mode: TerminalImageMode,
    /// Optional width in terminal cells.
    pub width: Option<u16>,
    /// Optional height in terminal cells.
    pub height: Option<u16>,
    /// Preserve aspect ratio when supported by the protocol.
    pub preserve_aspect_ratio: bool,
    /// Whether to move the cursor after rendering (kitty only).
    pub move_cursor: bool,
}

impl Default for TerminalImageConfig {
    fn default() -> Self {
        Self {
            mode: TerminalImageMode::Auto,
            width: None,
            height: None,
            preserve_aspect_ratio: true,
            move_cursor: false,
        }
    }
}

impl TerminalImageConfig {
    /// Sets the protocol selection mode.
    pub fn with_mode(mut self, mode: TerminalImageMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the size in terminal cells.
    pub fn with_size(mut self, width: Option<u16>, height: Option<u16>) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets whether to preserve aspect ratio (when supported).
    pub fn with_preserve_aspect_ratio(mut self, preserve: bool) -> Self {
        self.preserve_aspect_ratio = preserve;
        self
    }

    /// Sets whether to move the cursor after rendering (kitty only).
    pub fn with_move_cursor(mut self, move_cursor: bool) -> Self {
        self.move_cursor = move_cursor;
        self
    }
}

/// Rendered terminal image escape sequence.
#[derive(Debug, Clone)]
pub struct TerminalImage {
    /// Protocol used to render the image.
    pub protocol: ImageProtocol,
    payload: String,
}

impl TerminalImage {
    /// Returns the escape sequence as a string slice.
    pub fn as_str(&self) -> &str {
        &self.payload
    }

    /// Consumes the image and returns the escape sequence.
    pub fn into_string(self) -> String {
        self.payload
    }
}

impl std::fmt::Display for TerminalImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.payload)
    }
}

/// Errors that can occur while rendering terminal images.
#[derive(Debug)]
pub enum ImageError {
    UnsupportedTerminal,
    Io(std::io::Error),
    Decode(image::ImageError),
    Encode(image::ImageError),
    Svg(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::UnsupportedTerminal => {
                write!(f, "terminal does not support inline images")
            }
            ImageError::Io(err) => write!(f, "failed to read image: {err}"),
            ImageError::Decode(err) => write!(f, "failed to decode image: {err}"),
            ImageError::Encode(err) => write!(f, "failed to encode image: {err}"),
            ImageError::Svg(err) => write!(f, "failed to render svg: {err}"),
        }
    }
}

impl std::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ImageError::Io(err) => Some(err),
            ImageError::Decode(err) => Some(err),
            ImageError::Encode(err) => Some(err),
            ImageError::Svg(_) => None,
            ImageError::UnsupportedTerminal => None,
        }
    }
}

impl From<std::io::Error> for ImageError {
    fn from(err: std::io::Error) -> Self {
        ImageError::Io(err)
    }
}

/// Errors that can occur while rendering images into unified outputs.
#[derive(Debug)]
pub enum ImageRenderError {
    Terminal(ImageError),
    Ascii(ascii::AsciiError),
}

impl std::fmt::Display for ImageRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageRenderError::Terminal(err) => write!(f, "{err}"),
            ImageRenderError::Ascii(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ImageRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ImageRenderError::Terminal(err) => Some(err),
            ImageRenderError::Ascii(err) => Some(err),
        }
    }
}

/// Detects terminal support for inline image protocols.
///
/// This uses environment variables for best-effort detection. It is not
/// authoritative; callers should allow explicit overrides via
/// [`TerminalImageConfig::with_mode`].
pub fn detect_terminal_image_support() -> TerminalImageSupport {
    let term = env::var("TERM").ok().unwrap_or_default();
    let term_lower = term.to_lowercase();
    if term_lower.contains("kitty") || env::var("KITTY_WINDOW_ID").is_ok() {
        return TerminalImageSupport::Kitty;
    }

    let term_program = env::var("TERM_PROGRAM").ok().unwrap_or_default();
    if term_program.eq_ignore_ascii_case("ghostty")
        || term_lower.contains("ghostty")
        || env::var("GHOSTTY").is_ok()
    {
        return TerminalImageSupport::Kitty;
    }

    if term_program == "iTerm.app"
        || env::var("ITERM_SESSION_ID").is_ok()
        || env::var("LC_TERMINAL").ok().as_deref() == Some("iTerm2")
    {
        return TerminalImageSupport::Iterm2;
    }

    TerminalImageSupport::Unsupported
}

/// Renders an image from a file path into an inline image escape sequence.
pub fn render_image_path(
    path: impl AsRef<Path>,
    config: TerminalImageConfig,
) -> Result<TerminalImage, ImageError> {
    let svg_options = SvgRasterOptions {
        target_width: svg_target_width_from_config(config),
        current_color: SvgColor::Black,
    };
    let image = load_image_from_path(path.as_ref(), Some(svg_options))?;
    render_image(&image, config)
}

/// Renders an image from raw bytes into an inline image escape sequence.
pub fn render_image_bytes(
    bytes: &[u8],
    config: TerminalImageConfig,
) -> Result<TerminalImage, ImageError> {
    let svg_options = SvgRasterOptions {
        target_width: svg_target_width_from_config(config),
        current_color: SvgColor::Black,
    };
    let image = load_image_from_bytes(bytes, Some(svg_options))?;
    render_image(&image, config)
}

/// Renders an image path into a unified output based on the requested mode.
pub fn render_image_auto_path(
    path: impl AsRef<Path>,
    target: RenderTarget,
    output: ImageOutput,
    ascii_options: &ascii::AsciiOptions,
    terminal_config: TerminalImageConfig,
) -> Result<Rendered, ImageRenderError> {
    match output {
        ImageOutput::Ascii => render_ascii_path(path, target, ascii_options),
        ImageOutput::Terminal => render_terminal_path(path, target, terminal_config),
        ImageOutput::Auto => match detect_terminal_image_support() {
            TerminalImageSupport::Unsupported => render_ascii_path(path, target, ascii_options),
            _ => render_terminal_path(path, target, terminal_config),
        },
    }
}

/// Renders image bytes into a unified output based on the requested mode.
pub fn render_image_auto_bytes(
    bytes: &[u8],
    target: RenderTarget,
    output: ImageOutput,
    ascii_options: &ascii::AsciiOptions,
    terminal_config: TerminalImageConfig,
) -> Result<Rendered, ImageRenderError> {
    match output {
        ImageOutput::Ascii => render_ascii_bytes(bytes, target, ascii_options),
        ImageOutput::Terminal => render_terminal_bytes(bytes, target, terminal_config),
        ImageOutput::Auto => match detect_terminal_image_support() {
            TerminalImageSupport::Unsupported => render_ascii_bytes(bytes, target, ascii_options),
            _ => render_terminal_bytes(bytes, target, terminal_config),
        },
    }
}

fn render_terminal_path(
    path: impl AsRef<Path>,
    target: RenderTarget,
    config: TerminalImageConfig,
) -> Result<Rendered, ImageRenderError> {
    let config = config.with_size(Some(target.width), Some(target.height));
    let image = render_image_path(path, config).map_err(ImageRenderError::Terminal)?;
    Ok(Rendered::TerminalImage {
        image,
        fallback: None,
    })
}

fn render_terminal_bytes(
    bytes: &[u8],
    target: RenderTarget,
    config: TerminalImageConfig,
) -> Result<Rendered, ImageRenderError> {
    let config = config.with_size(Some(target.width), Some(target.height));
    let image = render_image_bytes(bytes, config).map_err(ImageRenderError::Terminal)?;
    Ok(Rendered::TerminalImage {
        image,
        fallback: None,
    })
}

fn render_ascii_path(
    path: impl AsRef<Path>,
    target: RenderTarget,
    ascii_options: &ascii::AsciiOptions,
) -> Result<Rendered, ImageRenderError> {
    let mut options = ascii_options.clone();
    options.width = target.width as u32;
    let rendered = ascii::render_image_path(path, &options).map_err(ImageRenderError::Ascii)?;
    Ok(Rendered::Grid(GridRendered::from(rendered)))
}

fn render_ascii_bytes(
    bytes: &[u8],
    target: RenderTarget,
    ascii_options: &ascii::AsciiOptions,
) -> Result<Rendered, ImageRenderError> {
    let mut options = ascii_options.clone();
    options.width = target.width as u32;
    let rendered = ascii::render_image_bytes(bytes, &options).map_err(ImageRenderError::Ascii)?;
    Ok(Rendered::Grid(GridRendered::from(rendered)))
}

/// Renders a decoded image into an inline image escape sequence.
pub fn render_image(
    image: &DynamicImage,
    config: TerminalImageConfig,
) -> Result<TerminalImage, ImageError> {
    let protocol = config
        .mode
        .resolve(detect_terminal_image_support())
        .ok_or(ImageError::UnsupportedTerminal)?;

    let png_bytes = encode_png(image)?;
    let payload = match protocol {
        ImageProtocol::Kitty => render_kitty(&png_bytes, config),
        ImageProtocol::Iterm2 => render_iterm2(&png_bytes, config),
    };

    Ok(TerminalImage { protocol, payload })
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, ImageError> {
    let mut out = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(ImageError::Encode)?;
    Ok(out)
}

fn svg_target_width_from_config(config: TerminalImageConfig) -> u32 {
    if let Some(width) = config.width {
        width as u32 * 8
    } else if let Some(height) = config.height {
        height as u32 * 16
    } else {
        256
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SvgColor {
    Black,
    White,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SvgRasterOptions {
    pub target_width: u32,
    pub current_color: SvgColor,
}

impl Default for SvgRasterOptions {
    fn default() -> Self {
        Self {
            target_width: 256,
            current_color: SvgColor::Black,
        }
    }
}

fn is_svg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("svg"))
        .unwrap_or(false)
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let Ok(svg_str) = std::str::from_utf8(bytes) else {
        return false;
    };
    let head: String = svg_str
        .trim_start_matches(|ch: char| ch.is_whitespace() || ch == '\u{feff}')
        .chars()
        .take(4096)
        .collect();
    head.to_ascii_lowercase().contains("<svg")
}

pub(crate) fn load_image_from_path(
    path: &Path,
    svg_options: Option<SvgRasterOptions>,
) -> Result<DynamicImage, ImageError> {
    if is_svg_path(path) {
        let options = svg_options.unwrap_or_default();
        let bytes = std::fs::read(path)?;
        render_svg_bytes(&bytes, options).map_err(ImageError::Svg)
    } else {
        image::open(path).map_err(ImageError::Decode)
    }
}

pub(crate) fn load_image_from_bytes(
    bytes: &[u8],
    svg_options: Option<SvgRasterOptions>,
) -> Result<DynamicImage, ImageError> {
    if looks_like_svg(bytes) {
        let options = svg_options.unwrap_or_default();
        render_svg_bytes(bytes, options).map_err(ImageError::Svg)
    } else {
        image::load_from_memory(bytes).map_err(ImageError::Decode)
    }
}

pub(crate) fn render_svg_bytes(
    bytes: &[u8],
    options: SvgRasterOptions,
) -> Result<DynamicImage, String> {
    let svg_str = std::str::from_utf8(bytes).map_err(|err| format!("invalid utf-8 svg: {err}"))?;
    let replacement = match options.current_color {
        SvgColor::Black => "black",
        SvgColor::White => "white",
    };
    let svg_str = svg_str.replace("currentColor", replacement);

    let mut opt = resvg::usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let tree =
        resvg::usvg::Tree::from_data(svg_str.as_bytes(), &opt).map_err(|err| err.to_string())?;

    let size = tree.size();
    let scale = options.target_width as f32 / size.width();
    let target_height = (size.height() * scale).round().max(1.0) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(options.target_width, target_height)
        .ok_or_else(|| "failed to allocate pixmap for svg".to_string())?;

    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap_mut,
    );

    let image = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .ok_or_else(|| "failed to build rgba buffer from svg".to_string())?;

    Ok(DynamicImage::ImageRgba8(image))
}

fn render_kitty(png_bytes: &[u8], config: TerminalImageConfig) -> String {
    let mut params = vec!["a=T".to_string(), "f=100".to_string()];
    if let Some(cols) = config.width {
        params.push(format!("c={cols}"));
    }
    if let Some(rows) = config.height {
        params.push(format!("r={rows}"));
    }
    if !config.move_cursor {
        params.push("C=1".to_string());
    }

    let encoded = STANDARD.encode(png_bytes);
    let mut out = String::new();
    let mut offset = 0;
    let mut first = true;

    while offset < encoded.len() {
        let end = (offset + KITTY_CHUNK_SIZE).min(encoded.len());
        let chunk = &encoded[offset..end];
        let more = end < encoded.len();

        let control = if first {
            format!("{},m={}", params.join(","), if more { 1 } else { 0 })
        } else {
            format!("m={}", if more { 1 } else { 0 })
        };

        out.push_str("\x1b_G");
        out.push_str(&control);
        out.push(';');
        out.push_str(chunk);
        out.push_str("\x1b\\");

        first = false;
        offset = end;
    }

    out
}

fn render_iterm2(png_bytes: &[u8], config: TerminalImageConfig) -> String {
    let mut attrs = Vec::new();
    attrs.push("inline=1".to_string());
    attrs.push(format!("size={}", png_bytes.len()));
    if let Some(width) = config.width {
        attrs.push(format!("width={width}"));
    }
    if let Some(height) = config.height {
        attrs.push(format!("height={height}"));
    }
    attrs.push(format!(
        "preserveAspectRatio={}",
        if config.preserve_aspect_ratio { 1 } else { 0 }
    ));

    let encoded = STANDARD.encode(png_bytes);
    let mut out = String::new();
    out.push_str("\x1b]1337;File=");
    out.push_str(&attrs.join(";"));
    out.push(':');
    out.push_str(&encoded);
    out.push_str("\x1b\\");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_image() -> DynamicImage {
        let mut img = image::RgbImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn kitty_payload_contains_control_prefix() {
        let image = tiny_image();
        let config = TerminalImageConfig::default().with_mode(TerminalImageMode::Kitty);
        let rendered = render_image(&image, config).unwrap();
        assert!(rendered.as_str().starts_with("\x1b_Ga=T,f=100"));
        assert!(rendered.as_str().contains("C=1"));
    }

    #[test]
    fn iterm_payload_contains_file_prefix() {
        let image = tiny_image();
        let config = TerminalImageConfig::default().with_mode(TerminalImageMode::Iterm2);
        let rendered = render_image(&image, config).unwrap();
        assert!(rendered.as_str().starts_with("\x1b]1337;File=inline=1"));
        assert!(rendered.as_str().contains("preserveAspectRatio=1"));
    }
}
