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

pub fn names() -> Vec<&'static str> {
    EMBEDDED.iter().map(|(name, _)| *name).collect()
}

pub fn font(name: &str) -> Option<Font> {
    let contents = EMBEDDED
        .iter()
        .find(|(embedded_name, _)| embedded_name.eq_ignore_ascii_case(name))
        .map(|(_, contents)| *contents)?;

    Font::from_bytes_latin1(contents).ok()
}

pub fn stack(names: &[&str]) -> Vec<Font> {
    let mut fonts = Vec::new();
    for name in names {
        if let Some(font) = font(name) {
            fonts.push(font);
        }
    }

    fonts
}

pub fn default() -> Vec<Font> {
    stack(DEFAULT_SET_NAMES)
}

pub fn default_names() -> &'static [&'static str] {
    DEFAULT_SET_NAMES
}

pub fn family_names() -> Vec<&'static str> {
    NAMED_SETS.iter().map(|(name, _)| *name).collect()
}

pub fn family(name: &str) -> Option<Vec<Font>> {
    let names = NAMED_SETS
        .iter()
        .find(|(set_name, _)| set_name.eq_ignore_ascii_case(name))
        .map(|(_, names)| *names)?;
    Some(stack(names))
}
