# Changelog

## [0.1.0] - 2025-01-10

Initial release.

### Added

- `Renderer` for rendering FIGlet text into bounded rectangles
- Automatic font fallback when text doesn't fit in bounds
- `Alignment` options: `TopLeft`, `TopCenter`, `TopRight`, `CenterLeft`, `Center`, `CenterRight`, `BottomLeft`, `BottomCenter`, `BottomRight`
- Configurable letter spacing (including negative values for overlap)
- Plain text fallback option via `Renderer::with_plain_fallback`
- `render_into` for buffer reuse in hot render loops

#### Fonts

- Embedded font families with automatic size fallback:
  - `banner` - large block letters
  - `blocky` - pixel-style using `█▀▄` characters
  - `script` - cursive style
  - `slant` - italic style
- Default font stack: `big` → `standard` → `small` → `mini`
- `fonts::font`, `fonts::stack`, `fonts::family` for font access
- Support for loading custom fonts via `Font::from_file`, `Font::from_content`, `Font::from_bytes_latin1`

#### Colors and Gradients

- `Fill` types: `Solid`, `Linear`, `Radial`
- `LinearGradient` with configurable angle and color stops
- `RadialGradient` with configurable center and color stops
- HSL color interpolation for smooth gradients
- `render_styled` and `StyledRendered` for colored output
- ANSI escape code output via `to_ansi_string`

#### Integrations

- `ratatui` feature flag for TUI widget integration
- `ArtBox` widget implementing ratatui's `Widget` trait

[0.1.0]: https://github.com/dmk/artbox/releases/tag/v0.1.0
