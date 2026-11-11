//! Headless rich text: formatted prose as plain data, with no look and no parsing. The shapes
//! are the ones prose is made of (headings, quotes, lists, task lists, code, tables, footnotes,
//! definition lists, emphasis, highlight, sub- and superscript), so a widget can draw them and
//! an app can build them from whatever it stores (Markdown, HTML, a database).
//!
//! The rules a renderer would otherwise re-derive live here: the marker of a list item
//! ([`marker`]), the number a footnote gets ([`footnote_order`]), a sub- or superscript as
//! Unicode ([`superscript`], [`subscript`]), and the columns of a ragged table
//! ([`columns`], [`cell`]).
//!
//! ```
//! use iced_adaptive_design::primitives::rich_text::*;
//!
//! let doc = vec![Block::Paragraph(vec![
//!     Inline::Text("Water is H".into()),
//!     Inline::Subscript(vec![Inline::Text("2".into())]),
//!     Inline::Text("O".into()),
//! ])];
//! assert_eq!(plain(&doc), "Water is H2O");
//! assert_eq!(subscript("2").as_deref(), Some("₂"));
//! ```

/// How a table column aligns its cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Text inside a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    /// Code in a line (monospace).
    Code(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strike(Vec<Inline>),
    /// Marked text, as with a highlighter.
    Highlight(Vec<Inline>),
    Subscript(Vec<Inline>),
    Superscript(Vec<Inline>),
    Link {
        content: Vec<Inline>,
        url: String,
    },
    Image {
        alt: String,
        url: String,
    },
    /// A reference to the footnote with this label.
    FootnoteRef(String),
    /// A forced line break inside a paragraph.
    LineBreak,
}

/// One list entry. `checked` is `Some` for a task-list item and `None` for a plain one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Item {
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

/// A term and what it means (one or more details).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub term: Vec<Inline>,
    pub details: Vec<Vec<Inline>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Footnote {
    pub label: String,
    pub blocks: Vec<Block>,
}

