use std::sync::Arc;

use figlet_rs::FIGfont;
use unicode_width::UnicodeWidthStr;

pub mod fonts;
pub mod integrations;

#[derive(Clone)]
pub struct Font {
    kind: FontKind,
}

#[derive(Clone)]
enum FontKind {
    Figlet(Arc<FIGfont>),
    Plain,
}

impl Font {
    pub fn figlet(font: FIGfont) -> Self {
        Self {
            kind: FontKind::Figlet(Arc::new(font)),
        }
    }

    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents = std::fs::read(path).map_err(|e| format!("{e:?}"))?;
        Self::from_bytes_latin1(&contents)
    }

    pub fn from_content(contents: &str) -> Result<Self, String> {
        parse_figlet_content(contents).map(Self::figlet)
    }

    pub fn from_bytes_latin1(bytes: &[u8]) -> Result<Self, String> {
        let contents = latin1_to_string(bytes);
        parse_figlet_content(&contents).map(Self::figlet)
    }

    pub fn standard() -> Option<Self> {
        FIGfont::standard().ok().map(Self::figlet)
    }

    pub fn plain() -> Self {
        Self {
            kind: FontKind::Plain,
        }
    }

    fn is_plain(&self) -> bool {
        matches!(self.kind, FontKind::Plain)
    }

    fn render_with_spacing(&self, content: &str, letter_spacing: i16) -> Option<String> {
        match &self.kind {
            FontKind::Figlet(font) => {
                if letter_spacing == 0 {
                    font.convert(content).map(|figure| figure.to_string())
                } else {
                    render_figlet_with_spacing(font, content, letter_spacing)
                }
            }
            FontKind::Plain => Some(apply_letter_spacing_plain(content, letter_spacing)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Clone)]
pub struct Renderer {
    fonts: Vec<Font>,
    alignment: Alignment,
    letter_spacing: i16,
}

impl Renderer {
    pub fn new(fonts: Vec<Font>) -> Self {
        Self {
            fonts,
            alignment: Alignment::TopLeft,
            letter_spacing: 0,
        }
    }

    pub fn with_plain_fallback(mut self) -> Self {
        if !self.fonts.iter().any(|font| font.is_plain()) {
            self.fonts.push(Font::plain());
        }
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_letter_spacing(mut self, letter_spacing: i16) -> Self {
        self.letter_spacing = letter_spacing;
        self
    }

    pub fn render(&self, text: &str, width: u16, height: u16) -> Result<Rendered, RenderError> {
        let mut buffer = String::new();
        let metrics = self.render_into(text, width, height, &mut buffer)?;
        Ok(Rendered {
            text: buffer,
            width: metrics.width,
            height: metrics.height,
            font_index: metrics.font_index,
        })
    }

    pub fn render_into(
        &self,
        text: &str,
        width: u16,
        height: u16,
        out: &mut String,
    ) -> Result<RenderMetrics, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::EmptyBounds);
        }

        if self.fonts.is_empty() {
            return Err(RenderError::EmptyFonts);
        }

        for (index, font) in self.fonts.iter().enumerate() {
            if let Some(rendered) = font.render_with_spacing(text, self.letter_spacing) {
                let (content_width, content_height) = measure_rendered(&rendered);
                if content_width <= width as usize && content_height <= height as usize {
                    align_rendered_into(&rendered, width, height, self.alignment, out);
                    return Ok(RenderMetrics {
                        width: content_width as u16,
                        height: content_height as u16,
                        font_index: index,
                    });
                }
            }
        }

        Err(RenderError::NoFit)
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new(fonts::default()).with_plain_fallback()
    }
}

pub fn render(text: &str, width: u16, height: u16) -> Result<Rendered, RenderError> {
    Renderer::default().render(text, width, height)
}

#[derive(Debug, Clone)]
pub struct Rendered {
    pub text: String,
    /// Dimensions of the rendered content before alignment padding.
    pub width: u16,
    pub height: u16,
    pub font_index: usize,
}

