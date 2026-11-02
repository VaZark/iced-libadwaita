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
//! **Sorting.** Mark columns with [`Column::sortable`]`(key)`, pass the app's
//! [`SortState`](crate::primitives::sorting::SortState) to [`ResponsiveTable::sort`] and handle
//! [`ResponsiveTable::on_sort`]`(key)` by calling `state.activate(key, false)` (ascending,
//! descending, unsorted; see [`primitives::sorting`](crate::primitives::sorting)) and sorting the
//! rows with `state.apply`. In the table the header cell is a button with an arrow (none, up,
//! down; a number when several columns sort); the cards presentation has no header row, so the
//! same events come from a **sort control above the cards**: a dropdown of the sortable columns
//! and a direction button. Both send the same `on_sort(key)`, so the app has one code path.
//! Sorting the data and paging it are the app's job; [`ResponsiveTable::footer`] is where a
//! [`Pagination`](super::Pagination) goes.
//!
//! `ResponsiveTable` is a [`MetaComponent`](super::MetaComponent) with two variants, a table
//! and cards; the choice is [`ResponsiveTable::when`]. Like [`responsive`](iced::widget::responsive),
//! it fills its container and scrolls vertically by itself; give it a bounded height.

use super::Element;
use super::meta_component::MetaComponent;
use crate::breakpoints::WidthClass;
use crate::primitives::When;
use crate::primitives::sorting::{Direction, SortState};
use crate::style::button as sb;
use crate::{Icon, Metrics, Theme};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space, text};
use iced::{Length, Padding};
use std::any::{Any, TypeId};
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
    sort_key: Option<Rc<dyn Any>>,
}

impl<'a, T, M> Column<'a, T, M> {
    pub fn new(header: impl Into<String>, cell: impl Fn(&T) -> Element<'a, M> + 'a) -> Self {
        Self {
            header: header.into(),
            cell: Rc::new(cell),
            width: Length::Fill,
            min_width: 100.0,
            sort_key: None,
        }
    }
    /// Makes the column sortable under `key` (any type; the table's [`sort`](ResponsiveTable::sort)
    /// and [`on_sort`](ResponsiveTable::on_sort) must use the same type).
    pub fn sortable<K: Any>(mut self, key: K) -> Self {
        self.sort_key = Some(Rc::new(key));
        self
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

/// How the table reads the app's sort state and reports a header activation, type-erased so
/// that `ResponsiveTable` does not need the key type as a parameter.
type DirOf<'a> = Rc<dyn Fn(&dyn Any) -> Option<(Direction, Option<usize>)> + 'a>;
type Fire<'a, M> = Rc<dyn Fn(&dyn Any) -> M + 'a>;
type Footer<'a, M> = Rc<dyn Fn() -> Element<'a, M> + 'a>;

/// Rows of `T` as a table or as cards, by the width of the container. See the
/// [module](self).
pub struct ResponsiveTable<'a, T, M> {
    rows: &'a [T],
    columns: Vec<Column<'a, T, M>>,
    card: Option<Cell<'a, T, M>>,
    mode: Mode,
    metrics: Metrics,
    height: Length,
    sort: Option<(TypeId, DirOf<'a>)>,
    on_sort: Option<(TypeId, Fire<'a, M>)>,
    footer: Option<Footer<'a, M>>,
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
            sort: None,
            on_sort: None,
            footer: None,
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

    /// The sort state to show (arrows on the headers, the cards' sort control). The table does
    /// not sort anything: sort the rows you pass in with [`SortState::apply`].
    pub fn sort<K: Clone + PartialEq + 'static>(mut self, state: &'a SortState<K>) -> Self {
        self.sort = Some((
            TypeId::of::<K>(),
            Rc::new(move |any| {
                let key = any.downcast_ref::<K>()?;
                state.direction(key).map(|d| (d, state.priority(key)))
            }),
        ));
        self
    }
    /// A sortable column was activated (its header, or its entry in the cards' sort control;
    /// the direction button sends the current column). Call `state.activate(key, false)`.
    pub fn on_sort<K: Clone + 'static>(mut self, f: impl Fn(K) -> M + 'a) -> Self {
        self.on_sort = Some((
            TypeId::of::<K>(),
            Rc::new(move |any| f(any.downcast_ref::<K>().expect("checked by type id").clone())),
        ));
        self
    }
    /// Content below the rows, outside the scrolling part: a [`Pagination`](super::Pagination)
    /// bar, a count. A closure, since the table is built once per presentation.
    pub fn footer(mut self, build: impl Fn() -> Element<'a, M> + 'a) -> Self {
        self.footer = Some(Rc::new(build));
        self
    }

    /// A column's sort key and everything a header needs, if it is sortable here: the message
    /// for activating it, its direction and its priority (with several sort keys).
    fn sorting(&self, col: &Column<'a, T, M>) -> Option<ColumnSort<M>> {
        let key = col.sort_key.as_deref()?;
        let (id, fire) = self.on_sort.as_ref()?;
        if (*key).type_id() != *id {
            return None;
        }
        let dir = self
            .sort
            .as_ref()
            .filter(|(sid, _)| sid == id)
            .and_then(|(_, f)| f(key));
        Some(ColumnSort {
            press: fire(key),
            dir: dir.map(|d| d.0),
            priority: dir.and_then(|d| d.1),
        })
    }

    /// The width the columns need to sit side by side, with gaps and padding.
    pub fn needed_width(&self) -> f32 {
        let m = &self.metrics;
        let cols: f32 = self.columns.iter().map(|c| c.min_width).sum();
        let gaps = self.columns.len().saturating_sub(1) as f32 * m.row_spacing;
        cols + gaps + m.card_padding.x()
    }
}

struct ColumnSort<M> {
    press: M,
    dir: Option<Direction>,
    priority: Option<usize>,
}

fn dim_caption<'a, M: 'a>(s: &str) -> Element<'a, M> {
    super::typography::caption_heading(s.to_owned())
        .style(crate::style::text::dim)
        .into()
}

