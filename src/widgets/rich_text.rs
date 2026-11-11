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
    blocks: &'a [Block],
    look: Look,
    compact: bool,
    max_blocks: Option<usize>,
    on_link: Option<Rc<dyn Fn(String) -> M + 'a>>,
    on_task: Option<Rc<dyn Fn(usize, bool) -> M + 'a>>,
}

/// Where we are while drawing: footnote numbers, the task counter, and the callbacks.
struct Ctx<'a, M> {
    look: Look,
    gap: f32,
    order: Vec<String>,
    tasks: Cell<usize>,
    on_link: Option<Rc<dyn Fn(String) -> M + 'a>>,
    on_task: Option<Rc<dyn Fn(usize, bool) -> M + 'a>>,
}

impl<'a, M: Clone + 'a> RichText<'a, M> {
    pub fn new(theme: &Theme, blocks: &'a [Block]) -> Self {
        Self {
            blocks,
            look: Look::of(theme),
            compact: false,
            max_blocks: None,
            on_link: None,
            on_task: None,
        }
    }

    /// Smaller text and tighter spacing, for a card.
    pub fn compact(mut self, on: bool) -> Self {
        self.compact = on;
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
        let cx = Ctx {
            look: r.look.scaled(k),
            gap: if r.compact { 6.0 } else { 12.0 },
            order: rt::footnote_order(r.blocks),
            tasks: Cell::new(0),
            on_link: r.on_link,
            on_task: r.on_task,
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

fn one<'a>(cx_look: &Look, text: String, st: &Style) -> Span<'a, String, Font> {
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
            .padding([0.0, 3.0]);
    }
    if let Some(url) = &st.link {
        s = s.link(url.clone()).underline(true);
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
                None => out.push(one(&cx.look, t.clone(), st)),
                Some(sc) => {
                    let mapped = match sc {
                        Script::Super => rt::superscript(t),
                        Script::Sub => rt::subscript(t),
                    };
                    // A character with no script form is drawn small and dim instead.
                    out.push(match mapped {
                        Some(m) => one(&cx.look, m, st),
                        None => one(
                            &cx.look,
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
                out.push(one(&cx.look, format!("▣ {label}"), &s));
            }
            Inline::FootnoteRef(label) => {
                let n = cx
                    .order
                    .iter()
                    .position(|l| l == label)
                    .map_or(0, |p| p + 1);
                let mark = rt::superscript(&n.to_string()).unwrap_or_else(|| format!("[{n}]"));
                let s = with(&|s| s.color = Some(cx.look.accent));
                out.push(one(&cx.look, mark, &s));
            }
            Inline::LineBreak => out.push(one(&cx.look, "\n".into(), st)),
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
    r.into()
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
            [Inline::Image { url, .. }] if Path::new(url).is_file() => container(
                image(url.clone())
                    .width(Length::Shrink)
                    .height(Length::Shrink),
            )
            .max_width(420.0)
            .into(),
            _ => line(cx, content, body_style(cx)),
        },
        Block::Quote(inner) => {
            let bar = cx.look.bar;
            // The bar is a second layer over the text, so it is as tall as the text: a `Fill`
            // bar in a row would have no height to fill.
            stack![
                container(blocks(cx, inner, depth)).padding(Padding::ZERO.left(14.0)),
                container(space().width(3.0).height(Length::Fill)).style(move |_: &Theme| {
                    container::Style {
                        background: Some(Background::Color(bar)),
                        border: Border::default().rounded(2.0),
                        ..container::Style::default()
                    }
                }),
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
            let mut col = column![].spacing(4);
            if let Some(l) = lang.as_deref().filter(|l| !l.is_empty()) {
                col = col.push(text(l.to_string()).size(cx.look.caption).color(cx.look.dim));
            }
            col = col.push(
                text(code.trim_end().to_string())
                    .size(mono.size)
                    .font(font(&mono)),
            );
            let (bg, radius) = (cx.look.code_bg, cx.look.radius);
            container(col)
                .padding(10)
                .width(Length::Fill)
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(bg)),
                    border: Border::default().rounded(radius),
                    ..container::Style::default()
                })
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
            None => text(rt::marker(start, n, depth))
                .size(cx.look.body)
                .color(cx.look.dim)
                .into(),
        };
        let content = blocks(cx, &item.blocks, depth + 1);
        row![container(lead).width(24.0).align_x(Alignment::End), content]
            .spacing(8)
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
            container(line_aligned(cx, rt::cell(cells, c), st, h))
                .width(Length::FillPortion(1))
                .padding([4, 8])
                .into()
        }))
        .into()
    };
    let mut col = column![one_row(head, true), divider()].spacing(2);
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
        let mut col = column![line(cx, &d.term, term)].spacing(2);
        for detail in &d.details {
            col = col.push(
                container(line(cx, detail, body_style(cx))).padding(Padding::ZERO.left(16.0)),
            );
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
        look: cx.look.scaled(0.9),
        gap: cx.gap / 2.0,
        order: cx.order.clone(),
        tasks: Cell::new(0),
        on_link: cx.on_link.clone(),
        on_task: None,
    };
    let rows = sorted.into_iter().enumerate().map(|(i, n)| {
        let k = number(n);
        let label = if k == usize::MAX { i + 1 } else { k + 1 };
        row![
            container(
                text(format!("{label}."))
                    .size(small.look.body)
                    .color(small.look.dim)
            )
            .width(24.0)
            .align_x(Alignment::End),
            blocks(&small, &n.blocks, depth),
        ]
        .spacing(8)
        .into()
    });
    column![divider(), column(rows).spacing(cx.gap / 2.0)]
        .spacing(cx.gap / 2.0)
        .into()
}
