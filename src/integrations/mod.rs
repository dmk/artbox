//! Framework integrations for artbox.
//!
//! This module contains optional integrations with other Rust crates.
//!
//! ## Available Integrations
//!
//! - **`ratatui`** (requires `ratatui` feature): Provides [`ArtBox`](ratatui::ArtBox)
//!   and [`SpriteBox`](ratatui::SpriteBox) widgets for rendering text and sprites.

#[cfg(feature = "ratatui")]
pub mod ratatui;
