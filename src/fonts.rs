//! Embedded FIGlet fonts and font management utilities.
//!
//! This module provides access to 27 embedded FIGlet fonts that are compiled
//! directly into the library. Fonts can be accessed individually or as
//! pre-configured stacks for automatic size fallback.
//!
//! # Font Access
//!
//! ```rust
//! use artbox::fonts;
//!
//! // Get a single font by name
//! let slant = fonts::font("slant").unwrap();
//!
//! // Build a custom font stack
//! let stack = fonts::stack(&["big", "standard", "small"]);
//!
//! // Use a themed font family
//! let cyber = fonts::family("cyber").unwrap();
//! ```
//!
//! # Available Fonts
//!
//! Use [`names()`] to get a list of all available font names.
//! Use [`family_names()`] to get a list of available font families.
//!
//! # Default Stack
//!
//! The [`default()`] function returns a stack of `big`, `standard`, `small`, `mini`
//! fonts, which provides good coverage across different display sizes.

use crate::Font;

const EMBEDDED: &[(&str, &[u8])] = &[
    ("banner", include_bytes!("../assets/fonts/banner.flf")),
    ("banner3", include_bytes!("../assets/fonts/banner3.flf")),
    ("banner4", include_bytes!("../assets/fonts/banner4.flf")),
    ("big", include_bytes!("../assets/fonts/big.flf")),
    (
        "cyberlarge",
        include_bytes!("../assets/fonts/cyberlarge.flf"),
    ),
    (
        "cybermedium",
        include_bytes!("../assets/fonts/cybermedium.flf"),
    ),
    (
        "cybersmall",
        include_bytes!("../assets/fonts/cybersmall.flf"),
    ),
    (
        "isometric1",
        include_bytes!("../assets/fonts/isometric1.flf"),
    ),
    ("keyboard", include_bytes!("../assets/fonts/keyboard.flf")),
    ("mini", include_bytes!("../assets/fonts/mini.flf")),
    ("poison", include_bytes!("../assets/fonts/poison.flf")),
    ("script", include_bytes!("../assets/fonts/script.flf")),
    ("shadow", include_bytes!("../assets/fonts/shadow.flf")),
    ("slant", include_bytes!("../assets/fonts/slant.flf")),
    ("small", include_bytes!("../assets/fonts/small.flf")),
    (
        "small_isometric1",
        include_bytes!("../assets/fonts/small_isometric1.flf"),
    ),
    (
        "small_keyboard",
        include_bytes!("../assets/fonts/small_keyboard.flf"),
    ),
    (
        "small_poison",
        include_bytes!("../assets/fonts/small_poison.flf"),
    ),
    (
        "small_script",
        include_bytes!("../assets/fonts/small_script.flf"),
    ),
    (
        "small_shadow",
        include_bytes!("../assets/fonts/small_shadow.flf"),
    ),
    (
        "small_slant",
        include_bytes!("../assets/fonts/small_slant.flf"),
    ),
    (
        "small_tengwar",
        include_bytes!("../assets/fonts/small_tengwar.flf"),
    ),
    ("smpoison", include_bytes!("../assets/fonts/smpoison.flf")),
    ("smscript", include_bytes!("../assets/fonts/smscript.flf")),
    ("smtengwar", include_bytes!("../assets/fonts/smtengwar.flf")),
    ("standard", include_bytes!("../assets/fonts/standard.flf")),
    ("tengwar", include_bytes!("../assets/fonts/tengwar.flf")),
];

const DEFAULT_SET_NAMES: &[&str] = &["big", "standard", "small", "mini"];
const NAMED_SETS: &[(&str, &[&str])] = &[
    ("banner", &["banner", "banner3", "banner4"]),
    ("cyber", &["cyberlarge", "cybermedium", "cybersmall"]),
    ("isometric1", &["isometric1", "small_isometric1"]),
    ("keyboard", &["keyboard", "small_keyboard"]),
    ("poison", &["poison", "small_poison", "smpoison"]),
    ("script", &["script", "small_script", "smscript"]),
    ("shadow", &["shadow", "small_shadow"]),
    ("slant", &["slant", "small_slant"]),
    ("tengwar", &["tengwar", "small_tengwar", "smtengwar"]),
];

/// Returns a list of all available embedded font names.
///
/// Font names are case-insensitive when used with [`font()`].
pub fn names() -> Vec<&'static str> {
    EMBEDDED.iter().map(|(name, _)| *name).collect()
}

/// Loads an embedded font by name.
///
/// Font name matching is case-insensitive.
///
/// # Examples
///
/// ```rust
/// let slant = artbox::fonts::font("slant").unwrap();
/// let also_slant = artbox::fonts::font("SLANT").unwrap(); // case-insensitive
/// ```
pub fn font(name: &str) -> Option<Font> {
    let contents = EMBEDDED
        .iter()
        .find(|(embedded_name, _)| embedded_name.eq_ignore_ascii_case(name))
        .map(|(_, contents)| *contents)?;

    Font::from_bytes_latin1(contents).ok()
}

