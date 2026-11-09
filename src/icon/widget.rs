use super::{IconSource, ResolvedIcon, resolve};
use crate::Theme;
use iced::advanced::{Widget, layout, mouse, renderer, widget::Tree};
use iced::advanced::{image as core_image, svg as core_svg};
use iced::widget::svg;
use iced::{Color, ContentFit, Length, Rectangle, Rotation, Size};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Mutex, OnceLock},
};

/// An icon that follows its parent's foreground and supports installed raster assets.
pub struct IconView<'a> {
    source: IconSource,
    width: Length,
    height: Length,
    size: f32,
    scale: u32,
    fit: ContentFit,
    rotation: Rotation,
    opacity: f32,
    weight: f32,
    style: Option<svg::StyleFn<'a, Theme>>,
    hovered: bool,
}
impl<'a> IconView<'a> {
    pub(super) fn new(source: IconSource, size: f32) -> Self {
        Self {
            source,
            width: Length::Fixed(size),
            height: Length::Fixed(size),
            size,
            scale: 1,
            fit: ContentFit::Contain,
            rotation: Rotation::default(),
            opacity: 1.0,
            weight: 0.0,
            style: None,
            hovered: false,
        }
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.width = value.into();
        self
    }
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.height = value.into();
        self
    }
    /// Output scale used for theme directory selection (default 1).
    pub fn scale(mut self, scale: u32) -> Self {
        self.scale = scale.max(1);
        self
    }
    pub fn content_fit(mut self, fit: ContentFit) -> Self {
        self.fit = fit;
        self
    }
    pub fn rotation(mut self, rotation: impl Into<Rotation>) -> Self {
        self.rotation = rotation.into();
        self
    }
    pub fn opacity(mut self, opacity: impl Into<f32>) -> Self {
        self.opacity = opacity.into().clamp(0.0, 1.0);
        self
    }
    /// Thicken SVG outlines by this many logical pixels on each side (default 0).
    /// Raster icons are unchanged. A small value such as 0.3 suits thin theme glyphs.
    pub fn weight(mut self, extra_pixels: f32) -> Self {
        self.weight = if extra_pixels.is_finite() {
            extra_pixels.clamp(0.0, 2.0)
        } else {
            0.0
        };
        self
    }
    pub fn style(mut self, style: impl Fn(&Theme, svg::Status) -> svg::Style + 'a) -> Self {
        self.style = Some(Box::new(style));
        self
    }
    pub fn class(mut self, class: svg::StyleFn<'a, Theme>) -> Self {
        self.style = Some(class);
        self
    }
    fn foreground(&self, theme: &Theme, inherited: Color, status: svg::Status) -> Color {
        self.style
            .as_ref()
            .and_then(|s| s(theme, status).color)
            .unwrap_or(inherited)
    }
}

#[derive(Clone)]
enum Asset {
    Svg(svg::Handle, Option<Color>),
    Image(core_image::Handle, Size),
}
#[derive(Hash, PartialEq, Eq)]
struct RenderKey {
    path: std::path::PathBuf,
    bytes: std::sync::Arc<[u8]>,
    palette: [[u8; 4]; 4],
    symbolic: bool,
}

/// Memoized icon work, dropped whenever the icon theme changes.
struct ThemeCache<K, V> {
    /// The icon theme the entries belong to, then the entries.
    state: Mutex<(String, HashMap<K, V>)>,
}
impl<K: Hash + Eq, V: Clone> ThemeCache<K, V> {
    fn new() -> Self {
        Self {
            state: Mutex::new(Default::default()),
        }
    }
    fn get_or_insert_with(&self, key: K, make: impl FnOnce() -> V) -> V {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let theme = crate::system::icon_theme();
        if state.0 != theme {
            *state = (theme, HashMap::new());
        }
        state.1.entry(key).or_insert_with(make).clone()
    }
}
static RENDERS: OnceLock<ThemeCache<RenderKey, Option<Asset>>> = OnceLock::new();
fn rgba(c: Color) -> [u8; 4] {
    c.into_rgba8()
}
fn css(c: [u8; 4]) -> String {
    format!("rgba({},{},{},{})", c[0], c[1], c[2], c[3] as f32 / 255.0)
}

