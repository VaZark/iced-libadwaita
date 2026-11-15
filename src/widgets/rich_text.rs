//! `RichText`: draws a [`rich_text`](crate::primitives::rich_text) document in the theme:
//! headings, paragraphs with emphasis, strike-through, highlight, inline code, links, sub- and
//! superscript and footnote marks, block quotes, bulleted, numbered and task lists, code blocks,
//! rules, tables, definition lists and a footnote section.
//!
//! It shows a document; it does not parse or edit one. Build the [`Block`]s from whatever the
//! app stores. Controlled: a task's checkbox only reports the toggle (`on_task`), and the app
//! changes its document.
//!
//! ```ignore
//! RichText::new(&theme, &self.blocks)
//!     .on_link(Msg::OpenUrl)               // a link was pressed
//!     .on_task(|n, on| Msg::Task(n, on))   // task `n` (reading order) was toggled
//! ```
//!
//! [`compact`](RichText::compact) draws it smaller and tighter, for a card;
//! [`max_blocks`](RichText::max_blocks) shows only the first few blocks.

use crate::Element;
use crate::Theme;
use crate::color::alpha;
use crate::components::{check_button, divider};
use crate::primitives::rich_text::{self as rt, Align, Block, Definition, Footnote, Inline, Item};
use iced::font::{Family, Style as FontStyle, Weight};
use iced::widget::text::Span;
use iced::widget::{column, container, image, rich_text, row, space, span, stack, text};
use iced::{Alignment, Background, Border, Color, Font, Length, Padding};
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

/// Colors and sizes read once from the theme.
#[derive(Clone, Copy)]
struct Look {
    accent: Color,
    dim: Color,
    code_bg: Color,
    mark_bg: Color,
    bar: Color,
    radius: f32,
    sizes: [f32; 6],
    body: f32,
    caption: f32,
    mono: f32,
}

impl Look {
    fn of(theme: &Theme) -> Self {
        let t = &theme.typography;
        let fg = theme.semantic.window.fg;
        Self {
            accent: theme.semantic.accent.color,
            dim: alpha(fg, 0.7),
            code_bg: alpha(fg, 0.10),
            mark_bg: alpha(theme.semantic.warning.color, 0.45),
            bar: alpha(theme.semantic.accent.color, 0.7),
            radius: theme.metrics.radius_m,
            sizes: [
                t.title_1,
                t.title_2,
                t.title_3,
                t.title_4,
                t.heading,
                t.caption_heading,
            ],
            body: t.body,
            caption: t.caption,
            mono: t.monospace,
        }
    }

    fn scaled(self, k: f32) -> Self {
        Self {
            sizes: self.sizes.map(|s| s * k),
            body: self.body * k,
            caption: self.caption * k,
            mono: self.mono * k,
            ..self
        }
    }
}

/// How text is drawn at this point of a line.
#[derive(Clone, Default)]
struct Style {
    bold: bool,
    italic: bool,
    strike: bool,
    mark: bool,
    mono: bool,
    code: bool,
    link: Option<String>,
    script: Option<Script>,
    size: f32,
    color: Option<Color>,
}

#[derive(Clone, Copy)]
enum Script {
    Sub,
    Super,
}

pub struct RichText<'a, M> {
    props: RichTextWidgetProps,
    blocks: &'a [Block],
    look: Look,
    compact: bool,
    max_blocks: Option<usize>,
    on_link: Option<Rc<dyn Fn(String) -> M + 'a>>,
    on_task: Option<Rc<dyn Fn(usize, bool) -> M + 'a>>,
    spacing: Option<f32>,
    code_padding: Option<f32>,
    code_spacing: Option<f32>,
    quote_indent: Option<f32>,
    marker_width: Option<f32>,
    marker_spacing: Option<f32>,
}

/// Where we are while drawing: footnote numbers, the task counter, and the callbacks.
struct Ctx<'a, M> {
    props: RichTextWidgetProps,
    look: Look,
    gap: f32,
    order: Vec<String>,
    tasks: Cell<usize>,
    on_link: Option<Rc<dyn Fn(String) -> M + 'a>>,
    on_task: Option<Rc<dyn Fn(usize, bool) -> M + 'a>>,
    code_padding: f32,
    code_spacing: f32,
    quote_indent: f32,
    marker_width: f32,
    marker_spacing: f32,
}

