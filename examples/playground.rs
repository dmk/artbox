// artbox playground (tui-dispatch)
//
// Run:
//   cargo run --example playground --features images -- --image <path>[,<path>...]

#[cfg(not(feature = "images"))]
fn main() {
    eprintln!("Playground requires the `images` feature.");
    eprintln!("Try: cargo run --example playground --features images -- --image <path>");
}

#[cfg(feature = "images")]
fn main() -> std::io::Result<()> {
    playground::run()
}

#[cfg(feature = "images")]
mod playground {
    use std::collections::HashMap;
    use std::io;
    use std::path::PathBuf;

    use artbox::images::ascii::{
        self as ascii_img, render_image, render_image_at_size, AsciiMode, AsciiOptions,
        AsciiRendered,
    };
    use artbox::images::{
        detect_terminal_image_support, render_image_path, TerminalImageConfig, TerminalImageMode,
        TerminalImageSupport,
    };
    use artbox::{
        fonts, Alignment, Color, ColorStop, Fill, LinearGradient, RadialGradient, Rgb, Sprite,
        SpriteLayer, SpriteSelection, SpriteSize, SpriteVariant,
    };
    use clap::Parser;
    use crossterm::event::{self, Event, KeyCode};
    use crossterm::execute;
    use crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    };
    use image::{image_dimensions, DynamicImage};
    use ratatui::layout::{Alignment as TextAlignment, Constraint, Layout, Rect};
    use ratatui::style::{Color as TuiColor, Modifier, Style};
    use ratatui::text::{Line, Span, Text};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};
    use ratatui::{backend::CrosstermBackend, buffer::Buffer, Terminal};
    use resvg::usvg;
    use tui_dispatch::prelude::*;

    #[derive(Parser)]
    #[command(name = "playground", about = "artbox playground (tui-dispatch)")]
    struct Args {
        /// Image path(s) to use as sprite layers (comma-separated).
        #[arg(long, value_delimiter = ',')]
        image: Vec<PathBuf>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Tab {
        Fonts,
        Sprites,
    }

    impl Tab {
        fn index(self) -> usize {
            match self {
                Tab::Fonts => 0,
                Tab::Sprites => 1,
            }
        }

        fn next(self) -> Self {
            match self {
                Tab::Fonts => Tab::Sprites,
                Tab::Sprites => Tab::Fonts,
            }
        }

        fn prev(self) -> Self {
            match self {
                Tab::Fonts => Tab::Sprites,
                Tab::Sprites => Tab::Fonts,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SpriteSource {
        Demo,
        Image,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum OutputMode {
        Ascii,
        Auto,
        Kitty,
        Iterm2,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FillMode {
        None,
        Solid,
        Linear,
        Radial,
    }

    impl FillMode {
        fn label(self) -> &'static str {
            match self {
                FillMode::None => "none",
                FillMode::Solid => "solid",
                FillMode::Linear => "linear",
                FillMode::Radial => "radial",
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum LayerMode {
        All,
        BaseOnly,
        OverlayOnly,
    }

    impl LayerMode {
        fn label(self) -> &'static str {
            match self {
                LayerMode::All => "all",
                LayerMode::BaseOnly => "base",
                LayerMode::OverlayOnly => "overlay",
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SpriteSizeOverride {
        Auto,
        Small,
        Medium,
        Large,
    }

    impl SpriteSizeOverride {
        fn label(self) -> &'static str {
            match self {
                SpriteSizeOverride::Auto => "auto",
                SpriteSizeOverride::Small => "small",
                SpriteSizeOverride::Medium => "medium",
                SpriteSizeOverride::Large => "large",
            }
        }

        fn selection(self) -> SpriteSelection<'static> {
            match self {
                SpriteSizeOverride::Auto => SpriteSelection::Auto,
                SpriteSizeOverride::Small => SpriteSelection::Size(SpriteSize::Small),
                SpriteSizeOverride::Medium => SpriteSelection::Size(SpriteSize::Medium),
                SpriteSizeOverride::Large => SpriteSelection::Size(SpriteSize::Large),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FontMenuItem {
        Font,
        Alignment,
        Spacing,
        Fill,
        Palette,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SpriteMenuItem {
        Source,
        Output,
        Width,
        Height,
        HScale,
        VScale,
        AsciiMode,
        Invert,
        Color,
        Overlay,
        Size,
        Palette,
        Layers,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum ImageColorMode {
        Image,
        Solid,
        Gradient,
        None,
    }

    impl ImageColorMode {
        fn label(self) -> &'static str {
            match self {
                ImageColorMode::Image => "image",
                ImageColorMode::Solid => "solid",
                ImageColorMode::Gradient => "gradient",
                ImageColorMode::None => "none",
            }
        }
    }

    #[derive(Clone, Copy, Debug, Action)]
    enum Action {
        NextTab,
        PrevTab,
        ToggleHelp,
        MenuUp,
        MenuDown,
        MenuLeft,
        MenuRight,
        MenuToggle,
        Quit,
    }

    struct AppState {
        tab: Tab,
        font_menu_index: usize,
        sprite_menu_index: usize,
        font_index: usize,
        alignment_index: usize,
        spacing: i16,
        fill_mode: FillMode,
        font_palette: usize,
        sprite_source: SpriteSource,
        output_mode: OutputMode,
        ascii_mode: AsciiMode,
        ascii_invert: bool,
        color_enabled: bool,
        image_color_mode: ImageColorMode,
        image_width: u32,
        image_height: u32,
        image_h_scale: f32,
        image_v_scale: f32,
        overlay: bool,
        sprite_size: SpriteSizeOverride,
        sprite_palette: usize,
        layer_mode: LayerMode,
        show_help: bool,
    }

    impl Default for AppState {
        fn default() -> Self {
            Self {
                tab: Tab::Fonts,
                font_menu_index: 0,
                sprite_menu_index: 0,
                font_index: 0,
                alignment_index: 4,
                spacing: 0,
                fill_mode: FillMode::Linear,
                font_palette: 0,
                sprite_source: SpriteSource::Demo,
                output_mode: OutputMode::Auto,
                ascii_mode: AsciiMode::Full,
                ascii_invert: false,
                color_enabled: true,
                image_color_mode: ImageColorMode::Image,
                image_width: 0,
                image_height: 0,
                image_h_scale: 1.0,
                image_v_scale: 1.0,
                overlay: false,
                sprite_size: SpriteSizeOverride::Auto,
                sprite_palette: 0,
                layer_mode: LayerMode::All,
                show_help: false,
            }
        }
    }

    struct AppConfig {
        font_choices: Vec<FontChoice>,
        alignments: Vec<(Alignment, &'static str)>,
        image_paths: Vec<PathBuf>,
        image_support: TerminalImageSupport,
    }

    #[derive(Clone, Copy)]
    enum FontChoice {
        Default,
        Family(&'static str),
    }

    impl FontChoice {
        fn label(self) -> &'static str {
            match self {
                FontChoice::Default => "default",
                FontChoice::Family(name) => name,
            }
        }
    }

    struct RawImageRequest {
        area: Rect,
        mode: TerminalImageMode,
        width: Option<u16>,
        height: Option<u16>,
    }

    #[derive(Default)]
    struct ImageCache {
        base: HashMap<ImageCacheKey, AsciiRendered>,
        overlay: HashMap<OverlayCacheKey, AsciiRendered>,
        dimensions: HashMap<PathBuf, (u32, u32)>,
        svg_meta: HashMap<PathBuf, SvgViewBox>,
        multi: HashMap<MultiLayerKey, MultiLayerRender>,
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct ImageCacheKey {
        path: PathBuf,
        width: u32,
        mode: AsciiMode,
        invert: bool,
        color: bool,
        h_scale_tenths: u16,
        v_scale_tenths: u16,
        target_w: u32,
        target_h: u32,
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct OverlayCacheKey {
        path: PathBuf,
        width: u32,
        h_scale_tenths: u16,
        v_scale_tenths: u16,
        target_w: u32,
        target_h: u32,
    }

    #[derive(Default)]
    struct RawImageCache {
        rendered: HashMap<RawImageCacheKey, artbox::images::TerminalImage>,
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct RawImageCacheKey {
        path: PathBuf,
        protocol: artbox::images::ImageProtocol,
        width: Option<u16>,
        height: Option<u16>,
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct MultiLayerKey {
        paths: Vec<PathBuf>,
        width: u32,
        mode: AsciiMode,
        invert: bool,
        h_scale_tenths: u16,
        v_scale_tenths: u16,
        color_mode: ImageColorMode,
        target_w: u32,
        target_h: u32,
        overlay: bool,
    }

    struct MultiLayerRender {
        rendered: AsciiRendered,
        layer_map: Option<Vec<Option<usize>>>,
        overlay: Option<AsciiRendered>,
    }

    #[derive(Clone, Copy, Debug)]
    struct SvgViewBox {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    }

    #[derive(Clone, Copy, Debug)]
    struct LayerTarget {
        canvas_w: u32,
        canvas_h: u32,
        base_view: SvgViewBox,
        scale_x: f32,
        scale_y: f32,
    }

    #[derive(Default)]
    struct RawImageState {
        active: bool,
        area: Rect,
        protocol: Option<artbox::images::ImageProtocol>,
        width: Option<u16>,
        height: Option<u16>,
    }

    #[derive(Clone, Copy)]
    struct Palette {
        name: &'static str,
        primary: Color,
        secondary: Color,
        accent: Color,
        accent_alt: Color,
    }

    const PALETTES: [Palette; 4] = [
        Palette {
            name: "Sunset",
            primary: Color::rgb(255, 110, 72),
            secondary: Color::rgb(255, 185, 120),
            accent: Color::rgb(255, 64, 120),
            accent_alt: Color::rgb(120, 80, 220),
        },
        Palette {
            name: "Ocean",
            primary: Color::rgb(0, 140, 200),
            secondary: Color::rgb(0, 190, 180),
            accent: Color::rgb(0, 90, 170),
            accent_alt: Color::rgb(0, 220, 255),
        },
        Palette {
            name: "Forest",
            primary: Color::rgb(44, 140, 72),
            secondary: Color::rgb(110, 190, 110),
            accent: Color::rgb(30, 90, 48),
            accent_alt: Color::rgb(180, 220, 120),
        },
        Palette {
            name: "Mono",
            primary: Color::rgb(220, 220, 220),
            secondary: Color::rgb(170, 170, 170),
            accent: Color::rgb(130, 130, 130),
            accent_alt: Color::rgb(90, 90, 90),
        },
    ];

    pub fn run() -> io::Result<()> {
        let args = Args::parse();

        let mut font_choices = Vec::new();
        font_choices.push(FontChoice::Default);
        for name in fonts::family_names() {
            font_choices.push(FontChoice::Family(name));
        }

        let alignments = vec![
            (Alignment::TopLeft, "top-left"),
            (Alignment::Top, "top"),
            (Alignment::TopRight, "top-right"),
            (Alignment::Left, "left"),
            (Alignment::Center, "center"),
            (Alignment::Right, "right"),
            (Alignment::BottomLeft, "bottom-left"),
            (Alignment::Bottom, "bottom"),
            (Alignment::BottomRight, "bottom-right"),
        ];

        let config = AppConfig {
            font_choices,
            alignments,
            image_paths: args.image,
            image_support: detect_terminal_image_support(),
        };

        let mut initial_state = AppState::default();
        if !config.image_paths.is_empty() {
            initial_state.sprite_source = SpriteSource::Image;
        }

        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        let mut store = Store::new(initial_state, reducer);
        let mut raw_request: Option<RawImageRequest> = None;
        let mut image_cache = ImageCache::default();
        let mut raw_state = RawImageState::default();
        let mut raw_cache = RawImageCache::default();
        let mut pending_clear = false;

        loop {
            if pending_clear {
                clear_raw_image(&mut terminal, &raw_state)?;
                pending_clear = false;
            }

            terminal.draw(|frame| {
                let area = frame.area();
                let layout = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(area);

                render_tabs(frame, layout[0], store.state().tab);
                raw_request = match store.state().tab {
                    Tab::Fonts => {
                        render_fonts_tab(frame, layout[1], store.state(), &config);
                        None
                    }
                    Tab::Sprites => render_sprites_tab(
                        frame,
                        layout[1],
                        store.state(),
                        &config,
                        &mut image_cache,
                    ),
                };
                render_footer(frame, layout[2]);

                if store.state().show_help {
                    render_help_overlay(frame, layout[1], store.state());
                }
            })?;

            if let Some(request) = raw_request.take() {
                let expected_protocol = protocol_for_mode(request.mode);
                if raw_state.active
                    && (raw_state.area != request.area
                        || raw_state.protocol != expected_protocol
                        || raw_state.width != request.width
                        || raw_state.height != request.height)
                {
                    clear_raw_image(&mut terminal, &raw_state)?;
                }

                let protocol = render_raw_image(&mut terminal, &config, &request, &mut raw_cache)?;
                raw_state.active = protocol.is_some();
                raw_state.area = request.area;
                raw_state.protocol = protocol;
                raw_state.width = request.width;
                raw_state.height = request.height;
            } else if raw_state.active {
                pending_clear = true;
                raw_state.active = false;
            }

            if let Event::Key(key) = event::read()? {
                let action = map_key_to_action(store.state(), key.code);
                if let Some(action) = action {
                    if !store.dispatch(action) {
                        break;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn reducer(state: &mut AppState, action: Action) -> bool {
        match action {
            Action::NextTab => {
                state.tab = state.tab.next();
                true
            }
            Action::PrevTab => {
                state.tab = state.tab.prev();
                true
            }
            Action::ToggleHelp => {
                state.show_help = !state.show_help;
                true
            }
            Action::MenuUp => {
                match state.tab {
                    Tab::Fonts => {
                        state.font_menu_index = state.font_menu_index.saturating_sub(1);
                    }
                    Tab::Sprites => {
                        state.sprite_menu_index = state.sprite_menu_index.saturating_sub(1);
                    }
                }
                true
            }
            Action::MenuDown => {
                match state.tab {
                    Tab::Fonts => {
                        let len = font_menu_items().len();
                        state.font_menu_index = (state.font_menu_index + 1).min(len - 1);
                    }
                    Tab::Sprites => {
                        let len = sprite_menu_items(state).len();
                        state.sprite_menu_index = (state.sprite_menu_index + 1).min(len - 1);
                    }
                }
                true
            }
            Action::MenuLeft => {
                apply_menu_change(state, -1);
                true
            }
            Action::MenuRight => {
                apply_menu_change(state, 1);
                true
            }
            Action::MenuToggle => {
                apply_menu_toggle(state);
                true
            }
            Action::Quit => false,
        }
    }

    fn map_key_to_action(state: &AppState, code: KeyCode) -> Option<Action> {
        if state.show_help {
            return match code {
                KeyCode::Esc | KeyCode::Char('?') => Some(Action::ToggleHelp),
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            };
        }

        match code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Esc => Some(Action::Quit),
            KeyCode::Tab => Some(Action::NextTab),
            KeyCode::BackTab => Some(Action::PrevTab),
            KeyCode::Up => Some(Action::MenuUp),
            KeyCode::Down => Some(Action::MenuDown),
            KeyCode::Left => Some(Action::MenuLeft),
            KeyCode::Right => Some(Action::MenuRight),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::MenuToggle),
            KeyCode::Char('?') => Some(Action::ToggleHelp),
            _ => None,
        }
    }

    fn font_menu_items() -> &'static [FontMenuItem] {
        const ITEMS: [FontMenuItem; 5] = [
            FontMenuItem::Font,
            FontMenuItem::Alignment,
            FontMenuItem::Spacing,
            FontMenuItem::Fill,
            FontMenuItem::Palette,
        ];
        &ITEMS
    }

    fn sprite_menu_items(state: &AppState) -> Vec<SpriteMenuItem> {
        let mut items = vec![SpriteMenuItem::Source];
        match state.sprite_source {
            SpriteSource::Demo => {
                items.push(SpriteMenuItem::Color);
                items.push(SpriteMenuItem::Size);
                items.push(SpriteMenuItem::Palette);
                items.push(SpriteMenuItem::Layers);
            }
            SpriteSource::Image => {
                items.push(SpriteMenuItem::Output);
                items.push(SpriteMenuItem::Width);
                items.push(SpriteMenuItem::Height);
                items.push(SpriteMenuItem::HScale);
                items.push(SpriteMenuItem::VScale);
                items.push(SpriteMenuItem::AsciiMode);
                items.push(SpriteMenuItem::Invert);
                items.push(SpriteMenuItem::Color);
                items.push(SpriteMenuItem::Overlay);
                items.push(SpriteMenuItem::Palette);
            }
        }
        items
    }

    fn apply_menu_change(state: &mut AppState, delta: i32) {
        match state.tab {
            Tab::Fonts => {
                let item = font_menu_items()[state.font_menu_index];
                match item {
                    FontMenuItem::Font => {
                        state.font_index = step_index(state.font_index, delta);
                    }
                    FontMenuItem::Alignment => {
                        state.alignment_index = step_index(state.alignment_index, delta);
                    }
                    FontMenuItem::Spacing => {
                        let next = state.spacing + delta as i16;
                        state.spacing = next.clamp(-3, 3);
                    }
                    FontMenuItem::Fill => {
                        state.fill_mode = cycle_enum(state.fill_mode, &FILL_MODES, delta);
                    }
                    FontMenuItem::Palette => {
                        state.font_palette = step_index(state.font_palette, delta);
                    }
                }
            }
            Tab::Sprites => {
                let items = sprite_menu_items(state);
                let item = items
                    .get(state.sprite_menu_index)
                    .copied()
                    .unwrap_or(SpriteMenuItem::Source);
                match item {
                    SpriteMenuItem::Source => {
                        state.sprite_source =
                            cycle_enum(state.sprite_source, &SPRITE_SOURCES, delta);
                        clamp_sprite_menu_index(state);
                    }
                    SpriteMenuItem::Output => {
                        state.output_mode = cycle_enum(state.output_mode, &OUTPUT_MODES, delta);
                    }
                    SpriteMenuItem::Width => {
                        state.image_width = step_dimension(state.image_width, delta, 2);
                    }
                    SpriteMenuItem::Height => {
                        state.image_height = step_dimension(state.image_height, delta, 1);
                    }
                    SpriteMenuItem::HScale => {
                        state.image_h_scale = step_scale(state.image_h_scale, delta);
                    }
                    SpriteMenuItem::VScale => {
                        state.image_v_scale = step_scale(state.image_v_scale, delta);
                    }
                    SpriteMenuItem::AsciiMode => {
                        state.ascii_mode = cycle_enum(state.ascii_mode, &ASCII_MODES, delta);
                    }
                    SpriteMenuItem::Invert => {
                        state.ascii_invert = !state.ascii_invert;
                    }
                    SpriteMenuItem::Color => {
                        if state.sprite_source == SpriteSource::Image {
                            state.image_color_mode =
                                cycle_enum(state.image_color_mode, &IMAGE_COLOR_MODES, delta);
                        } else {
                            state.color_enabled = !state.color_enabled;
                        }
                    }
                    SpriteMenuItem::Overlay => {
                        state.overlay = !state.overlay;
                    }
                    SpriteMenuItem::Size => {
                        state.sprite_size = cycle_enum(state.sprite_size, &SPRITE_SIZES, delta);
                    }
                    SpriteMenuItem::Palette => {
                        state.sprite_palette = step_index(state.sprite_palette, delta);
                    }
                    SpriteMenuItem::Layers => {
                        state.layer_mode = cycle_enum(state.layer_mode, &LAYER_MODES, delta);
                    }
                }
            }
        }
    }

    fn apply_menu_toggle(state: &mut AppState) {
        apply_menu_change(state, 1);
    }

    fn clamp_sprite_menu_index(state: &mut AppState) {
        let len = sprite_menu_items(state).len();
        if state.sprite_menu_index >= len {
            state.sprite_menu_index = len.saturating_sub(1);
        }
    }

    fn step_index(index: usize, delta: i32) -> usize {
        if delta >= 0 {
            index.wrapping_add(delta as usize)
        } else {
            index.wrapping_sub((-delta) as usize)
        }
    }

    fn step_dimension(value: u32, delta: i32, step: u32) -> u32 {
        if delta >= 0 {
            if value == 0 {
                step
            } else {
                value.saturating_add(step)
            }
        } else if value <= step {
            0
        } else {
            value.saturating_sub(step)
        }
    }

    fn step_scale(value: f32, delta: i32) -> f32 {
        let mut next = value + (delta as f32) * 0.1;
        next = (next * 10.0).round() / 10.0;
        next.clamp(0.1, 3.0)
    }

    fn format_dimension(value: u32) -> String {
        if value == 0 {
            "auto".to_string()
        } else {
            value.to_string()
        }
    }

    fn format_scale(value: f32) -> String {
        format!("{value:.1}")
    }

    fn scale_key(value: f32) -> u16 {
        (value * 10.0).round() as u16
    }

    fn resolve_ascii_width(
        state: &AppState,
        area_width: u32,
        path: &std::path::Path,
        cache: &mut ImageCache,
    ) -> u32 {
        if state.image_width > 0 {
            return state.image_width.max(1);
        }

        if state.image_height > 0 {
            if let Some((w, h)) = cached_image_dimensions(path, cache) {
                if w > 0 {
                    let aspect = h as f32 / w as f32;
                    let denom = aspect * 0.5 * state.image_v_scale.max(0.1);
                    if denom > 0.0 {
                        let width = (state.image_height as f32 / denom).round();
                        return width.max(1.0) as u32;
                    }
                }
            }
        }

        area_width.max(1)
    }

    fn cached_image_dimensions(
        path: &std::path::Path,
        cache: &mut ImageCache,
    ) -> Option<(u32, u32)> {
        if let Some(dimensions) = cache.dimensions.get(path) {
            return Some(*dimensions);
        }

        let mut dimensions = image_dimensions(path).ok();
        if dimensions.is_none() && is_svg(path) {
            dimensions = cached_svg_meta(path, cache).map(|meta| {
                (
                    meta.width.round().max(1.0) as u32,
                    meta.height.round().max(1.0) as u32,
                )
            });
        }

        if let Some(dimensions) = dimensions {
            cache.dimensions.insert(path.to_path_buf(), dimensions);
        }

        dimensions
    }

    fn cached_svg_meta(path: &std::path::Path, cache: &mut ImageCache) -> Option<SvgViewBox> {
        if let Some(meta) = cache.svg_meta.get(path) {
            return Some(*meta);
        }

        let bytes = std::fs::read(path).ok()?;
        let meta = parse_svg_meta(&bytes)?;
        cache.svg_meta.insert(path.to_path_buf(), meta);
        Some(meta)
    }

    fn is_svg(path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("svg"))
            .unwrap_or(false)
    }

    fn parse_svg_meta(bytes: &[u8]) -> Option<SvgViewBox> {
        let svg = std::str::from_utf8(bytes).ok()?;
        if let Some(view_box) = parse_svg_viewbox(svg) {
            return Some(view_box);
        }

        let width = parse_svg_length(svg, "width")?;
        let height = parse_svg_length(svg, "height")?;
        Some(SvgViewBox {
            x: 0.0,
            y: 0.0,
            width,
            height,
        })
    }

    fn parse_svg_viewbox(svg: &str) -> Option<SvgViewBox> {
        let value = find_svg_attr(svg, "viewBox")?;
        let cleaned = value.replace(',', " ");
        let mut parts = cleaned
            .split_whitespace()
            .filter_map(|part| part.parse::<f32>().ok());
        let x = parts.next()?;
        let y = parts.next()?;
        let width = parts.next()?;
        let height = parts.next()?;
        if width <= 0.0 || height <= 0.0 {
            None
        } else {
            Some(SvgViewBox {
                x,
                y,
                width,
                height,
            })
        }
    }

    fn parse_svg_length(svg: &str, attr: &str) -> Option<f32> {
        let value = find_svg_attr(svg, attr)?;
        parse_number_prefix(&value)
    }

    fn find_svg_attr(svg: &str, attr: &str) -> Option<String> {
        let bytes = svg.as_bytes();
        let mut idx = 0;
        while let Some(pos) = svg[idx..].find(attr) {
            let start = idx + pos;
            let before = start.checked_sub(1).and_then(|i| bytes.get(i).copied());
            if before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') {
                idx = start + attr.len();
                continue;
            }

            let mut i = start + attr.len();
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'=' {
                idx = i;
                continue;
            }
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            let quote = bytes[i];
            if quote != b'"' && quote != b'\'' {
                idx = i + 1;
                continue;
            }
            i += 1;
            let value_start = i;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            return Some(svg[value_start..i].to_string());
        }
        None
    }

    fn parse_number_prefix(value: &str) -> Option<f32> {
        let mut end = 0;
        for (idx, ch) in value.char_indices() {
            if idx == 0 && (ch == '+' || ch == '-') {
                end = idx + ch.len_utf8();
                continue;
            }
            if ch.is_ascii_digit() || ch == '.' {
                end = idx + ch.len_utf8();
            } else {
                break;
            }
        }
        if end == 0 {
            None
        } else {
            value[..end].parse::<f32>().ok()
        }
    }

    fn build_layer_target(path: &std::path::Path, cache: &mut ImageCache) -> Option<LayerTarget> {
        let base_view = if is_svg(path) {
            cached_svg_meta(path, cache)?
        } else {
            let (width, height) = cached_image_dimensions(path, cache)?;
            SvgViewBox {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            }
        };

        if base_view.width <= 0.0 || base_view.height <= 0.0 {
            return None;
        }

        let canvas_w = base_view.width.round().max(1.0) as u32;
        let canvas_h = base_view.height.round().max(1.0) as u32;
        Some(build_layer_target_for_canvas(base_view, canvas_w, canvas_h))
    }

    fn build_layer_target_for_canvas(
        base_view: SvgViewBox,
        canvas_w: u32,
        canvas_h: u32,
    ) -> LayerTarget {
        let scale_x = canvas_w as f32 / base_view.width.max(1.0);
        let scale_y = canvas_h as f32 / base_view.height.max(1.0);
        LayerTarget {
            canvas_w,
            canvas_h,
            base_view,
            scale_x,
            scale_y,
        }
    }

    fn target_pixel_dimensions(base_view: SvgViewBox, options: &AsciiOptions) -> (u32, u32) {
        let aspect_ratio = if base_view.width > 0.0 {
            base_view.height / base_view.width
        } else {
            1.0
        };
        let base_width = options.width as f32;
        let mut target_w = (base_width * options.h_scale).round().max(1.0) as u32;
        let mut target_h = (aspect_ratio * base_width * 0.5 * options.v_scale)
            .round()
            .max(1.0) as u32;
        if matches!(options.mode, AsciiMode::Block) {
            target_w = target_w.saturating_mul(2).max(1);
            target_h = target_h.saturating_mul(2).max(1);
        }
        (target_w, target_h)
    }

    fn cycle_enum<T: Copy + PartialEq>(current: T, options: &[T], delta: i32) -> T {
        let len = options.len() as i32;
        if len == 0 {
            return current;
        }
        let idx = options
            .iter()
            .position(|&value| value == current)
            .unwrap_or(0) as i32;
        let next = (idx + delta).rem_euclid(len) as usize;
        options[next]
    }

    const OUTPUT_MODES: [OutputMode; 4] = [
        OutputMode::Ascii,
        OutputMode::Auto,
        OutputMode::Kitty,
        OutputMode::Iterm2,
    ];

    const ASCII_MODES: [AsciiMode; 4] = [
        AsciiMode::Full,
        AsciiMode::Block,
        AsciiMode::Shade,
        AsciiMode::Ascii,
    ];

    const FILL_MODES: [FillMode; 4] = [
        FillMode::None,
        FillMode::Solid,
        FillMode::Linear,
        FillMode::Radial,
    ];

    const SPRITE_SOURCES: [SpriteSource; 2] = [SpriteSource::Demo, SpriteSource::Image];

    const SPRITE_SIZES: [SpriteSizeOverride; 4] = [
        SpriteSizeOverride::Auto,
        SpriteSizeOverride::Small,
        SpriteSizeOverride::Medium,
        SpriteSizeOverride::Large,
    ];

    const LAYER_MODES: [LayerMode; 3] =
        [LayerMode::All, LayerMode::BaseOnly, LayerMode::OverlayOnly];

    const IMAGE_COLOR_MODES: [ImageColorMode; 4] = [
        ImageColorMode::Image,
        ImageColorMode::Solid,
        ImageColorMode::Gradient,
        ImageColorMode::None,
    ];

    fn render_tabs(frame: &mut ratatui::Frame, area: Rect, tab: Tab) {
        let titles = ["Fonts", "Sprites"]
            .iter()
            .map(|title| Line::from(Span::raw(*title)))
            .collect::<Vec<_>>();
        let tabs = Tabs::new(titles)
            .select(tab.index())
            .block(Block::default().borders(Borders::ALL).title(" Playground "))
            .highlight_style(
                Style::default()
                    .fg(TuiColor::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(tabs, area);
    }

    fn render_footer(frame: &mut ratatui::Frame, area: Rect) {
        let text = "Tab/Shift-Tab tabs | ↑/↓ select | ←/→ change | Enter toggle | ? help | q quit";
        let help = Paragraph::new(text)
            .alignment(TextAlignment::Center)
            .style(Style::default().fg(TuiColor::DarkGray));
        frame.render_widget(help, area);
    }

    fn menu_line(text: &str, selected: bool) -> Line<'static> {
        if selected {
            let style = Style::default()
                .fg(TuiColor::Yellow)
                .add_modifier(Modifier::BOLD);
            Line::from(vec![
                Span::styled("> ", style),
                Span::styled(text.to_string(), style),
            ])
        } else {
            Line::from(format!("  {text}"))
        }
    }

    fn dim_line(text: String) -> Line<'static> {
        Line::from(Span::styled(text, Style::default().fg(TuiColor::DarkGray)))
    }

    fn render_fonts_tab(
        frame: &mut ratatui::Frame,
        area: Rect,
        state: &AppState,
        config: &AppConfig,
    ) {
        let sections = Layout::horizontal([Constraint::Length(34), Constraint::Min(0)]).split(area);
        let controls = render_font_controls(state, config);
        frame.render_widget(controls, sections[0]);

        let preview_block = Block::default().title(" Preview ").borders(Borders::ALL);
        let preview_area = preview_block.inner(sections[1]);
        frame.render_widget(preview_block, sections[1]);

        let renderer = build_renderer(state, config);
        let widget = artbox::integrations::ratatui::ArtBox::new(&renderer, "ARTBOX");
        frame.render_widget(widget, preview_area);
    }

    fn render_font_controls(state: &AppState, config: &AppConfig) -> Paragraph<'static> {
        let font_choice = config
            .font_choices
            .get(state.font_index % config.font_choices.len())
            .copied()
            .unwrap_or(FontChoice::Default);
        let alignment = config
            .alignments
            .get(state.alignment_index % config.alignments.len())
            .map(|(_, label)| *label)
            .unwrap_or("center");
        let palette = palette_for(state.font_palette);
        let mut lines = Vec::new();
        for (idx, item) in font_menu_items().iter().enumerate() {
            let selected = idx == state.font_menu_index;
            let text = match item {
                FontMenuItem::Font => format!("Font: {}", font_choice.label()),
                FontMenuItem::Alignment => format!("Align: {}", alignment),
                FontMenuItem::Spacing => format!("Spacing: {}", state.spacing),
                FontMenuItem::Fill => format!("Fill: {}", state.fill_mode.label()),
                FontMenuItem::Palette => format!("Palette: {}", palette.name),
            };
            lines.push(menu_line(&text, selected));
        }

        Paragraph::new(Text::from(lines))
            .block(Block::default().title(" Settings ").borders(Borders::ALL))
    }

    fn build_renderer(state: &AppState, config: &AppConfig) -> artbox::Renderer {
        let font_choice = config
            .font_choices
            .get(state.font_index % config.font_choices.len())
            .copied()
            .unwrap_or(FontChoice::Default);
        let fonts = match font_choice {
            FontChoice::Default => fonts::default(),
            FontChoice::Family(name) => fonts::family(name).unwrap_or_else(fonts::default),
        };

        let alignment = config
            .alignments
            .get(state.alignment_index % config.alignments.len())
            .map(|(alignment, _)| *alignment)
            .unwrap_or(Alignment::Center);

        let palette = palette_for(state.font_palette);
        let fill = fill_from_mode(state.fill_mode, palette);

        let mut renderer = artbox::Renderer::new(fonts)
            .with_plain_fallback()
            .with_alignment(alignment)
            .with_letter_spacing(state.spacing);

        if let Some(fill) = fill {
            renderer = renderer.with_fill(fill);
        }

        renderer
    }

    fn render_sprites_tab(
        frame: &mut ratatui::Frame,
        area: Rect,
        state: &AppState,
        config: &AppConfig,
        cache: &mut ImageCache,
    ) -> Option<RawImageRequest> {
        let sections = Layout::horizontal([Constraint::Length(36), Constraint::Min(0)]).split(area);
        let controls = render_sprite_controls(state, config);
        frame.render_widget(controls, sections[0]);

        let preview_block = Block::default().title(" Preview ").borders(Borders::ALL);
        let preview_area = preview_block.inner(sections[1]);
        frame.render_widget(preview_block, sections[1]);

        let palette = palette_for(state.sprite_palette);

        match state.sprite_source {
            SpriteSource::Demo => {
                let sprite = demo_sprite(palette, state.layer_mode, state.color_enabled);
                let selection = state.sprite_size.selection();
                let widget = artbox::integrations::ratatui::SpriteBox::new(&sprite)
                    .with_selection(selection);
                frame.render_widget(widget, preview_area);
                None
            }
            SpriteSource::Image => {
                render_image_source(frame, preview_area, state, config, palette, cache)
            }
        }
    }

    fn render_sprite_controls(state: &AppState, config: &AppConfig) -> Paragraph<'static> {
        let palette = palette_for(state.sprite_palette);
        let source_label = match state.sprite_source {
            SpriteSource::Demo => "demo".to_string(),
            SpriteSource::Image => {
                if config.image_paths.is_empty() {
                    "image (missing)".to_string()
                } else if config.image_paths.len() == 1 {
                    "image".to_string()
                } else {
                    format!("image ({})", config.image_paths.len())
                }
            }
        };

        let output_status = resolve_output(state.output_mode, config.image_support);
        let multi_layer = config.image_paths.len() > 1;
        let output_line = match state.sprite_source {
            SpriteSource::Demo => "ascii (sprite)".to_string(),
            SpriteSource::Image => match output_status {
                OutputStatus::Ascii => {
                    if multi_layer {
                        "ascii (layers)".to_string()
                    } else {
                        "ascii".to_string()
                    }
                }
                OutputStatus::Raw { label, .. } => {
                    if multi_layer {
                        format!("{label} -> ascii (layers)")
                    } else {
                        label.to_string()
                    }
                }
                OutputStatus::Fallback { label } => {
                    if multi_layer {
                        format!("{label} -> ascii (layers)")
                    } else {
                        format!("{label} -> ascii")
                    }
                }
            },
        };

        let support_label = match config.image_support {
            TerminalImageSupport::Kitty => "kitty",
            TerminalImageSupport::Iterm2 => "iterm2",
            TerminalImageSupport::Unsupported => "none",
            _ => "unknown",
        };

        let mut lines = Vec::new();
        for (idx, item) in sprite_menu_items(state).iter().enumerate() {
            let selected = idx == state.sprite_menu_index;
            let text = match item {
                SpriteMenuItem::Source => format!("Source: {}", source_label),
                SpriteMenuItem::Output => format!("Output: {}", output_line),
                SpriteMenuItem::Width => format!("Width: {}", format_dimension(state.image_width)),
                SpriteMenuItem::Height => {
                    format!("Height: {}", format_dimension(state.image_height))
                }
                SpriteMenuItem::HScale => {
                    format!("H-Scale: {}", format_scale(state.image_h_scale))
                }
                SpriteMenuItem::VScale => {
                    format!("V-Scale: {}", format_scale(state.image_v_scale))
                }
                SpriteMenuItem::AsciiMode => {
                    format!("Mode: {}", ascii_mode_label(state.ascii_mode))
                }
                SpriteMenuItem::Invert => format!("Invert: {}", on_off(state.ascii_invert)),
                SpriteMenuItem::Color => {
                    if state.sprite_source == SpriteSource::Image {
                        format!("Color: {}", state.image_color_mode.label())
                    } else {
                        format!("Color: {}", on_off(state.color_enabled))
                    }
                }
                SpriteMenuItem::Overlay => format!("Overlay: {}", on_off(state.overlay)),
                SpriteMenuItem::Size => format!("Size: {}", state.sprite_size.label()),
                SpriteMenuItem::Palette => format!("Palette: {}", palette.name),
                SpriteMenuItem::Layers => format!("Layers: {}", state.layer_mode.label()),
            };
            lines.push(menu_line(&text, selected));
        }

        if state.sprite_source == SpriteSource::Image {
            lines.push(Line::from(""));
            lines.push(dim_line(format!("Support: {}", support_label)));
        }

        Paragraph::new(Text::from(lines))
            .block(Block::default().title(" Settings ").borders(Borders::ALL))
    }

    fn render_image_source(
        frame: &mut ratatui::Frame,
        area: Rect,
        state: &AppState,
        config: &AppConfig,
        palette: Palette,
        cache: &mut ImageCache,
    ) -> Option<RawImageRequest> {
        if config.image_paths.is_empty() {
            let note = Paragraph::new("Pass --image <path>[,<path>...] to preview.")
                .alignment(TextAlignment::Center);
            frame.render_widget(note, area);
            return None;
        }

        let paths = config.image_paths.as_slice();
        let multi_layer = paths.len() > 1;

        let output_status = resolve_output(state.output_mode, config.image_support);
        if multi_layer {
            render_ascii_preview(frame, area, state, palette, paths, cache);
            return None;
        }

        match output_status {
            OutputStatus::Raw { mode, label } => {
                let note = Paragraph::new(format!("Raw image output ({label})"))
                    .alignment(TextAlignment::Center)
                    .style(Style::default().fg(TuiColor::DarkGray));
                frame.render_widget(note, area);
                let width = if state.image_width > 0 {
                    Some(state.image_width.min(u16::MAX as u32) as u16)
                } else {
                    Some(area.width)
                };
                let height = if state.image_height > 0 {
                    Some(state.image_height.min(u16::MAX as u32) as u16)
                } else {
                    Some(area.height)
                };
                Some(RawImageRequest {
                    area,
                    mode,
                    width,
                    height,
                })
            }
            OutputStatus::Ascii | OutputStatus::Fallback { .. } => {
                render_ascii_preview(frame, area, state, palette, paths, cache);
                None
            }
        }
    }

    fn render_ascii_preview(
        frame: &mut ratatui::Frame,
        area: Rect,
        state: &AppState,
        palette: Palette,
        paths: &[PathBuf],
        cache: &mut ImageCache,
    ) {
        let Some(base_path) = paths.first() else {
            return;
        };

        if paths.len() > 1 {
            render_ascii_multilayer(frame, area, state, palette, paths, cache);
            return;
        }

        let width = resolve_ascii_width(state, area.width.max(1) as u32, base_path, cache);
        let layer_target = build_layer_target(base_path, cache);
        let (target_w, target_h) = layer_target
            .map(|target| (target.canvas_w, target.canvas_h))
            .unwrap_or((0, 0));

        let base_options = AsciiOptions {
            width,
            h_scale: state.image_h_scale,
            v_scale: state.image_v_scale,
            mode: state.ascii_mode,
            invert: state.ascii_invert,
            color: state.image_color_mode == ImageColorMode::Image,
            ..AsciiOptions::default()
        };

        for (idx, path) in paths.iter().enumerate() {
            let (use_image_color, fill_override, overlay_color) =
                layer_color_settings(state.image_color_mode, palette, idx);
            let key = ImageCacheKey {
                path: path.clone(),
                width,
                mode: base_options.mode,
                invert: base_options.invert,
                color: use_image_color,
                h_scale_tenths: scale_key(state.image_h_scale),
                v_scale_tenths: scale_key(state.image_v_scale),
                target_w,
                target_h,
            };

            let options = AsciiOptions {
                color: use_image_color,
                ..base_options.clone()
            };

            let rendered = if let Some(rendered) = cache.base.get(&key) {
                rendered
            } else {
                match render_ascii_layer(path, &options, layer_target, cache) {
                    Ok(rendered) => {
                        cache.base.insert(key.clone(), rendered);
                        cache.base.get(&key).expect("just inserted")
                    }
                    Err(err) => {
                        let note = Paragraph::new(format!("Failed to render: {err}"))
                            .alignment(TextAlignment::Center);
                        frame.render_widget(note, area);
                        return;
                    }
                }
            };

            {
                let buf = frame.buffer_mut();
                draw_ascii(rendered, area, buf, None, fill_override.as_ref(), idx > 0);
            }

            if state.overlay {
                let overlay_key = OverlayCacheKey {
                    path: path.clone(),
                    width,
                    h_scale_tenths: scale_key(state.image_h_scale),
                    v_scale_tenths: scale_key(state.image_v_scale),
                    target_w,
                    target_h,
                };
                let overlay = if let Some(overlay) = cache.overlay.get(&overlay_key) {
                    overlay
                } else {
                    let overlay_options = AsciiOptions {
                        width,
                        mode: AsciiMode::Block,
                        color: false,
                        threshold: 180,
                        invert: false,
                        h_scale: state.image_h_scale,
                        v_scale: state.image_v_scale,
                        ..AsciiOptions::default()
                    };
                    match render_ascii_layer(path, &overlay_options, layer_target, cache) {
                        Ok(overlay) => {
                            cache.overlay.insert(overlay_key.clone(), overlay);
                            cache.overlay.get(&overlay_key).expect("just inserted")
                        }
                        Err(_) => return,
                    }
                };

                let buf = frame.buffer_mut();
                draw_ascii(overlay, area, buf, overlay_color, None, true);
            }
        }
    }

    fn render_ascii_multilayer(
        frame: &mut ratatui::Frame,
        area: Rect,
        state: &AppState,
        palette: Palette,
        paths: &[PathBuf],
        cache: &mut ImageCache,
    ) {
        let Some(base_path) = paths.first() else {
            return;
        };

        let width = resolve_ascii_width(state, area.width.max(1) as u32, base_path, cache);
        let base_view = if is_svg(base_path) {
            cached_svg_meta(base_path, cache)
        } else {
            cached_image_dimensions(base_path, cache).map(|(w, h)| SvgViewBox {
                x: 0.0,
                y: 0.0,
                width: w as f32,
                height: h as f32,
            })
        };

        let Some(base_view) = base_view else {
            let note =
                Paragraph::new("Failed to read base layer size.").alignment(TextAlignment::Center);
            frame.render_widget(note, area);
            return;
        };

        let base_options = AsciiOptions {
            width,
            h_scale: state.image_h_scale,
            v_scale: state.image_v_scale,
            mode: state.ascii_mode,
            invert: state.ascii_invert,
            color: state.image_color_mode == ImageColorMode::Image,
            ..AsciiOptions::default()
        };

        let (target_w, target_h) = target_pixel_dimensions(base_view, &base_options);
        let layer_target = build_layer_target_for_canvas(base_view, target_w, target_h);

        let key = MultiLayerKey {
            paths: paths.to_vec(),
            width,
            mode: base_options.mode,
            invert: base_options.invert,
            h_scale_tenths: scale_key(state.image_h_scale),
            v_scale_tenths: scale_key(state.image_v_scale),
            color_mode: state.image_color_mode,
            target_w,
            target_h,
            overlay: state.overlay,
        };

        let cached = if let Some(cached) = cache.multi.get(&key) {
            cached
        } else {
            let mut layer_images = Vec::new();
            for path in paths {
                match rasterize_layer_to_canvas(path, layer_target, base_options.color, true, cache)
                {
                    Ok(image) => layer_images.push(image.to_rgba8()),
                    Err(err) => {
                        let note = Paragraph::new(format!("Failed to render: {err}"))
                            .alignment(TextAlignment::Center);
                        frame.render_widget(note, area);
                        return;
                    }
                }
            }

            let mut composite = image::RgbaImage::new(target_w, target_h);
            for layer in &layer_images {
                alpha_over(layer, &mut composite);
            }

            let composite_image = DynamicImage::ImageRgba8(composite.clone());
            let rendered =
                match render_image_at_size(composite_image, &base_options, target_w, target_h) {
                    Ok(rendered) => rendered,
                    Err(err) => {
                        let note = Paragraph::new(format!("Failed to render: {err}"))
                            .alignment(TextAlignment::Center);
                        frame.render_widget(note, area);
                        return;
                    }
                };

            let layer_map = if matches!(
                state.image_color_mode,
                ImageColorMode::Solid | ImageColorMode::Gradient
            ) {
                Some(build_layer_map(
                    &layer_images,
                    &rendered,
                    base_options.mode,
                    base_options.alpha_threshold,
                ))
            } else {
                None
            };

            let overlay = if state.overlay {
                let overlay_options = AsciiOptions {
                    width,
                    mode: AsciiMode::Block,
                    color: false,
                    threshold: 180,
                    invert: false,
                    h_scale: state.image_h_scale,
                    v_scale: state.image_v_scale,
                    ..AsciiOptions::default()
                };
                render_image_at_size(
                    DynamicImage::ImageRgba8(composite),
                    &overlay_options,
                    target_w,
                    target_h,
                )
                .ok()
            } else {
                None
            };

            cache.multi.insert(
                key.clone(),
                MultiLayerRender {
                    rendered,
                    layer_map,
                    overlay,
                },
            );
            cache.multi.get(&key).expect("just inserted")
        };

        let mut rendered = cached.rendered.clone();
        if let Some(layer_map) = &cached.layer_map {
            apply_layer_colors(&mut rendered, layer_map, palette, state.image_color_mode);
        }

        {
            let buf = frame.buffer_mut();
            draw_ascii(&rendered, area, buf, None, None, false);
        }

        if let Some(overlay) = &cached.overlay {
            let overlay_color = if state.image_color_mode != ImageColorMode::None {
                Some(palette.accent.to_rgb())
            } else {
                None
            };
            let buf = frame.buffer_mut();
            draw_ascii(overlay, area, buf, overlay_color, None, true);
        }
    }

    fn render_ascii_layer(
        path: &PathBuf,
        options: &AsciiOptions,
        target: Option<LayerTarget>,
        cache: &mut ImageCache,
    ) -> Result<AsciiRendered, String> {
        if let Some(target) = target {
            if target.canvas_w == 0 || target.canvas_h == 0 {
                return Err("invalid target size".to_string());
            }
            let image = rasterize_layer_to_canvas(path, target, options.color, false, cache)?;
            render_image(image, options).map_err(|err| err.to_string())
        } else {
            ascii_img::render_image_path(path, options).map_err(|err| err.to_string())
        }
    }

    fn rasterize_layer_to_canvas(
        path: &PathBuf,
        target: LayerTarget,
        use_black: bool,
        align_to_origin: bool,
        cache: &mut ImageCache,
    ) -> Result<DynamicImage, String> {
        if is_svg(path) {
            let bytes = std::fs::read(path).map_err(|err| err.to_string())?;
            let meta = cached_svg_meta(path, cache);
            if align_to_origin {
                if let Some(meta) = meta {
                    if meta.width > 0.0
                        && meta.height > 0.0
                        && target.base_view.width > 0.0
                        && target.base_view.height > 0.0
                    {
                        return render_svg_bytes_to_size_aligned(
                            &bytes,
                            target.base_view,
                            meta,
                            (target.canvas_w, target.canvas_h),
                            use_black,
                        );
                    }
                }
                let layer_image = render_svg_bytes_to_size(
                    &bytes,
                    (target.canvas_w, target.canvas_h),
                    use_black,
                )?;
                Ok(layer_image)
            } else {
                let meta = meta.ok_or_else(|| "invalid svg".to_string())?;
                let layer_w = (meta.width * target.scale_x).round().max(1.0) as u32;
                let layer_h = (meta.height * target.scale_y).round().max(1.0) as u32;
                let layer_image = render_svg_bytes_to_size(&bytes, (layer_w, layer_h), use_black)?;
                let mut canvas = image::RgbaImage::new(target.canvas_w, target.canvas_h);
                let offset_x = ((meta.x - target.base_view.x) * target.scale_x).round() as i32;
                let offset_y = ((meta.y - target.base_view.y) * target.scale_y).round() as i32;
                blit_layer(&layer_image.to_rgba8(), &mut canvas, offset_x, offset_y);
                Ok(DynamicImage::ImageRgba8(canvas))
            }
        } else {
            let image = image::open(path).map_err(|err| err.to_string())?;
            if image.width() == target.canvas_w && image.height() == target.canvas_h {
                Ok(image)
            } else {
                Ok(image.resize_exact(
                    target.canvas_w,
                    target.canvas_h,
                    image::imageops::FilterType::Triangle,
                ))
            }
        }
    }

    fn render_svg_bytes_to_size(
        bytes: &[u8],
        target: (u32, u32),
        use_black: bool,
    ) -> Result<DynamicImage, String> {
        let svg_str =
            std::str::from_utf8(bytes).map_err(|err| format!("invalid utf-8 svg: {err}"))?;
        let replacement = if use_black { "black" } else { "white" };
        let svg_str = svg_str.replace("currentColor", replacement);

        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();
        let tree =
            usvg::Tree::from_data(svg_str.as_bytes(), &opt).map_err(|err| err.to_string())?;

        let size = tree.size();
        if size.width() <= 0.0 || size.height() <= 0.0 {
            return Err("svg has invalid dimensions".to_string());
        }

        let (target_w, target_h) = target;
        let scale_x = target_w as f32 / size.width();
        let scale_y = target_h as f32 / size.height();

        let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)
            .ok_or_else(|| "failed to allocate pixmap for svg".to_string())?;

        let mut pixmap_mut = pixmap.as_mut();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(scale_x, scale_y),
            &mut pixmap_mut,
        );

        let image =
            image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
                .ok_or_else(|| "failed to build rgba buffer from svg".to_string())?;

        Ok(DynamicImage::ImageRgba8(image))
    }

    fn render_svg_bytes_to_size_aligned(
        bytes: &[u8],
        base_view: SvgViewBox,
        layer_view: SvgViewBox,
        target: (u32, u32),
        use_black: bool,
    ) -> Result<DynamicImage, String> {
        if base_view.width <= 0.0 || base_view.height <= 0.0 {
            return render_svg_bytes_to_size(bytes, target, use_black);
        }
        if layer_view.width <= 0.0 || layer_view.height <= 0.0 {
            return render_svg_bytes_to_size(bytes, target, use_black);
        }

        let svg_str =
            std::str::from_utf8(bytes).map_err(|err| format!("invalid utf-8 svg: {err}"))?;
        let replacement = if use_black { "black" } else { "white" };
        let svg_str = svg_str.replace("currentColor", replacement);

        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();
        let tree =
            usvg::Tree::from_data(svg_str.as_bytes(), &opt).map_err(|err| err.to_string())?;

        let (target_w, target_h) = target;
        if target_w == 0 || target_h == 0 {
            return Err("invalid target size".to_string());
        }

        let mut pixmap = resvg::tiny_skia::Pixmap::new(target_w, target_h)
            .ok_or_else(|| "failed to allocate pixmap for svg".to_string())?;

        let scale_x = base_view.width / layer_view.width;
        let scale_y = base_view.height / layer_view.height;
        let px_scale_x = target_w as f32 / base_view.width;
        let px_scale_y = target_h as f32 / base_view.height;

        let mut align = resvg::tiny_skia::Transform::from_scale(scale_x, scale_y);
        align = align.pre_concat(resvg::tiny_skia::Transform::from_translate(
            -layer_view.x,
            -layer_view.y,
        ));

        let pixel_scale = resvg::tiny_skia::Transform::from_scale(px_scale_x, px_scale_y);
        let transform = pixel_scale.pre_concat(align);

        let mut pixmap_mut = pixmap.as_mut();
        resvg::render(&tree, transform, &mut pixmap_mut);

        let image =
            image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
                .ok_or_else(|| "failed to build rgba buffer from svg".to_string())?;

        Ok(DynamicImage::ImageRgba8(image))
    }

    fn alpha_over(src: &image::RgbaImage, dst: &mut image::RgbaImage) {
        let (w, h) = dst.dimensions();
        for y in 0..h {
            for x in 0..w {
                let s = src.get_pixel(x, y);
                let s_a = s[3] as f32 / 255.0;
                if s_a <= 0.0 {
                    continue;
                }
                let d = dst.get_pixel(x, y);
                let d_a = d[3] as f32 / 255.0;
                let out_a = s_a + d_a * (1.0 - s_a);
                let mut out = [0u8; 4];
                if out_a > 0.0 {
                    for i in 0..3 {
                        let s_c = s[i] as f32 / 255.0;
                        let d_c = d[i] as f32 / 255.0;
                        let out_c = (s_c * s_a + d_c * d_a * (1.0 - s_a)) / out_a;
                        out[i] = (out_c * 255.0).round().clamp(0.0, 255.0) as u8;
                    }
                }
                out[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
                dst.put_pixel(x, y, image::Rgba(out));
            }
        }
    }

    fn build_layer_map(
        layers: &[image::RgbaImage],
        rendered: &AsciiRendered,
        mode: AsciiMode,
        alpha_threshold: u8,
    ) -> Vec<Option<usize>> {
        let width = rendered.width as usize;
        let height = rendered.height as usize;
        let step = if matches!(mode, AsciiMode::Block) {
            2
        } else {
            1
        };
        let mut map = vec![None; width * height];

        for y in 0..height {
            for x in 0..width {
                let px = x as u32 * step as u32;
                let py = y as u32 * step as u32;
                let mut layer_idx = None;
                for (idx, layer) in layers.iter().enumerate().rev() {
                    if block_has_alpha(layer, px, py, step as u32, alpha_threshold) {
                        layer_idx = Some(idx);
                        break;
                    }
                }
                map[y * width + x] = layer_idx;
            }
        }

        map
    }

    fn block_has_alpha(
        layer: &image::RgbaImage,
        px: u32,
        py: u32,
        step: u32,
        alpha_threshold: u8,
    ) -> bool {
        let max_x = (px + step).min(layer.width());
        let max_y = (py + step).min(layer.height());
        for y in py..max_y {
            for x in px..max_x {
                let pixel = layer.get_pixel(x, y);
                if pixel[3] >= alpha_threshold {
                    return true;
                }
            }
        }
        false
    }

    fn apply_layer_colors(
        rendered: &mut AsciiRendered,
        layer_map: &[Option<usize>],
        palette: Palette,
        mode: ImageColorMode,
    ) {
        let width = rendered.width as usize;
        let height = rendered.height as usize;
        if layer_map.len() != width * height {
            return;
        }

        let layer_count = layer_map
            .iter()
            .filter_map(|idx| *idx)
            .max()
            .map(|idx| idx + 1)
            .unwrap_or(0);

        let mut fills = Vec::with_capacity(layer_count);
        for idx in 0..layer_count {
            let (_, fill, _) = layer_color_settings(mode, palette, idx);
            fills.push(fill);
        }

        for (row_idx, row) in rendered.chars.iter_mut().enumerate() {
            for (col_idx, sc) in row.iter_mut().enumerate() {
                if sc.ch == ' ' {
                    continue;
                }
                let idx = row_idx * width + col_idx;
                if let Some(layer_idx) = layer_map.get(idx).and_then(|v| *v) {
                    if let Some(Some(fill)) = fills.get(layer_idx) {
                        let nx = if width > 1 {
                            col_idx as f32 / (width - 1) as f32
                        } else {
                            0.5
                        };
                        let ny = if height > 1 {
                            row_idx as f32 / (height - 1) as f32
                        } else {
                            0.5
                        };
                        sc.fg = Some(fill.color_at(nx, ny));
                    }
                }
            }
        }
    }

    fn blit_layer(
        layer: &image::RgbaImage,
        canvas: &mut image::RgbaImage,
        offset_x: i32,
        offset_y: i32,
    ) {
        let canvas_w = canvas.width() as i32;
        let canvas_h = canvas.height() as i32;
        for y in 0..layer.height() {
            let dst_y = y as i32 + offset_y;
            if dst_y < 0 || dst_y >= canvas_h {
                continue;
            }
            for x in 0..layer.width() {
                let dst_x = x as i32 + offset_x;
                if dst_x < 0 || dst_x >= canvas_w {
                    continue;
                }
                let pixel = *layer.get_pixel(x, y);
                if pixel[3] == 0 {
                    continue;
                }
                canvas.put_pixel(dst_x as u32, dst_y as u32, pixel);
            }
        }
    }

    fn draw_ascii(
        rendered: &AsciiRendered,
        area: Rect,
        buf: &mut Buffer,
        override_color: Option<Rgb>,
        fill_override: Option<&Fill>,
        skip_spaces: bool,
    ) {
        for (row_idx, row) in rendered.chars.iter().enumerate() {
            let y = area.y + row_idx as u16;
            if y >= area.y + area.height {
                break;
            }

            for (col_idx, sc) in row.iter().enumerate() {
                if skip_spaces && sc.ch == ' ' {
                    continue;
                }

                let x = area.x + col_idx as u16;
                if x >= area.x + area.width {
                    break;
                }

                let cell = &mut buf[(x, y)];
                cell.set_char(sc.ch);

                let fg = if let Some(fill) = fill_override {
                    let nx = if rendered.width > 1 {
                        col_idx as f32 / (rendered.width - 1) as f32
                    } else {
                        0.5
                    };
                    let ny = if rendered.height > 1 {
                        row_idx as f32 / (rendered.height - 1) as f32
                    } else {
                        0.5
                    };
                    Some(fill.color_at(nx, ny))
                } else {
                    override_color.or(sc.fg)
                };
                match fg {
                    Some(rgb) => {
                        cell.set_fg(to_tui_color(rgb));
                    }
                    None => {
                        cell.set_fg(TuiColor::Reset);
                    }
                }
                cell.set_bg(TuiColor::Reset);
            }
        }
    }

    fn render_raw_image(
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        config: &AppConfig,
        request: &RawImageRequest,
        cache: &mut RawImageCache,
    ) -> io::Result<Option<artbox::images::ImageProtocol>> {
        let Some(path) = config.image_paths.first() else {
            return Ok(None);
        };

        let Some(protocol) = protocol_for_mode(request.mode) else {
            return Ok(None);
        };

        let key = RawImageCacheKey {
            path: path.clone(),
            protocol,
            width: request.width,
            height: request.height,
        };

        let cfg = TerminalImageConfig::default()
            .with_mode(request.mode)
            .with_size(request.width, request.height);

        let rendered = if let Some(rendered) = cache.rendered.get(&key) {
            rendered
        } else {
            match render_image_path(path, cfg) {
                Ok(rendered) => {
                    cache.rendered.insert(key.clone(), rendered);
                    cache.rendered.get(&key).expect("just inserted")
                }
                Err(_) => return Ok(None),
            }
        };

        execute!(
            terminal.backend_mut(),
            crossterm::cursor::MoveTo(request.area.x, request.area.y),
            crossterm::style::Print(rendered.as_str())
        )?;
        Ok(Some(rendered.protocol))
    }

    fn clear_raw_image(
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        state: &RawImageState,
    ) -> io::Result<()> {
        for row in 0..state.area.height {
            let y = state.area.y + row;
            execute!(
                terminal.backend_mut(),
                crossterm::cursor::MoveTo(state.area.x, y),
                crossterm::style::Print(" ".repeat(state.area.width as usize))
            )?;
        }

        if matches!(state.protocol, Some(artbox::images::ImageProtocol::Kitty)) {
            execute!(
                terminal.backend_mut(),
                crossterm::style::Print("\u{1b}_Ga=d\u{1b}\\")
            )?;
        }

        Ok(())
    }

    fn protocol_for_mode(mode: TerminalImageMode) -> Option<artbox::images::ImageProtocol> {
        match mode {
            TerminalImageMode::Kitty => Some(artbox::images::ImageProtocol::Kitty),
            TerminalImageMode::Iterm2 => Some(artbox::images::ImageProtocol::Iterm2),
            TerminalImageMode::Auto | TerminalImageMode::Disabled | _ => None,
        }
    }

    fn resolve_output(mode: OutputMode, support: TerminalImageSupport) -> OutputStatus {
        match mode {
            OutputMode::Ascii => OutputStatus::Ascii,
            OutputMode::Auto => match support {
                TerminalImageSupport::Kitty => OutputStatus::Raw {
                    mode: TerminalImageMode::Kitty,
                    label: "auto (kitty)",
                },
                TerminalImageSupport::Iterm2 => OutputStatus::Raw {
                    mode: TerminalImageMode::Iterm2,
                    label: "auto (iterm2)",
                },
                TerminalImageSupport::Unsupported | _ => OutputStatus::Fallback {
                    label: "auto (unsupported)",
                },
            },
            OutputMode::Kitty => {
                let label = if support == TerminalImageSupport::Kitty {
                    "kitty"
                } else {
                    "kitty (forced)"
                };
                OutputStatus::Raw {
                    mode: TerminalImageMode::Kitty,
                    label,
                }
            }
            OutputMode::Iterm2 => {
                let label = if support == TerminalImageSupport::Iterm2 {
                    "iterm2"
                } else {
                    "iterm2 (forced)"
                };
                OutputStatus::Raw {
                    mode: TerminalImageMode::Iterm2,
                    label,
                }
            }
        }
    }

    enum OutputStatus {
        Ascii,
        Raw {
            mode: TerminalImageMode,
            label: &'static str,
        },
        Fallback {
            label: &'static str,
        },
    }

    fn to_tui_color(rgb: Rgb) -> TuiColor {
        TuiColor::Rgb(rgb.r, rgb.g, rgb.b)
    }

    fn ascii_mode_label(mode: AsciiMode) -> &'static str {
        match mode {
            AsciiMode::Full => "full",
            AsciiMode::Block => "block",
            AsciiMode::Shade => "shade",
            AsciiMode::Ascii => "ascii",
            _ => "unknown",
        }
    }

    fn on_off(value: bool) -> &'static str {
        if value {
            "on"
        } else {
            "off"
        }
    }

    fn palette_for(index: usize) -> Palette {
        PALETTES[index % PALETTES.len()]
    }

    fn palette_color(palette: Palette, index: usize) -> Color {
        let colors = [
            palette.primary,
            palette.accent,
            palette.secondary,
            palette.accent_alt,
        ];
        colors[index % colors.len()]
    }

    fn palette_triplet(palette: Palette, index: usize) -> (Color, Color, Color) {
        let colors = [
            palette.primary,
            palette.accent,
            palette.secondary,
            palette.accent_alt,
        ];
        let len = colors.len();
        (
            colors[index % len],
            colors[(index + 1) % len],
            colors[(index + 2) % len],
        )
    }

    fn fill_from_mode(mode: FillMode, palette: Palette) -> Option<Fill> {
        match mode {
            FillMode::None => None,
            FillMode::Solid => Some(Fill::solid(palette.primary)),
            FillMode::Linear => Some(Fill::Linear(LinearGradient::new(
                25.0,
                vec![
                    ColorStop::new(0.0, palette.primary),
                    ColorStop::new(0.5, palette.accent),
                    ColorStop::new(1.0, palette.secondary),
                ],
            ))),
            FillMode::Radial => Some(Fill::Radial(RadialGradient::centered(
                0.85,
                palette.accent_alt,
                palette.secondary,
            ))),
        }
    }

    fn layer_color_settings(
        mode: ImageColorMode,
        palette: Palette,
        index: usize,
    ) -> (bool, Option<Fill>, Option<Rgb>) {
        match mode {
            ImageColorMode::Image => (true, None, Some(palette.accent.to_rgb())),
            ImageColorMode::Solid => {
                let color = palette_color(palette, index);
                (false, Some(Fill::solid(color)), Some(color.to_rgb()))
            }
            ImageColorMode::Gradient => {
                let (c0, c1, c2) = palette_triplet(palette, index);
                let fill = Fill::Linear(LinearGradient::new(
                    20.0,
                    vec![
                        ColorStop::new(0.0, c0),
                        ColorStop::new(0.5, c1),
                        ColorStop::new(1.0, c2),
                    ],
                ));
                (false, Some(fill), Some(c1.to_rgb()))
            }
            ImageColorMode::None => (false, None, None),
        }
    }

    fn demo_sprite(
        palette: Palette,
        layer_mode: LayerMode,
        color_enabled: bool,
    ) -> Sprite<'static> {
        const SMALL_BG: &str = " ***\n*   *\n*   *\n*   *\n ***";
        const SMALL_FG: &str = "  + \n +++\n+++++\n +++\n  + ";
        const MED_BG: &str =
            "  *****\n *     *\n*       *\n*       *\n*       *\n *     *\n  *****";
        const MED_FG: &str = "   +  \n  +++ \n +++++\n+++++++\n +++++\n  +++ \n   +  ";
        const LARGE_BG: &str = "   *******\n  *       *\n *         *\n*           *\n*           *\n*           *\n*           *\n*           *\n *         *\n  *       *\n   *******";
        const LARGE_FG: &str =
            "    +    \n   +++   \n  +++++  \n +++++++ \n+++++++++\n +++++++ \n  +++++  \n   +++   \n    +    ";

        let make_layers = |bg: &'static str, fg: &'static str| {
            let base = if color_enabled {
                SpriteLayer::colored(bg, palette.primary)
            } else {
                SpriteLayer::new(bg)
            };
            let overlay = if color_enabled {
                SpriteLayer::colored(fg, palette.accent)
            } else {
                SpriteLayer::new(fg)
            };
            match layer_mode {
                LayerMode::All => vec![base, overlay],
                LayerMode::BaseOnly => vec![base],
                LayerMode::OverlayOnly => vec![overlay],
            }
        };

        let small = SpriteVariant::new("small", make_layers(SMALL_BG, SMALL_FG));
        let medium = SpriteVariant::new("medium", make_layers(MED_BG, MED_FG));
        let large = SpriteVariant::new("large", make_layers(LARGE_BG, LARGE_FG));

        Sprite::new(vec![large, medium, small]).with_alignment(Alignment::Center)
    }

    fn render_help_overlay(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
        let mut lines = vec![
            Line::from("Global"),
            Line::from("  Tab/Shift-Tab  switch tabs"),
            Line::from("  ↑/↓            select setting"),
            Line::from("  ←/→ or Enter   change value"),
            Line::from("  ? / Esc        toggle help"),
            Line::from("  q              quit"),
            Line::from(""),
        ];

        match state.tab {
            Tab::Fonts => {
                lines.extend([
                    Line::from("Fonts"),
                    Line::from("  Font"),
                    Line::from("  Alignment"),
                    Line::from("  Spacing"),
                    Line::from("  Fill"),
                    Line::from("  Palette"),
                ]);
            }
            Tab::Sprites => {
                lines.push(Line::from("Sprites"));
                lines.push(Line::from("  Source"));
                match state.sprite_source {
                    SpriteSource::Demo => {
                        lines.extend([
                            Line::from("  Color"),
                            Line::from("  Size"),
                            Line::from("  Palette"),
                            Line::from("  Layers"),
                        ]);
                    }
                    SpriteSource::Image => {
                        lines.extend([
                            Line::from("  Output"),
                            Line::from("  Width"),
                            Line::from("  Height"),
                            Line::from("  H-Scale"),
                            Line::from("  V-Scale"),
                            Line::from("  Mode"),
                            Line::from("  Invert"),
                            Line::from("  Color"),
                            Line::from("  Overlay"),
                            Line::from("  Palette"),
                        ]);
                    }
                }
            }
        }

        let overlay_area = centered_rect(70, 70, area);
        frame.render_widget(Clear, overlay_area);
        let help = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .alignment(TextAlignment::Left);
        frame.render_widget(help, overlay_area);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

        let horizontal = Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

        horizontal[1]
    }
}