fn symbolic_css(css: &str) -> String {
    static PAINT: OnceLock<regex::Regex> = OnceLock::new();
    PAINT
        .get_or_init(|| {
            regex::Regex::new(r"(?i)(^|[;{])(\s*(?:fill|stroke|color)\s*:\s*)([^;}]+)")
                .expect("constant CSS pattern")
        })
        .replace_all(css, |capture: &regex::Captures<'_>| {
            let value = capture[3].trim();
            if value
                .split('!')
                .next()
                .unwrap_or_default()
                .trim()
                .eq_ignore_ascii_case("none")
                || value.starts_with("url(")
            {
                capture[0].to_owned()
            } else {
                format!(
                    "{}{}currentColor{}",
                    &capture[1],
                    &capture[2],
                    if value.contains('!') {
                        " !important"
                    } else {
                        ""
                    }
                )
            }
        })
        .into_owned()
}

fn opening_tag_end(text: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, byte) in text.as_bytes().get(start..)?.iter().enumerate() {
        match (quote, byte) {
            (None, b'"' | b'\'') => quote = Some(*byte),
            (Some(q), b) if q == *b => quote = None,
            (None, b'>') => return Some(start + offset),
            _ => (),
        }
    }
    None
}

fn symbolic_svg(bytes: &[u8], palette: [[u8; 4]; 4]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(bytes).ok()?;
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .ok()?;
    let mut edits = Vec::new();
    for node in doc.descendants().filter(|n| n.is_element()) {
        let role = node
            .ancestors()
            .find_map(|n| {
                n.attribute("class").and_then(|c| {
                    c.split_whitespace().find_map(|c| match c {
                        "success" => Some(1),
                        "warning" => Some(2),
                        "error" => Some(3),
                        _ => None,
                    })
                })
            })
            .unwrap_or(0);
        let color = css(palette[role]);
        // CSS-only paints must follow currentColor too; inline color beats author theme rules.
        if node.tag_name().name() == "style" {
            for child in node.children().filter(|n| n.is_text()) {
                let range = child.range();
                edits.push((range.clone(), symbolic_css(&text[range])));
            }
        }
        if node.attribute("style").is_none() {
            let end = opening_tag_end(text, node.range().start)?;
            let insert = if text.as_bytes().get(end.wrapping_sub(1)) == Some(&b'/') {
                end - 1
            } else {
                end
            };
            edits.push((
                insert..insert,
                format!(" style=\"color: {color} !important\""),
            ));
        }
        for attr in node.attributes() {
            if matches!(attr.name(), "fill" | "stroke" | "color") {
                if attr.value() != "none" && !attr.value().starts_with("url(") {
                    edits.push((attr.range_value(), color.clone()));
                }
            } else if attr.name() == "style" {
                let style = format!("{};color: {} !important", symbolic_css(attr.value()), color)
                    .replace('&', "&amp;")
                    .replace('"', "&quot;")
                    .replace('\'', "&apos;");
                edits.push((attr.range_value(), style));
            }
        }
    }
    edits.sort_by_key(|(r, _)| r.start);
    let mut output = text.to_owned();
    for (range, value) in edits.into_iter().rev() {
        output.replace_range(range, &value);
    }
    let options = resvg::usvg::Options {
        style_sheet: Some(format!(
            "svg {{ color: {}; fill: {}; }} .success {{ fill: {}; color: {}; }} .warning {{ fill: {}; color: {}; }} .error {{ fill: {}; color: {}; }}",
            css(palette[0]),
            css(palette[0]),
            css(palette[1]),
            css(palette[1]),
            css(palette[2]),
            css(palette[2]),
            css(palette[3]),
            css(palette[3])
        )),
        ..Default::default()
    };
    let tree = resvg::usvg::Tree::from_str(&output, &options).ok()?;
    Some(tree.to_string(&Default::default()).into_bytes())
}
// A full-color regular fallback can contain foreground details on a colored fill.
// Flattening every paint would erase those details; only tint monochrome fallbacks.
fn monochrome_svg(bytes: &[u8]) -> bool {
    use resvg::usvg::{Group, Node, Paint};
    fn visit(group: &Group, first: &mut Option<[u8; 3]>) -> bool {
        group.children().iter().all(|node| match node {
            Node::Group(group) => visit(group, first),
            Node::Path(path) => path
                .fill()
                .map(|f| f.paint())
                .into_iter()
                .chain(path.stroke().map(|s| s.paint()))
                .all(|paint| {
                    if let Paint::Color(c) = paint {
                        let rgb = [c.red, c.green, c.blue];
                        match first {
                            Some(first) => *first == rgb,
                            None => {
                                *first = Some(rgb);
                                true
                            }
                        }
                    } else {
                        false
                    }
                }),
            _ => false,
        })
    }
    resvg::usvg::Tree::from_data(bytes, &Default::default())
        .is_ok_and(|tree| visit(tree.root(), &mut None))
}
fn monochrome_pixels(pixels: &[u8]) -> bool {
    let mut colors = pixels.chunks_exact(4).filter(|p| p[3] != 0);
    let Some(first) = colors.next() else {
        return true;
    };
    colors.all(|pixel| pixel[..3] == first[..3])
}

