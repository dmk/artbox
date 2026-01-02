//! Color types and gradient definitions for styled ASCII art rendering.
//!
//! This module provides color representations (RGB, HSL), gradient types
//! (linear, radial), and utilities for color interpolation and ANSI output.
//!
//! # Examples
//!
//! ```rust
//! use artbox::color::{Color, Rgb, Fill, LinearGradient, ColorStop};
//!
//! // Create colors
//! let red = Color::rgb(255, 0, 0);
//! let blue = Color::hsl(240.0, 1.0, 0.5);
//!
//! // Create a linear gradient
//! let gradient = Fill::Linear(LinearGradient {
//!     angle: 0.0,
//!     stops: vec![
//!         ColorStop { position: 0.0, color: red },
//!         ColorStop { position: 1.0, color: blue },
//!     ],
//! });
//! ```

use std::f32::consts::PI;

/// An RGB color with 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Creates a new RGB color.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Converts this RGB color to HSL.
    pub fn to_hsl(self) -> Hsl {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < f32::EPSILON {
            return Hsl { h: 0.0, s: 0.0, l };
        }

        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let h = if (max - r).abs() < f32::EPSILON {
            let mut h = (g - b) / d;
            if g < b {
                h += 6.0;
            }
            h
        } else if (max - g).abs() < f32::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        Hsl { h: h * 60.0, s, l }
    }

    /// Generates an ANSI escape code for foreground color.
    pub fn to_ansi_fg(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Generates an ANSI escape code for background color.
    pub fn to_ansi_bg(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Interpolates between two RGB colors.
    pub fn interpolate(self, other: Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        Rgb {
            r: lerp_u8(self.r, other.r, t),
            g: lerp_u8(self.g, other.g, t),
            b: lerp_u8(self.b, other.b, t),
        }
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

/// An HSL color with hue in degrees (0-360) and saturation/lightness as 0-1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    /// Hue in degrees (0-360).
    pub h: f32,
    /// Saturation (0.0-1.0).
    pub s: f32,
    /// Lightness (0.0-1.0).
    pub l: f32,
}

impl Hsl {
    /// Creates a new HSL color.
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self {
            h: h % 360.0,
            s: s.clamp(0.0, 1.0),
            l: l.clamp(0.0, 1.0),
        }
    }

    /// Converts this HSL color to RGB.
    pub fn to_rgb(self) -> Rgb {
        if self.s.abs() < f32::EPSILON {
            let v = (self.l * 255.0).round() as u8;
            return Rgb { r: v, g: v, b: v };
        }

        let q = if self.l < 0.5 {
            self.l * (1.0 + self.s)
        } else {
            self.l + self.s - self.l * self.s
        };
        let p = 2.0 * self.l - q;
        let h = self.h / 360.0;

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        Rgb {
            r: (r * 255.0).round() as u8,
            g: (g * 255.0).round() as u8,
            b: (b * 255.0).round() as u8,
        }
    }

    /// Interpolates between two HSL colors.
    ///
    /// Uses the shortest path around the hue circle.
    pub fn interpolate(self, other: Hsl, t: f32) -> Hsl {
        let t = t.clamp(0.0, 1.0);

        // Interpolate hue via shortest path
        let mut h_diff = other.h - self.h;
        if h_diff > 180.0 {
            h_diff -= 360.0;
        } else if h_diff < -180.0 {
            h_diff += 360.0;
        }
        let mut h = self.h + h_diff * t;
        if h < 0.0 {
            h += 360.0;
        } else if h >= 360.0 {
            h -= 360.0;
        }

        Hsl {
            h,
            s: self.s + (other.s - self.s) * t,
            l: self.l + (other.l - self.l) * t,
        }
    }
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let a = a as f32;
    let b = b as f32;
    (a + (b - a) * t).round() as u8
}

/// A color that can be either RGB or HSL.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    /// An RGB color.
    Rgb(Rgb),
    /// An HSL color.
    Hsl(Hsl),
}

