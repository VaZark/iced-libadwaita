//! The "Rich text" page: one document that uses every element `RichText` draws, as the data a
//! host app would build from Markdown (or anything else).

use super::*;
use iad::primitives::rich_text::{Align, Block, Definition, Footnote, Inline, Item};
use iced::widget::column;

fn t(s: &str) -> Vec<Inline> {
    vec![Inline::Text(s.into())]
}

fn text_in(parts: Vec<Inline>) -> Block {
    Block::Paragraph(parts)
}

fn para(s: &str) -> Block {
    Block::Paragraph(t(s))
}

fn item(s: &str) -> Item {
    Item {
        checked: None,
        blocks: vec![para(s)],
    }
}

fn task(s: &str, done: bool) -> Item {
    Item {
        checked: Some(done),
        blocks: vec![para(s)],
    }
}

/// Every element, in the order a reader meets them. `done` are the two task checkboxes.
pub fn sample(done: [bool; 2]) -> Vec<Block> {
    use Inline::*;
    vec![
        Block::Heading {
            level: 1,
            id: None,
            content: t("Heading 1"),
        },
        Block::Heading {
            level: 2,
            id: None,
            content: t("Heading 2"),
        },
        Block::Heading {
            level: 3,
            id: Some("custom-id".into()),
            content: t("Heading 3"),
        },
        text_in(vec![
            Text("Plain text with ".into()),
            Strong(t("bold")),
            Text(", ".into()),
            Emphasis(t("italic")),
            Text(", ".into()),
            Strong(vec![Emphasis(t("both"))]),
            Text(", ".into()),
            Strike(t("strike-through")),
            Text(" and ".into()),
            Highlight(t("highlighted")),
            Text(" text. Water is H".into()),
            Subscript(t("2")),
            Text("O and E = mc".into()),
            Superscript(t("2")),
            Text(". Type ".into()),
            Code("cargo run".into()),
            Text(", read ".into()),
            Link {
                content: t("the docs"),
                url: "https://example.org".into(),
            },
            Text(", and mind the footnote.".into()),
            FootnoteRef("note".into()),
        ]),
        Block::Quote(vec![
            para("A block quote carries a bar of the accent colour."),
            Block::Quote(vec![para("They nest.")]),
        ]),
        Block::List {
            start: None,
            items: vec![
                item("Bullets"),
                Item {
                    checked: None,
                    blocks: vec![
                        para("Nest to change the marker"),
                        Block::List {
                            start: None,
                            items: vec![item("Second level"), item("Another")],
                        },
                    ],
                },
            ],
        },
        Block::List {
            start: Some(1),
            items: vec![
                item("Numbered"),
                item("Counting up"),
                item("From any start"),
            ],
        },
        Block::List {
            start: None,
            items: vec![
                task("Task lists have checkboxes", done[0]),
                task("Tick one", done[1]),
            ],
        },
        Block::Rule,
        Block::CodeBlock {
            lang: Some("rust".into()),
            code: "fn main() {\n    println!(\"Hello, rich text\");\n}".into(),
        },
        Block::Table {
            align: vec![Align::Left, Align::Center, Align::Right],
            head: vec![t("Left"), t("Centre"), t("Right")],
            rows: vec![
                vec![t("Header"), t("Title"), t("1")],
                vec![t("Paragraph"), t("Text"), t("22")],
            ],
        },
        Block::Definitions(vec![Definition {
            term: t("Term"),
            details: vec![t("What it means, indented under the term.")],
        }]),
        Block::Footnotes(vec![Footnote {
            label: "note".into(),
            blocks: vec![para(
                "Footnotes are numbered by first reference and drawn last.",
            )],
        }]),
    ]
}

impl Gallery {
    pub(super) fn rich_text_page(&self) -> Element<'_> {
        let m = self.metrics();
        let theme = self.build_theme();
        let full: Element<'_> = w::RichText::new(&theme, &self.rich_doc)
            .on_link(|_| Message::Noop)
            .on_task(Message::RichTask)
            .into();
        // The same document as a card shows it: smaller, and only the first few blocks.
        let compact: Element<'_> = w::RichText::new(&theme, &self.rich_doc)
            .compact(true)
            .max_blocks(4)
            .into();
        self.page_frame(
            column![
                self.group("Every element").push(w::card(full)),
                self.group("Compact, first blocks only").push(w::card(compact)),
                playground::code_block(
                    m,
                    "let doc = vec![Block::Heading { level: 1, id: None, content: vec![Inline::Text(\"Title\".into())] }, /* … */];\n\nRichText::new(&theme, &doc)\n    .on_link(Msg::OpenUrl)               // a link was pressed\n    .on_task(|n, on| Msg::Task(n, on))   // task n (reading order) was toggled\n    .compact(true).max_blocks(4)         // for a card"
                        .to_string(),
                ),
            ]
            .spacing(m.group_spacing),
        )
    }
}