impl<'a, M: Clone + 'a> RichText<'a, M> {
    /// Presentation overrides for native children; actions remain parent-owned.
    pub fn props(mut self, props: RichTextWidgetProps) -> Self {
        self.props = props;
        self
    }
    pub fn new(theme: &Theme, blocks: &'a [Block]) -> Self {
        Self {
            props: RichTextWidgetProps::default(),
            blocks,
            look: Look::of(theme),
            compact: false,
            max_blocks: None,
            on_link: None,
            on_task: None,
            spacing: None,
            code_padding: None,
            code_spacing: None,
            quote_indent: None,
            marker_width: None,
            marker_spacing: None,
        }
    }

    /// Smaller text and tighter spacing, for a card.
    pub fn compact(mut self, on: bool) -> Self {
        self.compact = on;
        self
    }

    /// The gap between blocks (default: the theme's `space_m`, `space_s` when compact).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The padding inside a code block (default: the theme's `code_padding`).
    pub fn code_padding(mut self, px: f32) -> Self {
        self.code_padding = Some(px);
        self
    }
    /// The gap between a code block's language and its code (default: the theme's `space_xs`).
    pub fn code_spacing(mut self, px: f32) -> Self {
        self.code_spacing = Some(px);
        self
    }
    /// How far a quote's text is indented past its bar (default 14).
    pub fn quote_indent(mut self, px: f32) -> Self {
        self.quote_indent = Some(px);
        self
    }
    /// The width of a list marker's column (default 24).
    pub fn marker_width(mut self, px: f32) -> Self {
        self.marker_width = Some(px);
        self
    }
    /// The gap between a list marker and its item (default: the theme's `gap`).
    pub fn marker_spacing(mut self, px: f32) -> Self {
        self.marker_spacing = Some(px);
        self
    }

    /// Draw only the first `n` blocks.
    pub fn max_blocks(mut self, n: usize) -> Self {
        self.max_blocks = Some(n);
        self
    }

    /// A link was pressed; gets its address. Without this, links are drawn but do nothing.
    pub fn on_link(mut self, f: impl Fn(String) -> M + 'a) -> Self {
        self.on_link = Some(Rc::new(f));
        self
    }

    /// A task-list checkbox was toggled: its number in reading order (see
    /// [`task_count`](rt::task_count)) and whether it is now checked. Without this, task
    /// checkboxes are drawn but cannot be toggled.
    pub fn on_task(mut self, f: impl Fn(usize, bool) -> M + 'a) -> Self {
        self.on_task = Some(Rc::new(f));
        self
    }
}

impl<'a, M: Clone + 'a> From<RichText<'a, M>> for Element<'a, M> {
    fn from(r: RichText<'a, M>) -> Self {
        let k = if r.compact { 0.9 } else { 1.0 };
        let m = crate::Metrics::desktop();
        let cx = Ctx {
            props: r.props,
            look: r.look.scaled(k),
            gap: r
                .spacing
                .unwrap_or(if r.compact { m.space_s } else { m.space_m }),
            order: rt::footnote_order(r.blocks),
            tasks: Cell::new(0),
            on_link: r.on_link,
            on_task: r.on_task,
            code_padding: r.code_padding.unwrap_or(m.code_padding),
            code_spacing: r.code_spacing.unwrap_or(m.space_xs),
            quote_indent: r.quote_indent.unwrap_or(14.0),
            marker_width: r.marker_width.unwrap_or(24.0),
            marker_spacing: r.marker_spacing.unwrap_or(m.gap),
        };
        let shown = r
            .max_blocks
            .map_or(r.blocks, |n| &r.blocks[..n.min(r.blocks.len())]);
        blocks(&cx, shown, 0)
    }
}

// ---- inlines -------------------------------------------------------------------------------

fn font(st: &Style) -> Font {
    Font {
        family: if st.mono {
            Family::Monospace
        } else {
            Font::DEFAULT.family
        },
        weight: if st.bold {
            Weight::Bold
        } else {
            Weight::Normal
        },
        style: if st.italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        },
        ..Font::DEFAULT
    }
}