fn asset(icon: ResolvedIcon, theme: &Theme, foreground: Color) -> Option<Asset> {
    let palette = [
        rgba(foreground),
        rgba(crate::color::fade(
            theme.semantic.success.color,
            foreground.a,
        )),
        rgba(crate::color::fade(
            theme.semantic.warning.color,
            foreground.a,
        )),
        rgba(crate::color::fade(theme.semantic.error.color, foreground.a)),
    ];
    let key = RenderKey {
        path: icon.path.clone(),
        bytes: icon.bytes.clone(),
        palette: if icon.symbolic { palette } else { [[0; 4]; 4] },
        symbolic: icon.symbolic,
    };
    let declared_symbolic = icon
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.ends_with("-symbolic"));
    RENDERS
        .get_or_init(ThemeCache::new)
        .get_or_insert_with(key, || match icon.path.extension()?.to_str()? {
            "svg" => {
                let bytes = if icon.symbolic && (declared_symbolic || monochrome_svg(&icon.bytes)) {
                    symbolic_svg(&icon.bytes, palette)?
                } else {
                    icon.bytes.to_vec()
                };
                Some(Asset::Svg(svg::Handle::from_memory(bytes), None))
            }
            "png" => {
                let image = image::load_from_memory(&icon.bytes).ok()?.to_rgba8();
                let (w, h) = image.dimensions();
                let mut pixels = image.into_raw();
                if icon.symbolic && (declared_symbolic || monochrome_pixels(&pixels)) {
                    tint_pixels(&mut pixels, palette[0]);
                }
                Some(Asset::Image(
                    core_image::Handle::from_rgba(w, h, pixels),
                    Size::new(w as f32, h as f32),
                ))
            }
            "xpm" => {
                let decoded = decode_xpm(&icon.bytes)?;
                if icon.symbolic {
                    if let Asset::Image(
                        core_image::Handle::Rgba {
                            width,
                            height,
                            pixels,
                            ..
                        },
                        size,
                    ) = decoded
                    {
                        let mut pixels = pixels.to_vec();
                        if declared_symbolic || monochrome_pixels(&pixels) {
                            tint_pixels(&mut pixels, palette[0]);
                        }
                        return Some(Asset::Image(
                            core_image::Handle::from_rgba(width, height, pixels),
                            size,
                        ));
                    }
                }
                Some(decoded)
            }
            _ => None,
        })
}

type WeightKey = (u64, u32, u32);
static WEIGHTED: OnceLock<ThemeCache<WeightKey, svg::Handle>> = OnceLock::new();
fn weighted_svg(handle: svg::Handle, weight: f32, logical_size: f32) -> svg::Handle {
    if weight <= 0.0 || logical_size <= 0.0 {
        return handle;
    }
    let key = (handle.id(), weight.to_bits(), logical_size.to_bits());
    WEIGHTED.get_or_init(ThemeCache::new).get_or_insert_with(key, || {
        let data = match handle.data() {
            core_svg::Data::Bytes(bytes) => bytes.to_vec(),
            core_svg::Data::Path(path) => std::fs::read(path).unwrap_or_default(),
        };
        let Ok(tree) = resvg::usvg::Tree::from_data(&data,&Default::default()) else { return handle.clone(); };
        let radius = weight * tree.size().width().min(tree.size().height()) / logical_size;
        let mut source = tree.to_string(&Default::default());
        let Some(open) = source.find("<svg").and_then(|start| opening_tag_end(&source,start)) else { return handle.clone(); };
        let Some(close) = source.rfind("</svg>") else { return handle.clone(); };
        source.insert_str(close,"</g>");
        source.insert_str(open+1,&format!("<defs><filter id=\"iad-icon-weight\" x=\"-20%\" y=\"-20%\" width=\"140%\" height=\"140%\"><feMorphology in=\"SourceGraphic\" operator=\"dilate\" radius=\"{radius}\"/></filter></defs><g filter=\"url(#iad-icon-weight)\">"));
        svg::Handle::from_memory(source.into_bytes())
    })
}