impl Rendered {
    pub fn metrics(&self) -> RenderMetrics {
        RenderMetrics {
            width: self.width,
            height: self.height,
            font_index: self.font_index,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderMetrics {
    pub width: u16,
    pub height: u16,
    pub font_index: usize,
}

#[derive(Debug, Clone)]
pub enum RenderError {
    EmptyBounds,
    EmptyFonts,
    NoFit,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::EmptyBounds => write!(f, "width and height must be non-zero"),
            RenderError::EmptyFonts => write!(f, "no fonts provided"),
            RenderError::NoFit => write!(f, "no font fits within the requested bounds"),
        }
    }
}

impl std::error::Error for RenderError {}

fn render_figlet_with_spacing(font: &FIGfont, content: &str, spacing: i16) -> Option<String> {
    if content.is_empty() {
        return None;
    }

    let mut characters = Vec::new();
    for ch in content.chars() {
        if let Some(character) = font.fonts.get(&(ch as u32)) {
            characters.push(character);
        }
    }

    if characters.is_empty() {
        return None;
    }

    let height = font.header_line.height as usize;
    let mut lines: Vec<Vec<char>> = vec![Vec::new(); height];
    let mut cursor: isize = 0;

    for character in characters {
        for (line_idx, line) in character.characters.iter().enumerate() {
            let row = &mut lines[line_idx];
            if cursor > 0 {
                let needed = cursor as usize;
                if row.len() < needed {
                    row.resize(needed, ' ');
                }
            }

            let mut pos = cursor.max(0) as usize;
            for ch in line.chars() {
                if pos >= row.len() {
                    row.push(ch);
                } else if ch != ' ' {
                    row[pos] = ch;
                }
                pos += 1;
            }
        }

        let glyph_width = character
            .characters
            .first()
            .map(|line| line.chars().count())
            .unwrap_or(0) as isize;
        cursor += glyph_width + spacing as isize;
        if cursor < 0 {
            cursor = 0;
        }
    }

    let mut out_lines = Vec::with_capacity(height);
    for mut row in lines {
        while row.last() == Some(&' ') {
            row.pop();
        }
        out_lines.push(row.into_iter().collect::<String>());
    }

    Some(out_lines.join("\n"))
}

fn apply_letter_spacing_plain(text: &str, spacing: i16) -> String {
    if spacing == 0 || text.is_empty() {
        return text.to_string();
    }

    let mut out_lines = Vec::new();

    for line in text.lines() {
        let mut row: Vec<char> = Vec::new();
        let mut cursor: isize = 0;
        for ch in line.chars() {
            if cursor > 0 {
                let needed = cursor as usize;
                if row.len() < needed {
                    row.resize(needed, ' ');
                }
            }
            let pos = cursor.max(0) as usize;
            if pos >= row.len() {
                row.push(ch);
            } else {
                row[pos] = ch;
            }
            cursor += 1 + spacing as isize;
            if cursor < 0 {
                cursor = 0;
            }
        }

        while row.last() == Some(&' ') {
            row.pop();
        }

        out_lines.push(row.into_iter().collect::<String>());
    }

    out_lines.join("\n")
}

fn latin1_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| *byte as char).collect()
}

fn parse_figlet_content(contents: &str) -> Result<FIGfont, String> {
    match FIGfont::from_content(contents) {
        Ok(font) => Ok(font),
        Err(err) => {
            let Some(sanitized) = sanitize_figlet_content(contents) else {
                return Err(err);
            };
            FIGfont::from_content(&sanitized)
        }
    }
}

fn sanitize_figlet_content(contents: &str) -> Option<String> {
    let lines: Vec<&str> = contents.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let mut header_parts = lines[0].split_whitespace();
    header_parts.next()?;
    let height: usize = header_parts.next()?.parse().ok()?;
    header_parts.next()?;
    header_parts.next()?;
    header_parts.next()?;
    let comment_lines: usize = header_parts.next()?.parse().ok()?;

    let offset = 1 + comment_lines + 102 * height;
    if lines.len() <= offset {
        return None;
    }

    let block_size = height + 1;
    let codetag_lines = lines.len().saturating_sub(offset);
    if codetag_lines == 0 || !codetag_lines.is_multiple_of(block_size) {
        return None;
    }

    let mut removed_any = false;
    let mut sanitized: Vec<&str> = Vec::with_capacity(lines.len());
    sanitized.extend_from_slice(&lines[..offset]);

    let blocks = codetag_lines / block_size;
    for i in 0..blocks {
        let start = offset + i * block_size;
        let tag_line = *lines.get(start)?;
        let code_token = tag_line.split_whitespace().next();
        let code = code_token.and_then(parse_codetag_code);
        if code.is_none() || code.is_some_and(|value| value < 0) {
            removed_any = true;
            continue;
        }

        sanitized.extend_from_slice(&lines[start..start + block_size]);
    }

    if !removed_any {
        return None;
    }

    let mut output = sanitized.join("\n");
    if contents.ends_with('\n') {
        output.push('\n');
    }
    Some(output)
}

