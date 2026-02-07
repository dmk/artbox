# artbox

Render FIGlet text into a bounded rectangle with colors and gradients.

```
cargo add artbox
```

## Quick Start

```rust
use artbox::render;

let result = render("Hello", 40, 8)?;
println!("{}", result.to_plain_string());
```

Output:
```
 _   _      _ _
| | | | ___| | | ___
| |_| |/ _ \ | |/ _ \
|  _  |  __/ | | (_) |
|_| |_|\___|_|_|\___/
```

## Gradients

```rust
use artbox::{Renderer, Fill, LinearGradient, ColorStop, Color};

let renderer = Renderer::default()
    .with_fill(Fill::Linear(LinearGradient::new(
        45.0,
        vec![
            ColorStop::new(0.0, Color::rgb(255, 0, 128)),
            ColorStop::new(1.0, Color::rgb(0, 128, 255)),
        ],
    )));

let styled = renderer.render_styled("Hi", 20, 6)?;
print!("{}", styled.to_ansi_string());
```

Supports solid colors, linear gradients (any angle), and radial gradients.

## Font Families

Built-in font families with size fallback:

```rust
use artbox::{Renderer, fonts};

// Blocky pixel style (█▀▄ characters)
let renderer = Renderer::new(fonts::family("blocky").unwrap());

// Available families: banner, blocky, script, slant
// Default stack: big -> standard -> small -> mini
```

Custom stacks:

```rust
let renderer = Renderer::new(fonts::stack(&["slant", "small_slant"]));
```

Load external fonts:

```rust
let font = Font::from_file("path/to/font.flf")?;
```

## Alignment

```rust
use artbox::{Renderer, Alignment};

let renderer = Renderer::default()
    .with_alignment(Alignment::Center)  // or TopLeft, BottomRight, etc.
    .with_letter_spacing(-1);           // negative = overlap
```

## Buffer Reuse

For hot paths, reuse the output buffer:

```rust
let mut buffer = String::new();
let metrics = renderer.render_into("Text", 40, 10, &mut buffer)?;
```

## ratatui Widget

Enable the `ratatui` feature:

```toml
artbox = { version = "0.1", features = ["ratatui"] }
```

```rust
use artbox::integrations::ratatui::ArtBox;

let widget = ArtBox::new(&renderer, "Hello");
frame.render_widget(widget, area);
```

## Docs

Hosted docs: https://dmk.github.io/artbox

Local docs:

```bash
cd docs
npm install
npm run dev
```

## CLI Example

```bash
cargo run --example gradient -- "Hello" 60 10 --gradient diagonal --from 255,0,128 --to 0,128,255
```