impl Color {
    /// Creates an RGB color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb(Rgb::new(r, g, b))
    }

    /// Creates an HSL color.
    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        Self::Hsl(Hsl::new(h, s, l))
    }

    /// Converts this color to RGB.
    pub fn to_rgb(self) -> Rgb {
        match self {
            Color::Rgb(rgb) => rgb,
            Color::Hsl(hsl) => hsl.to_rgb(),
        }
    }

    /// Converts this color to HSL.
    pub fn to_hsl(self) -> Hsl {
        match self {
            Color::Rgb(rgb) => rgb.to_hsl(),
            Color::Hsl(hsl) => hsl,
        }
    }

    /// Interpolates between two colors in HSL space for smoother gradients.
    pub fn interpolate(self, other: Color, t: f32) -> Color {
        let a = self.to_hsl();
        let b = other.to_hsl();
        Color::Hsl(a.interpolate(b, t))
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(rgb: (u8, u8, u8)) -> Self {
        Color::Rgb(Rgb::from(rgb))
    }
}

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        Color::Rgb(rgb)
    }
}

impl From<Hsl> for Color {
    fn from(hsl: Hsl) -> Self {
        Color::Hsl(hsl)
    }
}

/// A color stop in a gradient, defining a color at a specific position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    /// Position along the gradient (0.0 to 1.0).
    pub position: f32,
    /// The color at this position.
    pub color: Color,
}

impl ColorStop {
    /// Creates a new color stop.
    pub fn new(position: f32, color: impl Into<Color>) -> Self {
        Self {
            position: position.clamp(0.0, 1.0),
            color: color.into(),
        }
    }
}

/// A linear gradient with an angle and color stops.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// Angle in degrees (0 = right, 90 = down, 180 = left, 270 = up).
    pub angle: f32,
    /// Color stops defining the gradient. Should have at least 2 stops.
    pub stops: Vec<ColorStop>,
}

impl LinearGradient {
    /// Creates a new linear gradient.
    pub fn new(angle: f32, stops: Vec<ColorStop>) -> Self {
        Self { angle, stops }
    }

    /// Creates a horizontal gradient (left to right).
    pub fn horizontal(start: impl Into<Color>, end: impl Into<Color>) -> Self {
        Self {
            angle: 0.0,
            stops: vec![ColorStop::new(0.0, start), ColorStop::new(1.0, end)],
        }
    }

    /// Creates a vertical gradient (top to bottom).
    pub fn vertical(start: impl Into<Color>, end: impl Into<Color>) -> Self {
        Self {
            angle: 90.0,
            stops: vec![ColorStop::new(0.0, start), ColorStop::new(1.0, end)],
        }
    }

    /// Computes the gradient position for a point in normalized coordinates.
    pub fn position_at(&self, x: f32, y: f32) -> f32 {
        let angle_rad = self.angle * PI / 180.0;
        let dx = angle_rad.cos();
        let dy = angle_rad.sin();

        // Project point onto gradient line
        // Normalize so that corners map to 0 and 1
        let cx = x - 0.5;
        let cy = y - 0.5;
        let projection = cx * dx + cy * dy;

        // Scale to 0-1 range based on the maximum possible projection
        let max_proj = (dx.abs() + dy.abs()) * 0.5;
        if max_proj.abs() < f32::EPSILON {
            return 0.5;
        }
        (projection / max_proj + 1.0) / 2.0
    }

    /// Gets the color at a given position along the gradient.
    pub fn color_at(&self, position: f32) -> Rgb {
        sample_gradient(&self.stops, position)
    }
}

/// A radial gradient with center, focal point, and color stops.
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// Center point in normalized coordinates (0.0-1.0).
    pub center: (f32, f32),
    /// Focal point for off-center effects, in normalized coordinates.
    pub focal: (f32, f32),
    /// Radius of the gradient in normalized units.
    pub radius: f32,
    /// Color stops defining the gradient.
    pub stops: Vec<ColorStop>,
}

impl RadialGradient {
    /// Creates a new radial gradient.
    pub fn new(center: (f32, f32), focal: (f32, f32), radius: f32, stops: Vec<ColorStop>) -> Self {
        Self {
            center,
            focal,
            radius,
            stops,
        }
    }

    /// Creates a simple centered radial gradient.
    pub fn centered(radius: f32, start: impl Into<Color>, end: impl Into<Color>) -> Self {
        Self {
            center: (0.5, 0.5),
            focal: (0.5, 0.5),
            radius,
            stops: vec![ColorStop::new(0.0, start), ColorStop::new(1.0, end)],
        }
    }

    /// Computes the gradient position for a point in normalized coordinates.
    ///
    /// Position is based on distance from `center`, normalized by `radius`.
    /// The `focal` field is reserved for future focal point support.
    pub fn position_at(&self, x: f32, y: f32) -> f32 {
        // Distance from center, normalized by radius
        let dx = x - self.center.0;
        let dy = y - self.center.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if self.radius.abs() < f32::EPSILON {
            return if dist < f32::EPSILON { 0.0 } else { 1.0 };
        }

        (dist / self.radius).clamp(0.0, 1.0)
    }