fn tint_pixels(pixels: &mut [u8], foreground: [u8; 4]) {
    for pixel in pixels.chunks_exact_mut(4) {
        pixel[..3].copy_from_slice(&foreground[..3]);
        pixel[3] = (pixel[3] as u16 * foreground[3] as u16 / 255) as u8;
    }
}

fn xpm_color(value: &str) -> Option<[u8; 4]> {
    if value.eq_ignore_ascii_case("none") {
        return Some([0; 4]);
    }
    if let Some(hex) = value.strip_prefix('#') {
        if !matches!(hex.len(), 3 | 6 | 9 | 12) || !hex.is_ascii() {
            return None;
        }
        let digits = hex.len() / 3;
        let channel = |i| {
            u16::from_str_radix(&hex[i * digits..(i + 1) * digits], 16)
                .ok()
                .map(|v| (v as u32 * 255 / ((1u32 << (digits * 4)) - 1)) as u8)
        };
        return Some([channel(0)?, channel(1)?, channel(2)?, 255]);
    }
    let normalized = value.to_ascii_lowercase().replace(' ', "");
    if let Ok(c) = normalized.parse::<svgtypes::Color>() {
        return Some([c.red, c.green, c.blue, c.alpha]);
    }
    if let Some(level) = normalized
        .strip_prefix("gray")
        .or_else(|| normalized.strip_prefix("grey"))
        .and_then(|v| v.parse::<u8>().ok())
        .filter(|n| *n <= 100)
    {
        let level = (level as f32 * 2.55).round() as u8;
        return Some([level, level, level, 255]);
    }
    // XPM also permits X11 color names that are not part of the SVG/CSS palette.
    static X11: OnceLock<HashMap<String, [u8; 4]>> = OnceLock::new();
    X11.get_or_init(|| {
        std::fs::read_to_string("/usr/share/X11/rgb.txt")
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                let r = fields.next()?.parse().ok()?;
                let g = fields.next()?.parse().ok()?;
                let b = fields.next()?.parse().ok()?;
                let name = fields.collect::<String>().to_ascii_lowercase();
                Some((name, [r, g, b, 255]))
            })
            .collect()
    })
    .get(&normalized)
    .copied()
}

// XPM3 is a C string array; entries may share lines and use escaped pixel keys.
fn xpm_strings(text: &str) -> Option<Vec<String>> {
    let mut chars = text.chars().peekable();
    let mut strings = Vec::new();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(c) = chars.next() {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'/') {
            for c in chars.by_ref() {
                if c == '\n' {
                    break;
                }
            }
        } else if c == '"' {
            let mut string = String::new();
            loop {
                match chars.next()? {
                    '"' => break,
                    '\\' => {
                        let escaped = chars.next()?;
                        match escaped {
                            '\n' => (),
                            'n' => string.push('\n'),
                            'r' => string.push('\r'),
                            't' => string.push('\t'),
                            '0'..='7' => {
                                let mut value = escaped.to_digit(8)?;
                                for _ in 0..2 {
                                    if let Some(digit) = chars.peek().and_then(|c| c.to_digit(8)) {
                                        chars.next();
                                        value = value * 8 + digit;
                                    } else {
                                        break;
                                    }
                                }
                                string.push(char::from_u32(value)?);
                            }
                            _ => string.push(escaped),
                        }
                    }
                    c => string.push(c),
                }
            }
            strings.push(string);
        }
    }
    Some(strings)
}

