//! The token page, virtualized.
//!
//! The "All" page has around 300 rows. Building and laying out every one on each change made
//! scrolling and resizing stutter, so only the blocks in or near view are built; the others are
//! spacers of the same height, which keeps the scroll bar true to the whole page.
//!
//! A block is a group (its heading and its card of rows). Heights cannot be measured without
//! building, so they are computed: every row is one line, so a group is a constant plus the sum of
//! its rows, and the gradient and breakpoint-rule editors add a height per stop or rule. The
//! constants are the layout at the default metrics; `tests::block_heights_match_the_real_layout`
//! lays every group out and fails if one drifts.

use crate::tokens::ALL_SECTIONS;
use crate::{Editor, Msg};
use iad::layouts::virtualize::Viewport;
use iad::theme_config::{Gradient, Kind, Spec};
use iad::widgets as w;
use iad::{Element, Icon};
use iced::Length;
use iced::widget::{column, container, space};
use iced_adaptive_design as iad;

/// The narrowest window: the heights below hold from this width.
pub(crate) const MIN_WIDTH: f32 = 560.0;
/// A token row: a one-line title and description, with room above the natural height so a larger
/// font does not overflow it.
pub(crate) const ROW: f32 = 56.0;
/// What a gradient adds to its row (the preview and the angle), then each stop.
pub(crate) const GRADIENT: f32 = 56.0;
pub(crate) const STOP: f32 = 32.0;
/// What each breakpoint rule adds to its row.
pub(crate) const RULE: f32 = 32.0;
/// The line between two rows of a card, and what the card adds around them.
pub(crate) const SEPARATOR: f32 = 1.0;
pub(crate) const CARD: f32 = 0.0;
/// Pixels built beyond each edge of the view, so a quick scroll shows no gap.
const OVERSCAN: f32 = 600.0;
/// The viewport height assumed before the first scroll event.
pub(crate) const FIRST_VIEW: f32 = 1400.0;

/// A group of tokens on the page and how tall it is.
pub(crate) struct Block<'a> {
    pub section: &'a str,
    pub title: String,
    pub specs: Vec<&'a Spec>,
    pub height: f32,
}

impl Editor {
    /// How tall the row of `spec` is with the values as they are.
    pub(crate) fn row_height(&self, spec: &Spec) -> f32 {
        let (raw, _) = self.shown(spec);
        if spec.ty.kind() == Kind::Gradient {
            return Gradient::parse(&raw)
                .map_or(ROW, |g| ROW + GRADIENT + STOP * g.stops.len() as f32);
        }
        if spec.key == "breakpoint-rules" {
            let rules = raw.split(',').filter(|s| !s.trim().is_empty()).count();
            return ROW + RULE * rules as f32;
        }
        ROW
    }

    /// How tall a group is: its heading, then (unless folded) the card of its rows.
    pub(crate) fn group_height(&self, section: &str, title: &str, specs: &[&Spec]) -> f32 {
        let m = iad::Metrics::desktop();
        if self.is_folded(section, title) {
            return m.button_min_height;
        }
        let rows: f32 = specs.iter().map(|s| self.row_height(s)).sum();
        m.button_min_height
            + m.space_m
            + rows
            + SEPARATOR * specs.len().saturating_sub(1) as f32
            + CARD
    }

    /// The groups the page shows, in order, each with its height.
    pub(crate) fn blocks(&self) -> Vec<Block<'_>> {
        let mut blocks = Vec::new();
        for section in self.page_sections() {
            for (title, specs) in self.groups_of(section) {
                let specs: Vec<&Spec> = specs.into_iter().filter(|s| self.visible(s)).collect();
                if specs.is_empty() {
                    continue;
                }
                let height = self.group_height(section, &title, &specs);
                blocks.push(Block {
                    section,
                    title,
                    specs,
                    height,
                });
            }
        }
        blocks
    }

    /// The blocks to build: those in or within [`OVERSCAN`] of the viewport.
    pub(crate) fn built(&self, blocks: &[Block<'_>], gap: f32) -> std::ops::Range<usize> {
        let top = self.tokens_view.offset - OVERSCAN;
        let bottom = self.tokens_view.offset + self.tokens_view.height + OVERSCAN;
        let (mut y, mut first, mut last) = (0.0, None, 0);
        for (i, b) in blocks.iter().enumerate() {
            let end = y + b.height;
            if end >= top && y <= bottom {
                first.get_or_insert(i);
                last = i + 1;
            }
            y = end + gap;
        }
        first.map_or(0..0, |f| f..last)
    }

    /// The page: the tokens of the current section, or of every section when searching. Only the
    /// blocks near the viewport are built.
    pub(crate) fn page(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let blocks = self.blocks();
        if blocks.is_empty() {
            return w::status_page()
                .icon(Icon::Search.view(48.0))
                .title("No tokens match")
                .description(if self.overrides_only {
                    "No explicit overrides match in this scheme. Turn off “Overrides only” to include inherited base values."
                } else if self.modified_only {
                    "This file sets nothing that matches. Turn off “Modified only” to see every token."
                } else {
                    "Try another word: the search looks at names, descriptions and sections."
                })
                .into();
        }
        let searching = self.effective_section() == ALL_SECTIONS;
        let built = self.built(&blocks, m.group_spacing);
        let mut page = column![].spacing(m.group_spacing);
        for (i, b) in blocks.iter().enumerate() {
            page = page.push(if built.contains(&i) {
                container(self.group(b.section, &b.title, b.specs.clone(), searching, m))
                    .width(Length::Fill)
                    .height(Length::Fixed(b.height))
                    .into()
            } else {
                Element::from(space().height(b.height))
            });
        }
        page.into()
    }

    /// The scroll position the page was last given: what [`Editor::built`] goes by.
    pub(crate) fn scrolled(&mut self, offset: f32, height: f32) {
        self.tokens_view.set(offset, height);
    }

    /// Back to the top, for a page that is another page now.
    pub(crate) fn scroll_to_top(&mut self) {
        self.tokens_view = Viewport {
            offset: 0.0,
            ..self.tokens_view
        };
    }
}