fn parse_codetag_code(token: &str) -> Option<i32> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }

    let (sign, digits) = if let Some(stripped) = token.strip_prefix('-') {
        (-1, stripped)
    } else {
        (1, token)
    };

    let value = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        i32::from_str_radix(hex, 16).ok()?
    } else if digits.starts_with('0') && digits != "0" {
        i32::from_str_radix(digits, 8).ok()?
    } else {
        digits.parse::<i32>().ok()?
    };

    Some(sign * value)
}

fn measure_rendered(rendered: &str) -> (usize, usize) {
    let mut max_width = 0;
    let mut lines = 0;

    for line in rendered.trim_end_matches('\n').lines() {
        lines += 1;
        max_width = max_width.max(UnicodeWidthStr::width(line));
    }

    (max_width, lines)
}

fn align_rendered_into(
    rendered: &str,
    width: u16,
    height: u16,
    alignment: Alignment,
    out: &mut String,
) {
    out.clear();

    let box_width = width as usize;
    let box_height = height as usize;
    let lines: Vec<&str> = rendered.trim_end_matches('\n').lines().collect();
    let content_height = lines.len();
    let content_width = lines
        .iter()
        .map(|line| UnicodeWidthStr::width(*line))
        .max()
        .unwrap_or(0);

    let (h_align, v_align) = alignment_parts(alignment);

    let left_pad = match h_align {
        HAlign::Left => 0,
        HAlign::Center => (box_width - content_width) / 2,
        HAlign::Right => box_width - content_width,
    };

    let top_pad = match v_align {
        VAlign::Top => 0,
        VAlign::Middle => (box_height - content_height) / 2,
        VAlign::Bottom => box_height - content_height,
    };

    let blank_line = " ".repeat(box_width);
    if box_height > 0 {
        out.reserve((box_width + 1) * box_height - 1);
    }

    let mut line_count = 0;
    for _ in 0..top_pad {
        out.push_str(&blank_line);
        line_count += 1;
        if line_count < box_height {
            out.push('\n');
        }
    }

    for line in lines {
        let line_width = UnicodeWidthStr::width(line);
        let right_pad = box_width.saturating_sub(left_pad + line_width);

        push_spaces(out, left_pad);
        out.push_str(line);
        push_spaces(out, right_pad);

        line_count += 1;
        if line_count < box_height {
            out.push('\n');
        }
    }

    while line_count < box_height {
        out.push_str(&blank_line);
        line_count += 1;
        if line_count < box_height {
            out.push('\n');
        }
    }
}

fn push_spaces(out: &mut String, count: usize) {
    for _ in 0..count {
        out.push(' ');
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VAlign {
    Top,
    Middle,
    Bottom,
}

fn alignment_parts(alignment: Alignment) -> (HAlign, VAlign) {
    match alignment {
        Alignment::TopLeft => (HAlign::Left, VAlign::Top),
        Alignment::Top => (HAlign::Center, VAlign::Top),
        Alignment::TopRight => (HAlign::Right, VAlign::Top),
        Alignment::Left => (HAlign::Left, VAlign::Middle),
        Alignment::Center => (HAlign::Center, VAlign::Middle),
        Alignment::Right => (HAlign::Right, VAlign::Middle),
        Alignment::BottomLeft => (HAlign::Left, VAlign::Bottom),
        Alignment::Bottom => (HAlign::Center, VAlign::Bottom),
        Alignment::BottomRight => (HAlign::Right, VAlign::Bottom),
    }
}