fn one<'a>(
    cx_look: &Look,
    props: &RichTextLineProps,
    text: String,
    st: &Style,
) -> Span<'a, String, Font> {
    let mut s = span(text).size(st.size).font(font(st));
    if let Some(c) = st.color {
        s = s.color(c);
    }
    if st.mark {
        s = s.background(Background::Color(cx_look.mark_bg));
    }
    if st.code {
        s = s
            .background(Background::Color(cx_look.code_bg))
            .border(Border::default().rounded(cx_look.radius / 2.0))
            .padding([0.0, crate::Metrics::desktop().inline_padding_x]);
    }
    if let Some(url) = &st.link {
        s = s.link(url.clone()).underline(true);
    }
    if let Some(size) = props.size {
        s = s.size(size);
    }
    if let Some(font) = props.font {
        s = s.font(font);
    }
    if let Some(color) = props.color {
        s = s.color(color);
    }
    s.strikethrough(st.strike)
}

fn spans<'a, M>(
    cx: &Ctx<'a, M>,
    inlines: &[Inline],
    st: &Style,
    out: &mut Vec<Span<'a, String, Font>>,
) {
    let with = |f: &dyn Fn(&mut Style)| {
        let mut s = st.clone();
        f(&mut s);
        s
    };
    for i in inlines {
        match i {
            Inline::Text(t) => match st.script {
                None => out.push(one(&cx.look, &cx.props.line, t.clone(), st)),
                Some(sc) => {
                    let mapped = match sc {
                        Script::Super => rt::superscript(t),
                        Script::Sub => rt::subscript(t),
                    };
                    // A character with no script form is drawn small and dim instead.
                    out.push(match mapped {
                        Some(m) => one(&cx.look, &cx.props.line, m, st),
                        None => one(
                            &cx.look,
                            &cx.props.line,
                            t.clone(),
                            &Style {
                                size: st.size * 0.7,
                                color: Some(cx.look.dim),
                                ..st.clone()
                            },
                        ),
                    });
                }
            },
            Inline::Code(c) => out.push(one(
                &cx.look,
                &cx.props.line,
                c.clone(),
                &Style {
                    mono: true,
                    code: true,
                    size: cx.look.mono,
                    ..st.clone()
                },
            )),
            Inline::Emphasis(c) => spans(cx, c, &with(&|s| s.italic = true), out),
            Inline::Strong(c) => spans(cx, c, &with(&|s| s.bold = true), out),
            Inline::Strike(c) => spans(cx, c, &with(&|s| s.strike = true), out),
            Inline::Highlight(c) => spans(cx, c, &with(&|s| s.mark = true), out),
            Inline::Subscript(c) => spans(cx, c, &with(&|s| s.script = Some(Script::Sub)), out),
            Inline::Superscript(c) => spans(cx, c, &with(&|s| s.script = Some(Script::Super)), out),
            Inline::Link { content, url } => {
                let s = with(&|s| {
                    s.link = Some(url.clone());
                    s.color = Some(cx.look.accent);
                });
                spans(cx, content, &s, out);
            }
            Inline::Image { alt, url } => {
                let label = if alt.is_empty() {
                    url.as_str()
                } else {
                    alt.as_str()
                };
                let s = with(&|s| {
                    s.link = Some(url.clone());
                    s.color = Some(cx.look.accent);
                    s.italic = true;
                });
                out.push(one(&cx.look, &cx.props.line, format!("▣ {label}"), &s));
            }
            Inline::FootnoteRef(label) => {
                let n = cx
                    .order
                    .iter()
                    .position(|l| l == label)
                    .map_or(0, |p| p + 1);
                let mark = rt::superscript(&n.to_string()).unwrap_or_else(|| format!("[{n}]"));
                let s = with(&|s| s.color = Some(cx.look.accent));
                out.push(one(&cx.look, &cx.props.line, mark, &s));
            }
            Inline::LineBreak => out.push(one(&cx.look, &cx.props.line, "\n".into(), st)),
        }
    }
}

