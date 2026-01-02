//! Framework integrations for artbox.
//!
//! This module contains optional integrations with other Rust crates.
//!
//! ## Available Integrations
//!
//! - **`ratatui`** (requires `ratatui` feature): Provides an [`ArtBox`](ratatui::ArtBox)
//!   widget for rendering ASCII art in terminal UIs.

#[cfg(feature = "ratatui")]
pub mod ratatui;
