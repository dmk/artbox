# Agent Notes

artbox is a Rust crate for rendering figlet/ASCII text into a bounded rectangle, with optional ratatui integration.

## Repository Structure

- `src/lib.rs`: Core types (`Renderer`, `Font`, `Rendered`, `RenderMetrics`, `RenderError`) and rendering logic.
- `src/integrations/ratatui.rs`: `Widget` integration behind the `ratatui` feature flag.
- `Cargo.toml`: Feature flags (`ratatui`) and optional deps.

## Core Rendering Flow

```
Renderer::render(text, width, height)
  -> try fonts in order
  -> measure with unicode-width
  -> align into bounds
  -> return Rendered or NoFit
```

**Key behaviors:**
- Scaling is done only by choosing from the provided font stack order.
- `fonts::default` embeds a multi-size set (`big`, `standard`, `small`, `mini`) for scale-down.
- Use `Font::from_file`, `Font::from_content`, or `Font::from_bytes_latin1` to load `.flf` fonts.
- Plain text fallback is opt-in via `Renderer::with_plain_fallback`.
- Alignment can be set via `Renderer::with_alignment` using `Alignment` (e.g., `TopLeft`, `Center`, `BottomRight`).
- Letter spacing is configurable with `Renderer::with_letter_spacing` and supports negative values (overlap).
- Use `Renderer::render_into` to reuse output buffers in hot render loops.
- Embedded fonts are exposed via `artbox::fonts` (`fonts::stack` and `fonts::family`) for custom font stacks.

## After Meaningful Changes

Run the full verification suite before committing:

```bash
make verify
```

This runs: fmt-check, check, clippy, tests, and doc build.