fn line<'a, M: Clone + 'a>(cx: &Ctx<'a, M>, inlines: &[Inline], st: Style) -> Element<'a, M> {
    line_aligned(cx, inlines, st, iced::alignment::Horizontal::Left)
}

fn line_aligned<'a, M: Clone + 'a>(
    cx: &Ctx<'a, M>,
    inlines: &[Inline],
    st: Style,
    h: iced::alignment::Horizontal,
) -> Element<'a, M> {
    let mut out = Vec::new();
    spans(cx, inlines, &st, &mut out);
    let mut r = rich_text::<String, M, Theme, iced::Renderer>(out)
        .size(st.size)
        .align_x(h)
        .width(Length::Fill);
    if let Some(f) = &cx.on_link {
        let f = f.clone();
        r = r.on_link_click(move |url| f(url));
    }
    cx.props.line.apply(r).into()
}

fn body_style<M>(cx: &Ctx<'_, M>) -> Style {
    Style {
        size: cx.look.body,
        ..Style::default()
    }
}

// ---- blocks --------------------------------------------------------------------------------

fn blocks<'a, M: Clone + 'a>(cx: &Ctx<'a, M>, bs: &'a [Block], depth: usize) -> Element<'a, M> {
    column(bs.iter().map(|b| block(cx, b, depth)))
        .spacing(cx.gap)
        .width(Length::Fill)
        .into()
}

fn block<'a, M: Clone + 'a>(cx: &Ctx<'a, M>, b: &'a Block, depth: usize) -> Element<'a, M> {
    match b {
        Block::Heading { level, content, .. } => {
            let i = usize::from(level.clamp(&1, &6) - 1);
            let st = Style {
                bold: true,
                size: cx.look.sizes[i],
                ..Style::default()
            };
            line(cx, content, st)
        }
        Block::Paragraph(content) => match content.as_slice() {
            // A picture on its own is drawn as one, when it is a file we can read.
            [Inline::Image { url, .. }] if Path::new(url).is_file() => cx
                .props
                .image_container
                .apply(
                    container(
                        image(url.clone())
                            .width(Length::Shrink)
                            .height(Length::Shrink),
                    )
                    .max_width(420.0),
                )
                .into(),
            _ => line(cx, content, body_style(cx)),
        },
        Block::Quote(inner) => {
            let bar = cx.look.bar;
            // The bar is a second layer over the text, so it is as tall as the text: a `Fill`
            // bar in a row would have no height to fill.
            stack![
                cx.props.quote.apply(
                    container(blocks(cx, inner, depth))
                        .padding(Padding::ZERO.left(cx.quote_indent))
                ),
                cx.props
                    .quote_bar
                    .apply(container(space().width(3.0).height(Length::Fill)).style(
                        move |_: &Theme| {
                            container::Style {
                                background: Some(Background::Color(bar)),
                                border: Border::default().rounded(2.0),
                                ..container::Style::default()
                            }
                        }
                    )),
            ]
            .width(Length::Fill)
            .into()
        }
        Block::List { start, items } => list(cx, *start, items, depth),
        Block::CodeBlock { lang, code } => {
            let mono = Style {
                mono: true,
                size: cx.look.mono,
                ..Style::default()
            };
            let mut col = column![].spacing(cx.code_spacing);
            if let Some(l) = lang.as_deref().filter(|l| !l.is_empty()) {
                col = col.push(
                    cx.props
                        .language
                        .apply(text(l.to_string()).size(cx.look.caption).color(cx.look.dim)),
                );
            }
            col = col.push(
                cx.props.code.apply(
                    text(code.trim_end().to_string())
                        .size(mono.size)
                        .font(font(&mono)),
                ),
            );
            let (bg, radius) = (cx.look.code_bg, cx.look.radius);
            cx.props
                .code_container
                .apply(
                    container(col)
                        .padding(cx.code_padding)
                        .width(Length::Fill)
                        .style(move |_: &Theme| container::Style {
                            background: Some(Background::Color(bg)),
                            border: Border::default().rounded(radius),
                            ..container::Style::default()
                        }),
                )
                .into()
        }
        Block::Rule => divider().into(),
        Block::Table { align, head, rows } => table(cx, align, head, rows),
        Block::Definitions(defs) => definitions(cx, defs, depth),
        Block::Footnotes(notes) => footnotes(cx, notes, depth),
    }
}

