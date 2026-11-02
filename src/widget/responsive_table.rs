//! `ResponsiveTable`: one dataset, two presentations. A table while the columns fit, a stack
//! of cards when they would not.
//!
//! A table with many columns usually ends up scrolling on the X axis, which hides the cells
//! that matter. Here the *container* (not the window) decides: when its width is below what
//! the columns need ([`Column::min_width`]) every row is rendered as a card instead, with a
//! custom layout if you give one ([`ResponsiveTable::card`]) and a label/value list otherwise.
//! Nothing is ever clipped or scrolled sideways.
//!
//! ```ignore
//! ResponsiveTable::new(&self.people)
//!     .column(Column::new("Name", |p: &Person| body(&p.name).into()).min_width(140.0))
//!     .column(Column::new("Role", |p: &Person| body(&p.role).into()).min_width(120.0))
//!     .column(Column::new("Email", |p: &Person| body(&p.email).into()).min_width(220.0))
//!     // optional: your own card layout
//!     .card(|p| column![heading(&p.name), caption(&p.email)].into())
//! ```
//!
//! `ResponsiveTable` is a [`MetaComponent`](super::MetaComponent) with two variants, a table
//! and cards; the choice is [`ResponsiveTable::when`]. Like [`responsive`](iced::widget::responsive),
//! it fills its container and scrolls vertically by itself; give it a bounded height.

use super::Element;
use super::meta_component::MetaComponent;
use crate::Metrics;
use crate::breakpoints::WidthClass;
use crate::primitives::When;
use iced::Length;
use iced::widget::{column, container, row, space};
use std::rc::Rc;

/// What the table is drawn as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presentation {
    Table,
    Cards,
}

/// How [`ResponsiveTable`] picks its [`Presentation`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// A table while the container is as wide as the columns need, cards otherwise. The default.
    #[default]
    Auto,
    /// Always a table.
    Table,
    /// Always cards.
    Cards,
    /// Cards while the container's width class is below this one, a table from it (still
    /// cards if the columns do not fit). Thresholds follow the user's `[breakpoints]`.
    CardsBelow(WidthClass),
}

type Cell<'a, T, M> = Rc<dyn Fn(&T) -> Element<'a, M> + 'a>;

/// One column: a header and how to draw a row's cell.
pub struct Column<'a, T, M> {
    header: String,
    cell: Cell<'a, T, M>,
    width: Length,
    min_width: f32,
}

impl<'a, T, M> Column<'a, T, M> {
    pub fn new(header: impl Into<String>, cell: impl Fn(&T) -> Element<'a, M> + 'a) -> Self {
        Self {
            header: header.into(),
            cell: Rc::new(cell),
            width: Length::Fill,
            min_width: 100.0,
        }
    }
    /// The column's share of the table width (default `Fill`).
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    /// The narrowest this column may be before the table gives way to cards (default 100).
    pub fn min_width(mut self, px: f32) -> Self {
        self.min_width = px;
        self
    }
}

/// Rows of `T` as a table or as cards, by the width of the container. See the
/// [module](self).
pub struct ResponsiveTable<'a, T, M> {
    rows: &'a [T],
    columns: Vec<Column<'a, T, M>>,
    card: Option<Cell<'a, T, M>>,
    mode: Mode,
    metrics: Metrics,
    height: Length,
}

impl<'a, T: 'a, M: 'a> ResponsiveTable<'a, T, M> {
    pub fn new(rows: &'a [T]) -> Self {
        Self {
            rows,
            columns: Vec::new(),
            card: None,
            mode: Mode::Auto,
            metrics: Metrics::desktop(),
            height: Length::Fill,
        }
    }
    pub fn column(mut self, c: Column<'a, T, M>) -> Self {
        self.columns.push(c);
        self
    }
    /// A custom layout for one row as a card. Without it, a card is the first column as the
    /// title above every other column as a label/value pair.
    pub fn card(mut self, build: impl Fn(&T) -> Element<'a, M> + 'a) -> Self {
        self.card = Some(Rc::new(build));
        self
    }
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// Height of the table (default `Fill`; the body scrolls inside it).
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    /// The width the columns need to sit side by side, with gaps and padding.
    pub fn needed_width(&self) -> f32 {
        let m = &self.metrics;
        let cols: f32 = self.columns.iter().map(|c| c.min_width).sum();
        let gaps = self.columns.len().saturating_sub(1) as f32 * m.row_spacing;
        cols + gaps + m.card_padding.x()
    }
}

fn dim_caption<'a, M: 'a>(s: &str) -> Element<'a, M> {
    super::typography::caption_heading(s.to_owned())
        .style(crate::style::text::dim)
        .into()
}

impl<'a, T: 'a, M: 'a> ResponsiveTable<'a, T, M> {
    /// The condition under which the table (rather than cards) is shown, per [`Mode`].
    pub fn when(&self) -> Option<When> {
        let needed = self.needed_width();
        match self.mode {
            Mode::Table => Some(When::always()),
            Mode::Cards => None,
            Mode::Auto => Some(When::min_width(needed)),
            Mode::CardsBelow(c) => Some(When::from_class(c).and_min_width(needed)),
        }
    }