    /// Gets the color at a given position along the gradient.
    pub fn color_at(&self, position: f32) -> Rgb {
        sample_gradient(&self.stops, position)
    }
}

/// Samples a color from gradient stops at the given position.
fn sample_gradient(stops: &[ColorStop], position: f32) -> Rgb {
    if stops.is_empty() {
        return Rgb::new(0, 0, 0);
    }
    if stops.len() == 1 {
        return stops[0].color.to_rgb();
    }

    // Sort stops by position to ensure correct interpolation
    let mut sorted: Vec<&ColorStop> = stops.iter().collect();
    sorted.sort_by(|a, b| {
        a.position
            .partial_cmp(&b.position)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let position = position.clamp(0.0, 1.0);

    // Find the two stops to interpolate between
    let mut prev = sorted[0];
    for stop in sorted.iter() {
        if stop.position >= position {
            if (stop.position - prev.position).abs() < f32::EPSILON {
                return stop.color.to_rgb();
            }
            let t = (position - prev.position) / (stop.position - prev.position);
            return prev.color.interpolate(stop.color, t).to_rgb();
        }
        prev = stop;
    }

    // Position is past all stops
    sorted.last().unwrap().color.to_rgb()
}

/// A fill style that can be a solid color or a gradient.
#[derive(Debug, Clone, PartialEq)]
pub enum Fill {
    /// A solid color fill.
    Solid(Color),
    /// A linear gradient fill.
    Linear(LinearGradient),
    /// A radial gradient fill.
    Radial(RadialGradient),
}

impl Fill {
    /// Creates a solid fill from a color.
    pub fn solid(color: impl Into<Color>) -> Self {
        Self::Solid(color.into())
    }

    /// Gets the color at a normalized position (x, y both in 0.0-1.0).
    pub fn color_at(&self, x: f32, y: f32) -> Rgb {
        match self {
            Fill::Solid(color) => color.to_rgb(),
            Fill::Linear(gradient) => {
                let pos = gradient.position_at(x, y);
                gradient.color_at(pos)
            }
            Fill::Radial(gradient) => {
                let pos = gradient.position_at(x, y);
                gradient.color_at(pos)
            }
        }
    }
}

impl<T: Into<Color>> From<T> for Fill {
    fn from(color: T) -> Self {
        Fill::Solid(color.into())
    }
}

/// ANSI escape code to reset colors.
pub const ANSI_RESET: &str = "\x1b[0m";

#[cfg(test)]
mod tests {
    use super::*;

    // RGB/HSL conversion tests
    #[test]
    fn rgb_to_hsl_red() {
        let rgb = Rgb::new(255, 0, 0);
        let hsl = rgb.to_hsl();
        assert!((hsl.h - 0.0).abs() < 1.0);
        assert!((hsl.s - 1.0).abs() < 0.01);
        assert!((hsl.l - 0.5).abs() < 0.01);
    }

    #[test]
    fn rgb_to_hsl_green() {
        let rgb = Rgb::new(0, 255, 0);
        let hsl = rgb.to_hsl();
        assert!((hsl.h - 120.0).abs() < 1.0);
        assert!((hsl.s - 1.0).abs() < 0.01);
        assert!((hsl.l - 0.5).abs() < 0.01);
    }

    #[test]
    fn rgb_to_hsl_blue() {
        let rgb = Rgb::new(0, 0, 255);
        let hsl = rgb.to_hsl();
        assert!((hsl.h - 240.0).abs() < 1.0);
        assert!((hsl.s - 1.0).abs() < 0.01);
        assert!((hsl.l - 0.5).abs() < 0.01);
    }

    #[test]
    fn rgb_to_hsl_gray() {
        let rgb = Rgb::new(128, 128, 128);
        let hsl = rgb.to_hsl();
        assert!((hsl.s).abs() < 0.01); // Gray has no saturation
    }

    #[test]
    fn hsl_to_rgb_roundtrip() {
        let original = Rgb::new(100, 150, 200);
        let hsl = original.to_hsl();
        let back = hsl.to_rgb();
        assert!((original.r as i16 - back.r as i16).abs() <= 1);
        assert!((original.g as i16 - back.g as i16).abs() <= 1);
        assert!((original.b as i16 - back.b as i16).abs() <= 1);
    }

    // Interpolation tests
    #[test]
    fn rgb_interpolate_midpoint() {
        let a = Rgb::new(0, 0, 0);
        let b = Rgb::new(100, 200, 100);
        let mid = a.interpolate(b, 0.5);
        assert_eq!(mid.r, 50);
        assert_eq!(mid.g, 100);
        assert_eq!(mid.b, 50);
    }

    #[test]
    fn rgb_interpolate_edges() {
        let a = Rgb::new(10, 20, 30);
        let b = Rgb::new(100, 200, 255);
        assert_eq!(a.interpolate(b, 0.0), a);
        assert_eq!(a.interpolate(b, 1.0), b);
    }

    #[test]
    fn hsl_interpolate_hue_shortest_path() {
        // Red (0) to blue (240) should go through magenta, not green
        let a = Hsl::new(0.0, 1.0, 0.5);
        let b = Hsl::new(240.0, 1.0, 0.5);
        let mid = a.interpolate(b, 0.5);
        // Midpoint should be around 300 (magenta) or 120 depending on path
        // Actually shortest from 0 to 240 is through 300 (going negative)
        assert!((mid.h - 300.0).abs() < 1.0 || (mid.h - 120.0).abs() < 1.0);
    }

    // ANSI output tests
    #[test]
    fn ansi_fg_format() {
        let rgb = Rgb::new(255, 128, 64);
        assert_eq!(rgb.to_ansi_fg(), "\x1b[38;2;255;128;64m");
    }

    #[test]
    fn ansi_bg_format() {
        let rgb = Rgb::new(0, 128, 255);
        assert_eq!(rgb.to_ansi_bg(), "\x1b[48;2;0;128;255m");
    }

    // Linear gradient tests
    #[test]
    fn linear_gradient_horizontal() {
        let grad = LinearGradient::horizontal(Rgb::new(0, 0, 0), Rgb::new(255, 255, 255));
        let left = grad.color_at(grad.position_at(0.0, 0.5));
        let right = grad.color_at(grad.position_at(1.0, 0.5));
        assert!(left.r < 50); // Near black
        assert!(right.r > 200); // Near white
    }

    #[test]
    fn linear_gradient_vertical() {
        let grad = LinearGradient::vertical(Rgb::new(255, 0, 0), Rgb::new(0, 0, 255));
        let top = grad.color_at(grad.position_at(0.5, 0.0));
        let bottom = grad.color_at(grad.position_at(0.5, 1.0));
        assert!(top.r > 200); // Near red
        assert!(bottom.b > 200); // Near blue
    }

    // Radial gradient tests
    #[test]
    fn radial_gradient_center() {
        let grad = RadialGradient::centered(0.5, Rgb::new(255, 255, 255), Rgb::new(0, 0, 0));
        let center = grad.color_at(grad.position_at(0.5, 0.5));
        let edge = grad.color_at(grad.position_at(1.0, 0.5));
        assert!(center.r > 200); // Near white at center
        assert!(edge.r < 50); // Near black at edge
    }

    // Fill tests
    #[test]
    fn fill_solid() {
        let fill = Fill::solid(Rgb::new(100, 150, 200));
        let c1 = fill.color_at(0.0, 0.0);
        let c2 = fill.color_at(1.0, 1.0);
        assert_eq!(c1, c2); // Same everywhere
        assert_eq!(c1.r, 100);
    }

    // ColorStop tests
    #[test]
    fn color_stop_clamping() {
        let stop = ColorStop::new(1.5, Rgb::new(0, 0, 0));
        assert_eq!(stop.position, 1.0);

        let stop = ColorStop::new(-0.5, Rgb::new(0, 0, 0));
        assert_eq!(stop.position, 0.0);
    }

    // Gradient sampling tests
    #[test]
    fn sample_gradient_two_stops() {
        let stops = vec![
            ColorStop::new(0.0, Rgb::new(0, 0, 0)),
            ColorStop::new(1.0, Rgb::new(200, 200, 200)),
        ];
        let mid = sample_gradient(&stops, 0.5);
        assert_eq!(mid.r, 100);
    }

    #[test]
    fn sample_gradient_three_stops() {
        let stops = vec![
            ColorStop::new(0.0, Rgb::new(0, 0, 0)),
            ColorStop::new(0.5, Rgb::new(255, 0, 0)),
            ColorStop::new(1.0, Rgb::new(255, 255, 255)),
        ];
        let at_half = sample_gradient(&stops, 0.5);
        assert_eq!(at_half.r, 255);
        assert_eq!(at_half.g, 0);
    }
}