fn list<'a, M: Clone + 'a>(
    cx: &Ctx<'a, M>,
    start: Option<u64>,
    items: &'a [Item],
    depth: usize,
) -> Element<'a, M> {
    let rows = items.iter().enumerate().map(|(n, item)| {
        let lead: Element<'a, M> = match item.checked {
            Some(done) => {
                let index = cx.tasks.get();
                cx.tasks.set(index + 1);
                let mut c = check_button("", done);
                if let Some(f) = &cx.on_task {
                    let f = f.clone();
                    c = c.on_toggle(move |on| f(index, on));
                }
                c.into()
            }
            None => cx
                .props
                .list_marker
                .apply(
                    text(rt::marker(start, n, depth))
                        .size(cx.look.body)
                        .color(cx.look.dim),
                )
                .into(),
        };
        let content = blocks(cx, &item.blocks, depth + 1);
        row![
            cx.props.marker_container.apply(
                container(lead)
                    .width(cx.marker_width)
                    .align_x(Alignment::End)
            ),
            content
        ]
        .spacing(cx.marker_spacing)
        .into()
    });
    column(rows)
        .spacing(cx.gap / 2.0)
        .width(Length::Fill)
        .into()
}

fn table<'a, M: Clone + 'a>(
    cx: &Ctx<'a, M>,
    aligns: &[Align],
    head: &'a [Vec<Inline>],
    rows: &'a [Vec<Vec<Inline>>],
) -> Element<'a, M> {
    let cols = rt::columns(head, rows);
    let one_row = |cells: &'a [Vec<Inline>], bold: bool| -> Element<'a, M> {
        row((0..cols).map(|c| {
            let st = Style {
                bold,
                size: cx.look.body,
                ..Style::default()
            };
            let h = match rt::align(aligns, c) {
                Align::Left => iced::alignment::Horizontal::Left,
                Align::Center => iced::alignment::Horizontal::Center,
                Align::Right => iced::alignment::Horizontal::Right,
            };
            cx.props
                .table_cell
                .apply(
                    container(line_aligned(cx, rt::cell(cells, c), st, h))
                        .width(Length::FillPortion(1))
                        .padding([
                            crate::Metrics::desktop().space_xs,
                            crate::Metrics::desktop().gap,
                        ]),
                )
                .into()
        }))
        .into()
    };
    let mut col =
        column![one_row(head, true), divider()].spacing(crate::Metrics::desktop().space_xxs);
    for r in rows {
        col = col.push(one_row(r, false));
    }
    col.width(Length::Fill).into()
}

fn definitions<'a, M: Clone + 'a>(
    cx: &Ctx<'a, M>,
    defs: &'a [Definition],
    _depth: usize,
) -> Element<'a, M> {
    column(defs.iter().map(|d| {
        let term = Style {
            bold: true,
            size: cx.look.body,
            ..Style::default()
        };
        let mut col = column![line(cx, &d.term, term)].spacing(crate::Metrics::desktop().space_xxs);
        for detail in &d.details {
            col = col.push(cx.props.definition_detail.apply(
                container(line(cx, detail, body_style(cx))).padding(Padding::ZERO.left(16.0)),
            ));
        }
        col.into()
    }))
    .spacing(cx.gap / 2.0)
    .into()
}