impl<'a, T: 'a, M: Clone + 'a> ResponsiveTable<'a, T, M> {
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

impl<'a, T: 'a, M: Clone + 'a> From<ResponsiveTable<'a, T, M>> for Element<'a, M> {
    fn from(t: ResponsiveTable<'a, T, M>) -> Self {
        t.into_meta().into()
    }
}

fn is_empty<T, M>(t: &ResponsiveTable<'_, T, M>) -> bool {
    t.rows.is_empty() || t.columns.is_empty()
}

fn table_view<'a, T: 'a, M: Clone + 'a>(t: &ResponsiveTable<'a, T, M>) -> Element<'a, M> {
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
    let header = line(t.columns.iter().map(|c| header_cell(t, c)).collect());
    let mut rows = t
        .rows
        .iter()
        .map(|item| line(t.columns.iter().map(|c| (c.cell)(item)).collect()));
    let mut card = super::Card::new(rows.next().expect("not empty"));
    for r in rows {
        card = card.push(r);
    }
    let mut body = column![
        container(header).padding(m.card_padding),
        super::scrolled_window(card).height(Length::Fill),
    ]
    .spacing(m.space_s);
    if let Some(footer) = &t.footer {
        body = body.push(footer());
    }
    container(body).padding(m.page_padding).into()
}

/// A column header: dim caption text, or for a sortable column a button with the arrow.
fn header_cell<'a, T: 'a, M: Clone + 'a>(
    t: &ResponsiveTable<'a, T, M>,
    col: &Column<'a, T, M>,
) -> Element<'a, M> {
    let Some(sort) = t.sorting(col) else {
        return dim_caption(&col.header);
    };
    let m = &t.metrics;
    let sorted = sort.dir.is_some();
    let arrow = match sort.dir {
        Some(Direction::Asc) => Icon::GoUp,
        Some(Direction::Desc) => Icon::GoDown,
        None => Icon::GoUpDown,
    };
    // SVG tints ignore alpha: a solid foreground when sorted, a solid dim mix otherwise.
    let icon =
        arrow
            .view(m.icon_size * 0.75)
            .style(move |th: &Theme, _| iced::widget::svg::Style {
                color: Some(if sorted {
                    th.semantic.icon_solid(false)
                } else {
                    th.semantic.dim_solid()
                }),
            });
    let mut content = row![super::typography::caption_heading(col.header.clone()), icon]
        .spacing(m.space_xs)
        .align_y(Vertical::Center);
    if let Some(n) = sort.priority {
        content = content.push(super::typography::caption(n.to_string()));
    }
    // No padding: the label stays on the column's left edge, aligned with the cells.
    button(content)
        .padding(Padding::ZERO)
        .style(move |th: &Theme, status| {
            let hot = sorted || matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                text_color: if hot {
                    th.semantic.window.fg
                } else {
                    th.semantic.dim()
                },
                ..button::Style::default()
            }
        })
        .on_press(sort.press)
        .into()
}

