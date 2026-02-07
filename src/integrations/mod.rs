//! Framework integrations for artbox.
//!
//! This module contains optional integrations with other Rust crates.
//!
//! ## Available Integrations
//!
//! - **`ratatui`** (requires `ratatui` feature): Provides `ArtBox`
//!   and `SpriteBox` widgets for rendering text and sprites.

#[cfg(feature = "ratatui")]
pub mod ratatui;