fn footnotes<'a, M: Clone + 'a>(
    cx: &Ctx<'a, M>,
    notes: &'a [Footnote],
    depth: usize,
) -> Element<'a, M> {
    // Numbered as they are first referenced; notes nobody refers to go last.
    let number = |n: &Footnote| {
        cx.order
            .iter()
            .position(|l| *l == n.label)
            .unwrap_or(usize::MAX)
    };
    let mut sorted: Vec<&Footnote> = notes.iter().collect();
    sorted.sort_by_key(|n| number(n));
    let small = Ctx {
        props: cx.props.clone(),
        look: cx.look.scaled(0.9),
        gap: cx.gap / 2.0,
        order: cx.order.clone(),
        tasks: Cell::new(0),
        on_link: cx.on_link.clone(),
        on_task: None,
        code_padding: cx.code_padding,
        code_spacing: cx.code_spacing,
        quote_indent: cx.quote_indent,
        marker_width: cx.marker_width,
        marker_spacing: cx.marker_spacing,
    };
    let rows = sorted.into_iter().enumerate().map(|(i, n)| {
        let k = number(n);
        let label = if k == usize::MAX { i + 1 } else { k + 1 };
        row![
            cx.props.footnote_marker_container.apply(
                container(
                    cx.props.footnote_marker.apply(
                        text(format!("{label}."))
                            .size(small.look.body)
                            .color(small.look.dim)
                    )
                )
                .width(24.0)
                .align_x(Alignment::End)
            ),
            blocks(&small, &n.blocks, depth),
        ]
        .spacing(crate::Metrics::desktop().gap)
        .into()
    });
    column![divider(), column(rows).spacing(cx.gap / 2.0)]
        .spacing(cx.gap / 2.0)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rt::{Block, Inline, Item};

    #[test]
    fn the_gaps_and_indents_are_parameters() {
        let blocks = vec![
            Block::Quote(vec![Block::Rule]),
            Block::List {
                start: None,
                items: vec![Item {
                    checked: None,
                    blocks: vec![Block::Paragraph(vec![Inline::Text("x".into())])],
                }],
            },
            Block::CodeBlock {
                lang: Some("rs".into()),
                code: "let x = 1;".into(),
            },
        ];
        let r = RichText::<()>::new(&Theme::light(), &blocks)
            .spacing(3.0)
            .code_padding(4.0)
            .code_spacing(5.0)
            .quote_indent(6.0)
            .marker_width(7.0)
            .marker_spacing(8.0);
        assert_eq!((r.spacing, r.marker_width), (Some(3.0), Some(7.0)));
        let _: Element<'_, ()> = r.into();
    }
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct RichTextWidgetProps {
    pub line: RichTextLineProps,
    pub image_container: crate::components::ContainerProps,
    pub quote: crate::components::ContainerProps,
    pub quote_bar: crate::components::ContainerProps,
    pub language: crate::components::TextProps,
    pub code: crate::components::TextProps,
    pub code_container: crate::components::ContainerProps,
    pub list_marker: crate::components::TextProps,
    pub marker_container: crate::components::ContainerProps,
    pub table_cell: crate::components::ContainerProps,
    pub definition_detail: crate::components::ContainerProps,
    pub footnote_marker_container: crate::components::ContainerProps,
    pub footnote_marker: crate::components::TextProps,
}

/// Native presentation supported by iced's rich text (no link handlers or document state).
/// Span size, font and color overrides are applied after document typography as well.
#[derive(Clone, Default)]
pub struct RichTextLineProps {
    pub size: Option<iced::Pixels>,
    pub font: Option<Font>,
    pub color: Option<Color>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub align_x: Option<iced::widget::text::Alignment>,
    pub align_y: Option<iced::alignment::Vertical>,
    pub line_height: Option<iced::widget::text::LineHeight>,
    pub wrapping: Option<iced::widget::text::Wrapping>,
    pub style: Option<crate::components::props::TextStyle>,
}
impl RichTextLineProps {
    pub fn size(mut self, value: impl Into<iced::Pixels>) -> Self {
        self.size = Some(value.into());
        self
    }
    pub fn font(mut self, value: impl Into<Font>) -> Self {
        self.font = Some(value.into());
        self
    }
    pub fn color(mut self, value: impl Into<Color>) -> Self {
        self.color = Some(value.into());
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.width = Some(value.into());
        self
    }
    pub fn height(mut self, value: impl Into<Length>) -> Self {
        self.height = Some(value.into());
        self
    }
    pub fn align_x(mut self, value: impl Into<iced::widget::text::Alignment>) -> Self {
        self.align_x = Some(value.into());
        self
    }
    pub fn align_y(mut self, value: impl Into<iced::alignment::Vertical>) -> Self {
        self.align_y = Some(value.into());
        self
    }
    pub fn line_height(mut self, value: impl Into<iced::widget::text::LineHeight>) -> Self {
        self.line_height = Some(value.into());
        self
    }
    pub fn wrapping(mut self, value: impl Into<iced::widget::text::Wrapping>) -> Self {
        self.wrapping = Some(value.into());
        self
    }
    pub fn style(
        mut self,
        style: impl Fn(&Theme) -> iced::widget::text::Style + Send + Sync + 'static,
    ) -> Self {
        self.style = Some(std::sync::Arc::new(style));
        self
    }

