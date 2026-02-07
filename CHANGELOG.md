# Changelog

## [0.2.0] - 2026-02-07

### Added

- Unified `Artbox` API for text, sprites, and images using `RenderTarget` + `Rendered`
- `sprites` module with layered variants, size selection, and per-layer fills
- `images` feature for image-to-ASCII plus Kitty/iTerm2 terminal image output
- `cli` feature and `artbox` binary for PNG/SVG to ASCII conversion
- `fonts::try_stack` helper for validating named embedded font stacks
- Hosted docs site and expanded usage examples

### Changed

- Public API was rewritten after `0.1.0` (breaking for `0.1.x` consumers)
- Feature model is now fully opt-in: `default = []`, with `ratatui`, `images`, and `cli` feature flags
- Font loading now returns typed `FontError` variants (`Io`, `InvalidUtf8`, `Parse`)
- Public enums are now `#[non_exhaustive]` for forward-compatible API evolution

### Fixed

- Sprite width measurement now respects Unicode display width
- Gradient stops are pre-sorted at construction for stable interpolation
- `ratatui` widget rendering avoids per-cell allocations in the draw path
- `--no-color` ASCII output now renders using terminal foreground color

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

[0.2.0]: https://github.com/dmk/artbox/releases/tag/v0.2.0
[0.1.0]: https://github.com/dmk/artbox/releases/tag/v0.1.0