/// A part of the page of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// `level` is 1 (largest) to 6; the id is an anchor name, if the author gave one.
    Heading {
        level: u8,
        id: Option<String>,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Quote(Vec<Block>),
    /// `start` is `Some(n)` for a numbered list that counts from `n`, `None` for bullets.
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    Rule,
    Table {
        align: Vec<Align>,
        head: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    Definitions(Vec<Definition>),
    /// The notes the document's [`Inline::FootnoteRef`]s point to, drawn at its end.
    Footnotes(Vec<Footnote>),
}

// ---- text ----------------------------------------------------------------------------------

/// The characters of `inlines`, without any formatting.
pub fn plain_inlines(inlines: &[Inline]) -> String {
    let mut out = String::new();
    for i in inlines {
        match i {
            Inline::Text(t) | Inline::Code(t) => out.push_str(t),
            Inline::Emphasis(c)
            | Inline::Strong(c)
            | Inline::Strike(c)
            | Inline::Highlight(c)
            | Inline::Subscript(c)
            | Inline::Superscript(c)
            | Inline::Link { content: c, .. } => out.push_str(&plain_inlines(c)),
            Inline::Image { alt, .. } => out.push_str(alt),
            Inline::FootnoteRef(_) => {}
            Inline::LineBreak => out.push('\n'),
        }
    }
    out
}

/// The characters of a document, without formatting: blocks on their own lines. For search,
/// previews and accessibility labels.
pub fn plain(blocks: &[Block]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for b in blocks {
        match b {
            Block::Heading { content, .. } | Block::Paragraph(content) => {
                lines.push(plain_inlines(content));
            }
            Block::Quote(inner) => lines.push(plain(inner)),
            Block::List { items, .. } => {
                for i in items {
                    lines.push(plain(&i.blocks));
                }
            }
            Block::CodeBlock { code, .. } => lines.push(code.trim_end().to_string()),
            Block::Rule => {}
            Block::Table { head, rows, .. } => {
                for r in std::iter::once(head).chain(rows) {
                    lines.push(
                        r.iter()
                            .map(|c| plain_inlines(c))
                            .collect::<Vec<_>>()
                            .join(" "),
                    );
                }
            }
            Block::Definitions(defs) => {
                for d in defs {
                    lines.push(plain_inlines(&d.term));
                    lines.extend(d.details.iter().map(|x| plain_inlines(x)));
                }
            }
            Block::Footnotes(notes) => {
                for n in notes {
                    lines.push(plain(&n.blocks));
                }
            }
        }
    }
    lines.retain(|l| !l.is_empty());
    lines.join("\n")
}

// ---- lists ---------------------------------------------------------------------------------

/// The marker before list item number `index` (from 0) at nesting `depth` (0 is the outermost):
/// `1.`, `2.` … counting from `start` for a numbered list, and `•`, `◦`, `▪` going inwards for
/// bullets.
pub fn marker(start: Option<u64>, index: usize, depth: usize) -> String {
    match start {
        Some(n) => format!("{}.", n + index as u64),
        None => ["•", "◦", "▪"][depth % 3].to_string(),
    }
}

/// How many task-list items (checkbox items) the blocks hold, nested ones included. Tasks are
/// numbered `0..` in reading order, which is how a toggle says which one it means.
pub fn task_count(blocks: &[Block]) -> usize {
    blocks
        .iter()
        .map(|b| match b {
            Block::List { items, .. } => items
                .iter()
                .map(|i| usize::from(i.checked.is_some()) + task_count(&i.blocks))
                .sum(),
            Block::Quote(inner) => task_count(inner),
            _ => 0,
        })
        .sum()
}

// ---- footnotes -----------------------------------------------------------------------------

fn refs_in(inlines: &[Inline], out: &mut Vec<String>) {
    for i in inlines {
        match i {
            Inline::FootnoteRef(l) => {
                if !out.contains(l) {
                    out.push(l.clone());
                }
            }
            Inline::Emphasis(c)
            | Inline::Strong(c)
            | Inline::Strike(c)
            | Inline::Highlight(c)
            | Inline::Subscript(c)
            | Inline::Superscript(c)
            | Inline::Link { content: c, .. } => refs_in(c, out),
            _ => {}
        }
    }
}

fn refs_in_blocks(blocks: &[Block], out: &mut Vec<String>) {
    for b in blocks {
        match b {
            Block::Heading { content, .. } | Block::Paragraph(content) => refs_in(content, out),
            Block::Quote(inner) => refs_in_blocks(inner, out),
            Block::List { items, .. } => items.iter().for_each(|i| refs_in_blocks(&i.blocks, out)),
            Block::Table { head, rows, .. } => {
                head.iter().for_each(|c| refs_in(c, out));
                rows.iter().flatten().for_each(|c| refs_in(c, out));
            }
            Block::Definitions(defs) => {
                for d in defs {
                    refs_in(&d.term, out);
                    d.details.iter().for_each(|x| refs_in(x, out));
                }
            }
            Block::CodeBlock { .. } | Block::Rule | Block::Footnotes(_) => {}
        }
    }
}

/// The labels of the footnotes the text refers to, in the order they are first referenced. A
/// label's number is its place in this list plus one, whatever the author named it.
pub fn footnote_order(blocks: &[Block]) -> Vec<String> {
    let mut out = Vec::new();
    refs_in_blocks(blocks, &mut out);
    out
}

// ---- sub- and superscript ------------------------------------------------------------------

const SUPER: &str = "0123456789+-=()abcdefghijklmnoprstuvwxyz";
const SUPER_TO: &str = "⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾ᵃᵇᶜᵈᵉᶠᵍʰⁱʲᵏˡᵐⁿᵒᵖʳˢᵗᵘᵛʷˣʸᶻ";
const SUB: &str = "0123456789+-=()aehijklmnoprstuvx";
const SUB_TO: &str = "₀₁₂₃₄₅₆₇₈₉₊₋₌₍₎ₐₑₕᵢⱼₖₗₘₙₒₚᵣₛₜᵤᵥₓ";

fn map(s: &str, from: &str, to: &str) -> Option<String> {
    let to: Vec<char> = to.chars().collect();
    s.chars()
        .map(|c| {
            let c = c.to_ascii_lowercase();
            from.chars().position(|f| f == c).map(|i| to[i])
        })
        .collect()
}

/// `s` as superscript characters, or `None` if one of them has no such form (a renderer then
/// draws it small instead).
pub fn superscript(s: &str) -> Option<String> {
    map(s, SUPER, SUPER_TO)
}

/// `s` as subscript characters, or `None` if one of them has no such form.
pub fn subscript(s: &str) -> Option<String> {
    map(s, SUB, SUB_TO)
}

// ---- tables --------------------------------------------------------------------------------

/// How many columns a table has: the widest of its header and rows.
pub fn columns(head: &[Vec<Inline>], rows: &[Vec<Vec<Inline>>]) -> usize {
    rows.iter()
        .map(Vec::len)
        .chain([head.len()])
        .max()
        .unwrap_or(0)
}

const NO_CELL: &[Inline] = &[];

/// The cell at `col` of a row, empty when the row is short.
pub fn cell(row: &[Vec<Inline>], col: usize) -> &[Inline] {
    row.get(col).map_or(NO_CELL, Vec::as_slice)
}

/// A column's alignment; columns the author gave none are left-aligned.
pub fn align(aligns: &[Align], col: usize) -> Align {
    aligns.get(col).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Vec<Inline> {
        vec![Inline::Text(s.into())]
    }

    #[test]
    fn plain_text_drops_formatting_and_keeps_alt_text() {
        let doc = vec![
            Block::Heading {
                level: 1,
                id: None,
                content: t("Title"),
            },
            Block::Paragraph(vec![
                Inline::Strong(t("bold")),
                Inline::Text(" and ".into()),
                Inline::Image {
                    alt: "logo".into(),
                    url: "a.png".into(),
                },
                Inline::FootnoteRef("1".into()),
            ]),
            Block::Rule,
        ];
        assert_eq!(plain(&doc), "Title\nbold and logo");
    }

    #[test]
    fn markers_count_from_the_start_and_bullets_change_with_depth() {
        assert_eq!(marker(Some(3), 0, 0), "3.");
        assert_eq!(marker(Some(3), 2, 4), "5.");
        assert_eq!(marker(None, 0, 0), "•");
        assert_eq!(marker(None, 7, 1), "◦");
        assert_eq!(marker(None, 0, 3), "•");
    }

    #[test]
    fn tasks_are_counted_through_nesting_and_quotes() {
        let inner = Block::List {
            start: None,
            items: vec![Item {
                checked: Some(true),
                blocks: vec![],
            }],
        };
        let doc = vec![
            Block::List {
                start: None,
                items: vec![
                    Item {
                        checked: Some(false),
                        blocks: vec![inner.clone()],
                    },
                    Item {
                        checked: None,
                        blocks: vec![],
                    },
                ],
            },
            Block::Quote(vec![inner]),
        ];
        assert_eq!(task_count(&doc), 3);
    }

    #[test]
    fn footnotes_are_numbered_by_first_reference_not_by_label() {
        let doc = vec![
            Block::Paragraph(vec![
                Inline::FootnoteRef("b".into()),
                Inline::Emphasis(vec![Inline::FootnoteRef("a".into())]),
                Inline::FootnoteRef("b".into()),
            ]),
            Block::Quote(vec![Block::Paragraph(vec![Inline::FootnoteRef(
                "c".into(),
            )])]),
            Block::Footnotes(vec![Footnote {
                label: "z".into(),
                blocks: vec![],
            }]),
        ];
        assert_eq!(footnote_order(&doc), ["b", "a", "c"]);
    }

    #[test]
    fn script_forms_exist_for_digits_and_common_letters_only() {
        assert_eq!(superscript("2").as_deref(), Some("²"));
        assert_eq!(superscript("TH").as_deref(), Some("ᵗʰ"));
        assert_eq!(subscript("10").as_deref(), Some("₁₀"));
        assert_eq!(superscript("q"), None);
        assert_eq!(subscript("b"), None);
        assert_eq!(superscript(""), Some(String::new()));
    }

    #[test]
    fn every_mapped_character_has_a_form() {
        assert_eq!(SUPER.chars().count(), SUPER_TO.chars().count());
        assert_eq!(SUB.chars().count(), SUB_TO.chars().count());
    }

    #[test]
    fn ragged_tables_read_as_the_widest_row_with_empty_cells() {
        let head = vec![t("a"), t("b")];
        let rows = vec![vec![t("1")], vec![t("1"), t("2"), t("3")]];
        assert_eq!(columns(&head, &rows), 3);
        assert!(cell(&rows[0], 2).is_empty());
        assert_eq!(cell(&rows[1], 2), t("3").as_slice());
        assert_eq!(align(&[Align::Right], 0), Align::Right);
        assert_eq!(align(&[Align::Right], 5), Align::Left);
    }
}