    pub fn apply<'a, M: 'a>(
        &self,
        mut widget: iced::widget::text::Rich<'a, String, M, Theme>,
    ) -> iced::widget::text::Rich<'a, String, M, Theme> {
        if let Some(v) = self.size {
            widget = widget.size(v);
        }
        if let Some(v) = self.font {
            widget = widget.font(v);
        }
        if let Some(v) = self.width {
            widget = widget.width(v);
        }
        if let Some(v) = self.height {
            widget = widget.height(v);
        }
        if let Some(v) = self.align_x {
            widget = widget.align_x(v);
        }
        if let Some(v) = self.align_y {
            widget = widget.align_y(v);
        }
        if let Some(v) = self.line_height {
            widget = widget.line_height(v);
        }
        if let Some(v) = self.wrapping {
            widget = widget.wrapping(v);
        }
        if let Some(style) = self.style.clone() {
            widget = widget.style(move |theme| style(theme));
        }
        if let Some(v) = self.color {
            widget = widget.color(v);
        }
        widget
    }
}

impl RichTextWidgetProps {
    pub fn line(mut self, props: RichTextLineProps) -> Self {
        self.line = props;
        self
    }
    pub fn image_container(mut self, props: crate::components::ContainerProps) -> Self {
        self.image_container = props;
        self
    }
    pub fn quote(mut self, props: crate::components::ContainerProps) -> Self {
        self.quote = props;
        self
    }
    pub fn quote_bar(mut self, props: crate::components::ContainerProps) -> Self {
        self.quote_bar = props;
        self
    }
    pub fn language(mut self, props: crate::components::TextProps) -> Self {
        self.language = props;
        self
    }
    pub fn code(mut self, props: crate::components::TextProps) -> Self {
        self.code = props;
        self
    }
    pub fn code_container(mut self, props: crate::components::ContainerProps) -> Self {
        self.code_container = props;
        self
    }
    pub fn list_marker(mut self, props: crate::components::TextProps) -> Self {
        self.list_marker = props;
        self
    }
    pub fn marker_container(mut self, props: crate::components::ContainerProps) -> Self {
        self.marker_container = props;
        self
    }
    pub fn table_cell(mut self, props: crate::components::ContainerProps) -> Self {
        self.table_cell = props;
        self
    }
    pub fn definition_detail(mut self, props: crate::components::ContainerProps) -> Self {
        self.definition_detail = props;
        self
    }
    pub fn footnote_marker_container(mut self, props: crate::components::ContainerProps) -> Self {
        self.footnote_marker_container = props;
        self
    }
    pub fn footnote_marker(mut self, props: crate::components::TextProps) -> Self {
        self.footnote_marker = props;
        self
    }
    pub fn apply<'a, M>(&self, widget: RichText<'a, M>) -> RichText<'a, M>
    where
        M: Clone + 'a,
    {
        widget.props(self.clone())
    }
}

#[cfg(test)]
mod native_props_tests {
    use super::*;
    use crate::widgets::synced_text::native_props_tests::{assert_override, text_props};
    #[test]
    fn defaults_preserve_layout_and_overrides_reach_native_children() {
        let blocks = [Block::CodeBlock {
            lang: Some("Rust".into()),
            code: "let x = 1;".into(),
        }];
        let theme = Theme::light();
        let make = || RichText::<()>::new(&theme, &blocks);
        assert_override(
            make().into(),
            make().props(RichTextWidgetProps::default()).into(),
            make()
                .props(RichTextWidgetProps::default().code(text_props()))
                .into(),
        );
    }
}