/// The cards' stand-in for the header row: a dropdown of the sortable columns and a button
/// that cycles the direction of the current one. `None` if nothing is sortable.
fn sort_control<'a, T: 'a, M: Clone + 'a>(t: &ResponsiveTable<'a, T, M>) -> Option<Element<'a, M>> {
    let m = &t.metrics;
    let cols: Vec<(&Column<'a, T, M>, ColumnSort<M>)> = t
        .columns
        .iter()
        .filter_map(|c| t.sorting(c).map(|s| (c, s)))
        .collect();
    if cols.is_empty() {
        return None;
    }
    let current = cols
        .iter()
        .find(|(_, s)| s.dir.is_some() && s.priority.is_none_or(|p| p == 1));
    let side = m.button_min_height;
    let items = cols
        .iter()
        .map(|(c, s)| super::menu_item(c.header.clone(), s.dir.is_some(), s.press.clone()));
    let trigger = container(
        row![
            text(
                current
                    .map_or("None", |(c, _)| c.header.as_str())
                    .to_owned()
            )
            .wrapping(text::Wrapping::None),
            Icon::GoDown.view(m.icon_size),
        ]
        .spacing(m.space_s)
        .align_y(Vertical::Center),
    )
    .center_y(Length::Fixed(side - m.button_padding.y()));
    let picker = super::Dropdown::custom(trigger, super::popover(items, 280.0));

    let (arrow, press) = match current {
        Some((_, s)) => (
            if s.dir == Some(Direction::Desc) {
                Icon::GoDown
            } else {
                Icon::GoUp
            },
            Some(s.press.clone()),
        ),
        None => (Icon::GoUpDown, None),
    };
    let enabled = press.is_some();
    let dir_button =
        button(
            container(arrow.view(m.icon_size).style(move |th: &Theme, _| {
                iced::widget::svg::Style {
                    color: Some(th.semantic.icon_solid(!enabled)),
                }
            }))
            .center(Length::Fixed(side)),
        )
        .padding(Padding::ZERO)
        .style(sb::default)
        .on_press_maybe(press);

    Some(
        row![
            super::typography::caption_heading("Sort by").style(crate::style::text::dim),
            picker,
            dir_button,
        ]
        .spacing(m.space_s)
        .align_y(Vertical::Center)
        .into(),
    )
}

fn cards_view<'a, T: 'a, M: Clone + 'a>(t: &ResponsiveTable<'a, T, M>) -> Element<'a, M> {
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
    let scroll = super::scrolled_window(column(cards).spacing(m.space_m).width(Length::Fill))
        .height(Length::Fill);
    let mut body = column![].spacing(m.space_m);
    if let Some(sort) = sort_control(t) {
        body = body.push(sort);
    }
    body = body.push(scroll);
    if let Some(footer) = &t.footer {
        body = body.push(footer());
    }
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

    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Key {
        Name,
        Age,
    }

    #[test]
    fn sorting_reads_the_state_and_reports_the_key() {
        let rows = [()];
        let mut state = SortState::new().multi(true);
        state.activate(Key::Age, false);
        state.activate(Key::Name, true);
        let t = ResponsiveTable::<(), Key>::new(&rows)
            .column(Column::new("Name", |_| space().into()).sortable(Key::Name))
            .column(Column::new("Age", |_| space().into()).sortable(Key::Age))
            .column(Column::new("Plain", |_| space().into()))
            .sort(&state)
            .on_sort(|k: Key| k);
        let s = t.sorting(&t.columns[0]).unwrap();
        assert_eq!(
            (s.press, s.dir, s.priority),
            (Key::Name, Some(Direction::Asc), Some(2))
        );
        let s = t.sorting(&t.columns[1]).unwrap();
        assert_eq!((s.press, s.priority), (Key::Age, Some(1)));
        assert!(t.sorting(&t.columns[2]).is_none(), "no key, not sortable");
    }

    #[test]
    fn a_column_is_not_sortable_without_a_handler_or_with_another_key_type() {
        let rows = [()];
        let state = SortState::<Key>::new();
        let t = ResponsiveTable::<(), Key>::new(&rows)
            .column(Column::new("Name", |_| space().into()).sortable(Key::Name))
            .sort(&state);
        assert!(t.sorting(&t.columns[0]).is_none(), "no on_sort");
        let t = t.on_sort(|n: u8| Key::Age.max_if(n));
        assert!(t.sorting(&t.columns[0]).is_none(), "key type differs");
    }

    impl Key {
        fn max_if(self, _: u8) -> Key {
            self
        }
    }
}
