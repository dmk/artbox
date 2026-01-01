# artbox

Render figlet/ASCII text into a fixed rectangle.

## Usage

```rust
use artbox::{render, Alignment};

fn main() -> Result<(), artbox::RenderError> {
    let rendered = render("Hello", 40, 10)?;
    println!("{}", rendered.text);
    Ok(())
}
```

## Renderer reuse

```rust
use artbox::{Alignment, Renderer, fonts};

let fonts = fonts::default();
let renderer = Renderer::new(fonts)
    .with_plain_fallback()
    .with_alignment(Alignment::Center)
    .with_letter_spacing(-1);

let mut out = String::new();
let metrics = renderer.render_into("Hello", 40, 10, &mut out)?;
```

## Fonts

- `Font::from_file`, `Font::from_content`, and `Font::from_bytes_latin1` load `.flf` fonts.
- `fonts::default()` returns the built-in size stack (`big`, `standard`, `small`, `mini`).
- `fonts::family("slant")` returns a named family stack (e.g., `slant`, `script`).
- `fonts::stack(&["big", "small"])` builds a custom stack.

## ratatui integration

Enable the `ratatui` feature and use the widget wrapper:

```rust
use artbox::integrations::ratatui::ArtBox;

let widget = ArtBox::new(&renderer, "Hello");
```

## Features

- `ratatui`: enables the widget integration.