fn decode_xpm(bytes: &[u8]) -> Option<Asset> {
    let text = std::str::from_utf8(bytes).ok()?;
    let lines: Vec<String> = if text.trim_start().starts_with("! XPM2") {
        text.lines()
            .filter(|l| !l.starts_with('!'))
            .map(str::to_owned)
            .collect()
    } else {
        xpm_strings(text)?
    };
    let header: Vec<usize> = lines
        .first()?
        .split_whitespace()
        .take(4)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let [w, h, n, cpp] = *header.as_slice() else {
        return None;
    };
    if w == 0 || h == 0 || cpp == 0 || w.checked_mul(h)? > 16_777_216 {
        return None;
    }
    let mut colors = HashMap::new();
    for line in lines.get(1..=n)? {
        let key = line.get(..cpp)?;
        let fields: Vec<_> = line.get(cpp..)?.split_whitespace().collect();
        let start = ["c", "g", "g4", "m"]
            .into_iter()
            .find_map(|k| fields.iter().position(|v| *v == k))?
            + 1;
        let end = (start..fields.len())
            .find(|i| matches!(fields[*i], "s" | "c" | "g" | "g4" | "m"))
            .unwrap_or(fields.len());
        let color = xpm_color(&fields.get(start..end)?.join(" "))?;
        colors.insert(key, color);
    }
    let mut pixels = Vec::with_capacity(w * h * 4);
    for row in lines.get(n + 1..n + 1 + h)? {
        for x in 0..w {
            pixels.extend_from_slice(colors.get(row.get(x * cpp..(x + 1) * cpp)?)?);
        }
    }
    Some(Asset::Image(
        core_image::Handle::from_rgba(w as u32, h as u32, pixels),
        Size::new(w as f32, h as f32),
    ))
}
impl<M> Widget<M, Theme, iced::Renderer> for IconView<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }
    fn layout(
        &mut self,
        _: &mut Tree,
        _: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(self.width, self.height, Size::new(self.size, self.size)))
    }
    fn update(
        &mut self,
        _: &mut Tree,
        _: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
        _: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        _: &Rectangle,
    ) {
        let hovered = cursor.is_over(layout.bounds());
        if self.hovered != hovered {
            self.hovered = hovered;
            if self.style.is_some() {
                shell.request_redraw();
            }
        }
    }
    fn draw(
        &self,
        _: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        inherited: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let status = if cursor.is_over(bounds) {
            svg::Status::Hovered
        } else {
            svg::Status::Idle
        };
        let fg = self.foreground(theme, inherited.text_color, status);
        let size = bounds.width.min(bounds.height).ceil().max(1.0) as u32;
        let icon = match &self.source {
            IconSource::Svg(handle) => Some(Asset::Svg(handle.clone(), Some(fg))),
            IconSource::Icon(icon) => icon
                .resolve(size, self.scale)
                .and_then(|i| asset(i, theme, fg)),
            IconSource::Named { name, symbolic } => {
                resolve(name, size, self.scale, *symbolic).and_then(|i| asset(i, theme, fg))
            }
        };
        let Some(icon) = icon else {
            return;
        };
        let natural = match &icon {
            Asset::Svg(h, _) => {
                let s = core_svg::Renderer::measure_svg(renderer, h);
                Size::new(s.width as f32, s.height as f32)
            }
            Asset::Image(_, s) => *s,
        };
        if natural.width <= 0.0 || natural.height <= 0.0 {
            return;
        }
        let rotated = self.rotation.apply(natural);
        let fitted = self.fit.fit(rotated, bounds.size());
        let final_size = Size::new(
            natural.width * fitted.width / rotated.width,
            natural.height * fitted.height / rotated.height,
        );
        let drawing = Rectangle::new(
            iced::Point::new(
                bounds.center_x() - final_size.width / 2.0,
                bounds.center_y() - final_size.height / 2.0,
            ),
            final_size,
        );
        match icon {
            Asset::Svg(handle, color) => core_svg::Renderer::draw_svg(
                renderer,
                core_svg::Svg {
                    handle: weighted_svg(handle, self.weight, bounds.width.min(bounds.height)),
                    color,
                    rotation: self.rotation.radians(),
                    opacity: self.opacity * color.map_or(1.0, |c| c.a),
                },
                drawing,
                bounds,
            ),
            Asset::Image(handle, _) => core_image::Renderer::draw_image(
                renderer,
                core_image::Image::new(handle)
                    .rotation(self.rotation.radians())
                    .opacity(self.opacity),
                drawing,
                bounds,
            ),
        }
    }
}
impl<'a, M: 'a> From<IconView<'a>> for iced::Element<'a, M, Theme> {
    fn from(icon: IconView<'a>) -> Self {
        Self::new(icon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pixel(svg: &[u8], x: usize) -> [u8; 4] {
        let tree = resvg::usvg::Tree::from_data(svg, &Default::default()).unwrap();
        let mut pixmap = resvg::tiny_skia::Pixmap::new(4, 1).unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        pixmap.data()[x * 4..x * 4 + 4].try_into().unwrap()
    }
    const SYMBOL: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="1"><rect width="1" height="1" fill="#222"/><g class="success"><rect x="1" width="1" height="1" fill="#999"/></g><rect class="warning" x="2" width="1" height="1" style="fill:#555;stroke:none"/><rect class="error" x="3" width="1" height="1"/></svg>"##;
    #[test]
    fn extra_weight_thickens_a_system_svg_without_changing_its_bounds() {
        let handle = svg::Handle::from_memory(br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><path fill="#fff" d="M7 3h1v10H7z"/></svg>"##.to_vec());
        let weighted = weighted_svg(handle.clone(), 0.6, 16.0);
        assert_ne!(weighted, handle);
        let raster = |handle: &svg::Handle| {
            let core_svg::Data::Bytes(bytes) = handle.data() else {
                panic!("memory SVG")
            };
            let tree = resvg::usvg::Tree::from_data(bytes, &Default::default()).unwrap();
            assert_eq!(tree.size().width(), 16.0);
            let mut pixmap = resvg::tiny_skia::Pixmap::new(16, 16).unwrap();
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::identity(),
                &mut pixmap.as_mut(),
            );
            pixmap
                .data()
                .chunks_exact(4)
                .map(|p| p[3] as u32)
                .sum::<u32>()
        };
        assert!(raster(&weighted) > raster(&handle));
    }

    #[test]
    fn monochrome_fallbacks_can_be_tinted_without_erasing_full_color_details() {
        assert!(monochrome_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="1"><path fill="#333" d="M0 0h4v1H0z"/></svg>"##));
        assert!(!monochrome_svg(SYMBOL));
        assert!(monochrome_pixels(&[0, 0, 0, 255, 0, 0, 0, 128]));
        assert!(!monochrome_pixels(&[255, 255, 255, 255, 0, 0, 0, 255]));
    }

    #[test]
    fn foreground_follows_parent_state_theme_and_explicit_override() {
        use iced::widget::button::Status;
        let view = super::super::Icon::Check.view(16.0);
        for theme in [
            Theme::light(),
            Theme::dark(),
            Theme::light().with_tint(Some(Color::from_rgba(0.0, 0.0, 0.0, 0.9))),
        ] {
            for status in [
                Status::Active,
                Status::Hovered,
                Status::Pressed,
                Status::Disabled,
            ] {
                for style in [
                    crate::styles::button::suggested(&theme, status),
                    crate::styles::button::destructive(&theme, status),
                    crate::styles::button::headerbar(&theme, status),
                ] {
                    assert_eq!(
                        view.foreground(&theme, style.text_color, svg::Status::Idle),
                        style.text_color
                    );
                }
            }
            let explicit = super::super::Icon::Check.colored(16.0, Color::from_rgb(0.3, 0.5, 0.8));
            assert_eq!(
                explicit.foreground(&theme, Color::BLACK, svg::Status::Idle),
                Color::from_rgb(0.3, 0.5, 0.8)
            );
            for surface in [
                theme.semantic.window,
                theme.semantic.headerbar,
                theme.semantic.active_toggle,
            ] {
                assert!(
                    crate::color::contrast_ratio(
                        surface.fg,
                        crate::color::composite_over(surface.bg, theme.semantic.window.bg)
                    ) >= 4.49
                );
            }
        }
    }
    #[test]
    fn regular_assets_keep_colors_and_symbolic_rasters_follow_foreground() {
        let bytes: std::sync::Arc<[u8]> = SYMBOL.into();
        let theme = Theme::light();
        let resolved = |symbolic| ResolvedIcon {
            path: if symbolic {
                "fixture-symbolic.svg".into()
            } else {
                "fixture.svg".into()
            },
            bytes: bytes.clone(),
            symbolic,
        };
        let Some(Asset::Svg(regular, None)) = asset(resolved(false), &theme, Color::WHITE) else {
            panic!("SVG expected")
        };
        let Some(Asset::Svg(regular_again, None)) = asset(resolved(false), &theme, Color::BLACK)
        else {
            panic!("SVG expected")
        };
        assert_eq!(regular, regular_again);
        let Some(Asset::Svg(first, None)) = asset(resolved(true), &theme, Color::WHITE) else {
            panic!("SVG expected")
        };
        let Some(Asset::Svg(second, None)) = asset(resolved(true), &theme, Color::BLACK) else {
            panic!("SVG expected")
        };
        assert_ne!(first, second);
        let mut pixels = [255, 0, 0, 128, 0, 255, 0, 0];
        tint_pixels(&mut pixels, [0, 0, 255, 128]);
        assert_eq!(pixels, [0, 0, 255, 64, 0, 0, 255, 0]);
    }

    #[test]
    fn stylesheet_paints_and_semantic_strokes_follow_foreground() {
        let source = br##"<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg" width="4" height="1"><style>.paint { color:#123;fill:#333 !important;stroke:none; }</style><rect class="paint" width="1" height="1"/><rect class="paint success" x="1" width="1" height="1"/><rect class="paint warning" x="2" width="1" height="1"/><rect class="paint error" x="3" width="1" height="1"/></svg>"##;
        let palette = [
            [255, 255, 255, 255],
            [0, 255, 0, 255],
            [255, 255, 0, 255],
            [255, 0, 0, 255],
        ];
        let svg = symbolic_svg(source, palette).unwrap();
        for (x, color) in palette.into_iter().enumerate() {
            assert_eq!(pixel(&svg, x), color);
        }
    }

    #[test]
    fn symbolic_foreground_semantic_colors_and_state_alpha() {
        let palette = [
            [255, 255, 255, 255],
            [0, 255, 0, 255],
            [255, 255, 0, 255],
            [255, 0, 0, 255],
        ];
        let svg = symbolic_svg(SYMBOL, palette).unwrap();
        for (x, color) in palette.into_iter().enumerate() {
            assert_eq!(pixel(&svg, x), color);
        }
        let changed = symbolic_svg(SYMBOL, [[0, 0, 255, 128]; 4]).unwrap();
        assert_eq!(pixel(&changed, 0), [0, 0, 128, 128]);
        assert_ne!(svg, changed);
        assert!(symbolic_svg(b"broken", palette).is_none());
    }
    #[test]
    fn xpm_decodes_transparency_multichar_pixels_and_rejects_invalid_data() {
        let xpm = b"/* XPM */\nstatic char *x[]={\n\"2 1 2 2\",\n\"aa c #fff\",\n\"bb c None\",\n\"aabb\"};";
        let Some(Asset::Image(
            core_image::Handle::Rgba {
                width,
                height,
                pixels,
                ..
            },
            _,
        )) = decode_xpm(xpm)
        else {
            panic!("XPM must decode")
        };
        assert_eq!((width, height), (2, 1));
        assert_eq!(pixels.as_ref(), [255, 255, 255, 255, 0, 0, 0, 0]);
        assert!(
            decode_xpm(
                br##"/* comment with "quotes" */ char *x[]={"1 1 1 1","x c #123456789","x"};"##
            )
            .is_some()
        );
        assert!(decode_xpm(b"! XPM2\n1 1 1 1\nx c gray50\nx").is_some());
        assert!(decode_xpm(b"invalid").is_none());
    }
    #[test]
    fn installed_linux_themes_resolve_and_render() {
        let roots = vec![std::path::PathBuf::from("/usr/share/icons")];
        for theme in ["Adwaita", "Papirus", "breeze"] {
            if !roots[0].join(theme).join("index.theme").exists() {
                continue;
            }
            let mut count = 0;
            for icon in super::super::Icon::ALL {
                if let Some(resolved) =
                    super::super::resolver::lookup(&roots, theme, icon.name(), 16, 1, true)
                {
                    let path = resolved.path.clone();
                    assert!(
                        asset(resolved, &Theme::light(), Color::BLACK).is_some(),
                        "{theme}: {} at {}",
                        icon.name(),
                        path.display()
                    );
                    count += 1;
                }
            }
            assert!(count > 10, "{theme} resolved {count} icons");
            eprintln!(
                "{theme}: {count}/{} predefined icons resolved and decoded",
                super::super::Icon::ALL.len()
            );
        }
    }
}