/// Creates a font stack from a list of font names.
///
/// Invalid font names are silently skipped. The resulting stack can be
/// passed to [`Renderer::new()`](crate::Renderer::new).
///
/// # Examples
///
/// ```rust
/// use artbox::{Renderer, fonts};
///
/// let renderer = Renderer::new(fonts::stack(&["big", "small", "mini"]));
/// ```
pub fn stack(names: &[&str]) -> Vec<Font> {
    let mut fonts = Vec::new();
    for name in names {
        if let Some(font) = font(name) {
            fonts.push(font);
        }
    }

    fonts
}

/// Returns the default font stack.
///
/// The default stack includes `big`, `standard`, `small`, and `mini` fonts,
/// providing good coverage for most display sizes.
pub fn default() -> Vec<Font> {
    stack(DEFAULT_SET_NAMES)
}

/// Returns the font names in the default stack.
pub fn default_names() -> &'static [&'static str] {
    DEFAULT_SET_NAMES
}

/// Returns a list of available font family names.
///
/// Families are themed collections of fonts at different sizes.
/// Available families: `banner`, `cyber`, `isometric1`, `keyboard`,
/// `poison`, `script`, `shadow`, `slant`, `tengwar`.
pub fn family_names() -> Vec<&'static str> {
    NAMED_SETS.iter().map(|(name, _)| *name).collect()
}

/// Loads a font family by name.
///
/// Font families are themed collections of fonts at different sizes,
/// suitable for use as a font stack.
///
/// # Examples
///
/// ```rust
/// use artbox::{Renderer, fonts};
///
/// // Use the "cyber" family for a tech aesthetic
/// let renderer = Renderer::new(fonts::family("cyber").unwrap());
/// ```
pub fn family(name: &str) -> Option<Vec<Font>> {
    let names = NAMED_SETS
        .iter()
        .find(|(set_name, _)| set_name.eq_ignore_ascii_case(name))
        .map(|(_, names)| *names)?;
    Some(stack(names))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_returns_all_embedded() {
        let all_names = names();
        assert_eq!(all_names.len(), EMBEDDED.len());
        assert!(all_names.contains(&"standard"));
        assert!(all_names.contains(&"big"));
        assert!(all_names.contains(&"small"));
        assert!(all_names.contains(&"mini"));
    }

    #[test]
    fn font_loads_by_name() {
        assert!(font("standard").is_some());
        assert!(font("big").is_some());
        assert!(font("slant").is_some());
    }

    #[test]
    fn font_case_insensitive() {
        assert!(font("STANDARD").is_some());
        assert!(font("Standard").is_some());
        assert!(font("sTaNdArD").is_some());
    }

    #[test]
    fn font_invalid_name_returns_none() {
        assert!(font("nonexistent_font").is_none());
        assert!(font("").is_none());
    }

    #[test]
    fn stack_builds_font_list() {
        let fonts = stack(&["big", "standard", "small"]);
        assert_eq!(fonts.len(), 3);
    }

    #[test]
    fn stack_skips_invalid_names() {
        let fonts = stack(&["big", "invalid_font", "small"]);
        assert_eq!(fonts.len(), 2);
    }

    #[test]
    fn stack_empty_input() {
        let fonts = stack(&[]);
        assert!(fonts.is_empty());
    }

    #[test]
    fn default_returns_expected_fonts() {
        let fonts = default();
        assert_eq!(fonts.len(), DEFAULT_SET_NAMES.len());
    }

    #[test]
    fn default_names_matches_default() {
        let names = default_names();
        assert_eq!(names, DEFAULT_SET_NAMES);
        assert_eq!(names, &["big", "standard", "small", "mini"]);
    }

    #[test]
    fn family_names_returns_all_families() {
        let fam_names = family_names();
        assert_eq!(fam_names.len(), NAMED_SETS.len());
        assert!(fam_names.contains(&"cyber"));
        assert!(fam_names.contains(&"banner"));
        assert!(fam_names.contains(&"slant"));
    }

    #[test]
    fn family_loads_by_name() {
        let cyber = family("cyber");
        assert!(cyber.is_some());
        let fonts = cyber.unwrap();
        assert!(!fonts.is_empty());
    }

    #[test]
    fn family_case_insensitive() {
        assert!(family("CYBER").is_some());
        assert!(family("Cyber").is_some());
    }

    #[test]
    fn family_invalid_returns_none() {
        assert!(family("nonexistent_family").is_none());
    }

    #[test]
    fn all_embedded_fonts_load_successfully() {
        for name in names() {
            let result = font(name);
            assert!(result.is_some(), "Font '{}' failed to load", name);
        }
    }

    #[test]
    fn all_families_load_successfully() {
        for name in family_names() {
            let result = family(name);
            assert!(result.is_some(), "Family '{}' failed to load", name);
            assert!(
                !result.unwrap().is_empty(),
                "Family '{}' has no fonts",
                name
            );
        }
    }
}