    /// The meta component this table is: the table variant (if the mode allows it) first,
    /// cards as the fallback.
    pub fn into_meta(self) -> MetaComponent<'a, Presentation, M> {
        let (when, height) = (self.when(), self.height);
        let t = Rc::new(self);
        let mut meta = MetaComponent::new().height(height);
        if let Some(when) = when {
            let t = t.clone();
            meta = meta.variant(Presentation::Table, when, move |_| table_view(&t));
        }
        meta.variant(Presentation::Cards, When::always(), move |_| cards_view(&t))
    }
}

impl<'a, T: 'a, M: 'a> From<ResponsiveTable<'a, T, M>> for Element<'a, M> {
    fn from(t: ResponsiveTable<'a, T, M>) -> Self {
        t.into_meta().into()
    }
}

fn is_empty<T, M>(t: &ResponsiveTable<'_, T, M>) -> bool {
    t.rows.is_empty() || t.columns.is_empty()
}

fn table_view<'a, T: 'a, M: 'a>(t: &ResponsiveTable<'a, T, M>) -> Element<'a, M> {
    if is_empty(t) {
        return space().into();
    }
    let m = &t.metrics;
    let line = |cells: Vec<Element<'a, M>>| {
        row(cells
            .into_iter()
            .zip(&t.columns)
            .map(|(c, col)| container(c).width(col.width).into()))
        .spacing(m.row_spacing)
    };
    let header = line(t.columns.iter().map(|c| dim_caption(&c.header)).collect());
    let mut rows = t
        .rows
        .iter()
        .map(|item| line(t.columns.iter().map(|c| (c.cell)(item)).collect()));
    let mut card = super::Card::new(rows.next().expect("not empty"));
    for r in rows {
        card = card.push(r);
    }
    let body = column![
        container(header).padding(m.card_padding),
        super::scrolled_window(card).height(Length::Fill),
    ]
    .spacing(m.space_s);
    container(body).padding(m.page_padding).into()
}

fn cards_view<'a, T: 'a, M: 'a>(t: &ResponsiveTable<'a, T, M>) -> Element<'a, M> {
    if is_empty(t) {
        return space().into();
    }
    let m = &t.metrics;
    let cards = t.rows.iter().map(|item| {
        let inner = match &t.card {
            Some(build) => build(item),
            None => default_card(&t.columns, item, m),
        };
        super::Card::new(inner).into()
    });
    let body = super::scrolled_window(column(cards).spacing(m.space_m).width(Length::Fill))
        .height(Length::Fill);
    container(body).padding(m.page_padding).into()
}

/// The first column as a title, then a `label  value` line for each of the others.
fn default_card<'a, T, M: 'a>(
    columns: &[Column<'a, T, M>],
    item: &T,
    m: &Metrics,
) -> Element<'a, M> {
    let mut col = column![].spacing(m.space_s).width(Length::Fill);
    for (i, c) in columns.iter().enumerate() {
        if i == 0 {
            col = col.push((c.cell)(item));
        } else {
            col = col.push(
                row![
                    container(dim_caption(&c.header)).width(Length::FillPortion(2)),
                    container((c.cell)(item)).width(Length::FillPortion(3)),
                ]
                .spacing(m.row_spacing),
            );
        }
    }
    col.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breakpoints::Breakpoints;

    const B: Breakpoints = Breakpoints {
        sm: 576.0,
        md: 768.0,
        lg: 992.0,
        xl: 1200.0,
        xxl: 1400.0,
    };

    fn pick(mode: Mode, needed_cols: f32, width: f32) -> Presentation {
        let rows = [()];
        let t = ResponsiveTable::<(), ()>::new(&rows)
            .column(Column::new("a", |_| space().into()).min_width(needed_cols))
            .mode(mode);
        *t.into_meta().pick(width, &B).unwrap()
    }

    fn needed(cols: f32) -> f32 {
        let rows = [()];
        ResponsiveTable::<(), ()>::new(&rows)
            .column(Column::new("a", |_| space().into()).min_width(cols))
            .needed_width()
    }

    #[test]
    fn auto_follows_what_the_columns_need() {
        let n = needed(600.0);
        assert_eq!(pick(Mode::Auto, 600.0, n - 1.0), Presentation::Cards);
        assert_eq!(pick(Mode::Auto, 600.0, n), Presentation::Table);
    }

    #[test]
    fn forced_modes_ignore_the_width() {
        assert_eq!(pick(Mode::Table, 600.0, 100.0), Presentation::Table);
        assert_eq!(pick(Mode::Cards, 600.0, 2000.0), Presentation::Cards);
    }

    #[test]
    fn cards_below_a_class_but_never_a_clipped_table() {
        let m = Mode::CardsBelow(WidthClass::Lg);
        assert_eq!(pick(m, 300.0, 900.0), Presentation::Cards, "below lg");
        assert_eq!(pick(m, 300.0, 1000.0), Presentation::Table);
        assert_eq!(pick(m, 1200.0, 1000.0), Presentation::Cards, "does not fit");
    }

    #[test]
    fn needed_width_counts_columns_gaps_and_padding() {
        let rows = [()];
        let t = ResponsiveTable::<(), ()>::new(&rows)
            .column(Column::new("a", |_| space().into()).min_width(100.0))
            .column(Column::new("b", |_| space().into()).min_width(50.0));
        let m = Metrics::desktop();
        assert_eq!(t.needed_width(), 150.0 + m.row_spacing + m.card_padding.x());
    }
}
