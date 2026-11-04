//! `ItemRow`: one row of a music, podcast, video or file list: a picture, a title with a subtitle, optional extra columns, and a trailing
//! part (a time, a menu). Together with [`VirtualList`](super::VirtualList) and a
//! [`SortHeader`](super::SortHeader) it is the list component.
//!
//! * **Every column is flex** ([`FlexCells`](super::FlexCells)). A fixed width is a *starting* size:
//!   when the row narrows, all the columns give way together, in proportion, **none below what its
//!   header label needs**, so the title is never squeezed out and a header is never cut off. The
//!   header and the rows resolve the same flex, so they line up at any width.
//! * **Regular**: picture, then the title over its subtitle. The title column **grows**
//!   (`flex-grow: 1`): it covers all the width the picture, the extra columns and the suffix leave,
//!   so give the other columns fixed widths.
//! * **Compact** ([`compact`](ItemRow::compact)): no picture, and the title and subtitle share
//!   the line, each in its own column, for dense lists.
//! * [`column`](ItemRow::column) adds a cell of a given width (an album, a BPM, a time);
//!   [`suffix`](ItemRow::suffix) adds the trailing part (a menu button).
//!
//! **The row is content, like a `<tr>`**: no button, no hover, no press, no surface, and no
//! inset unless asked for ([`padding`](ItemRow::padding), or [`inset`](ItemRow::inset) for the
//! classic list inset). The caller wraps it in the surface that makes it a control:
//! [`as_button`](super::as_button): events only, so the row keeps its own layout.
//!
//! ```ignore
//! ItemRow::new(&track.title)
//!     .subtitle(&track.artist)
//!     .picture(cover(track.id, 40.0))
//!     .column(Length::Fixed(160.0), caption(&track.album))
//!     .column(Length::Fixed(44.0), caption(track.time()))
//!     .suffix(menu_button(Icon::ViewMore, menu))
//!     .inset()
//!
//! as_button(row).height(Length::Fill).on_press(Msg::Play(track.id))
//! ```

use super::sort_header::flex_of;
use super::{Element, Flex, FlexCells, SortHeader, typography as ty};
use crate::Metrics;
use crate::primitives::sorting::SortState;
use crate::style::text as st;
use iced::alignment::Horizontal;
use iced::widget::{column, container};
use iced::{Length, Padding};

/// The usual width of a picture (a cover, an avatar) at the start of a row.
pub const PICTURE_WIDTH: f32 = 40.0;
/// The usual width of a time or a count column.
pub const TIME_WIDTH: f32 = 44.0;
/// The usual width of the trailing part: a menu button.
pub const SUFFIX_WIDTH: f32 = 36.0;

/// See the [module](self).
pub struct ItemRow<'a, M> {
    title: String,
    subtitle: Option<String>,
    picture: Option<Element<'a, M>>,
    columns: Vec<(Flex, Horizontal, Element<'a, M>)>,
    /// The flex and alignment a shared [`ItemColumns`] gave each column, for [`cell`](Self::cell).
    spec: Vec<(Flex, Horizontal)>,
    /// The title's (and, compact, the subtitle's) flex.
    title_flex: Flex,
    subtitle_flex: Flex,
    suffix: Vec<Element<'a, M>>,
    compact: bool,
    height: Length,
    padding: Option<Padding>,
    inset: bool,
    metrics: Metrics,
}

impl<'a, M: 'a> ItemRow<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            picture: None,
            columns: Vec::new(),
            spec: Vec::new(),
            title_flex: flex_of(Length::Fill, "Title"),
            subtitle_flex: flex_of(Length::Fill, "Artist"),
            suffix: Vec::new(),
            compact: false,
            height: Length::Shrink,
            padding: None,
            inset: false,
            metrics: Metrics::desktop(),
        }
    }
    /// The dimmed line under the title (the artist, the show).
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
    /// The picture at the start (a cover, an avatar). Hidden in compact mode.
    pub fn picture(mut self, picture: impl Into<Element<'a, M>>) -> Self {
        self.picture = Some(picture.into());
        self
    }
    /// An extra cell of `width` after the title: an album, a BPM, a time. Add as many as needed.
    pub fn column(mut self, width: impl Into<Length>, cell: impl Into<Element<'a, M>>) -> Self {
        self.columns
            .push((flex_of(width.into(), ""), Horizontal::Left, cell.into()));
        self
    }
    /// Aligns the cell just added within its width: `Right` for a time or a count.
    pub fn align(mut self, align: Horizontal) -> Self {
        if let Some(last) = self.columns.last_mut() {
            last.1 = align;
        }
        self
    }
    /// The next extra cell, in the width and alignment the shared [`ItemColumns`] gave this
    /// column (see [`ItemColumns::row`]). Cells are given in the columns' order.
    pub fn cell(mut self, cell: impl Into<Element<'a, M>>) -> Self {
        let (width, align) = self
            .spec
            .get(self.columns.len())
            .copied()
            .unwrap_or((flex_of(Length::Fill, ""), Horizontal::Left));
        self.columns.push((width, align, cell.into()));
        self
    }
    /// A trailing part: a menu button, a toggle. Add as many as needed.
    pub fn suffix(mut self, part: impl Into<Element<'a, M>>) -> Self {
        self.suffix.push(part.into());
        self
    }
    /// No picture, and the subtitle on the title's line.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// The row's height. `Fill` centers the content in the height it is given (the rows of a
    /// [`VirtualList`](super::VirtualList)); the default fits the content.
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
    /// The space between the row's edge and its content. None by default, button or not: a
    /// row is content, and the surface around it (a card, a list) sets the inset.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }
    /// The classic inset for a row that is its own button in a list: `space_s` left, `space_m`
    /// right, and a little above and below when the row fits its content. Overridden by
    /// [`padding`](Self::padding).
    pub fn inset(mut self) -> Self {
        self.inset = true;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<ItemRow<'a, M>> for Element<'a, M> {
    fn from(r: ItemRow<'a, M>) -> Self {
        let m = r.metrics;
        // Every cell is flex: the line divides its width by the same rules as the header, so the
        // columns line up and shrink together, none below what its label needs.
        let mut line = FlexCells::new().spacing(m.space_m);
        if r.compact {
            line = line.cell(r.title_flex, ty::body(r.title).width(Length::Fill));
            if let Some(s) = r.subtitle {
                line = line.cell(
                    r.subtitle_flex,
                    ty::caption(s).style(st::dim).width(Length::Fill),
                );
            }
        } else {
            if let Some(p) = r.picture {
                line = line.cell(Flex::fixed(PICTURE_WIDTH), p);
            }
            let mut text = column![ty::body(r.title)];
            if let Some(s) = r.subtitle {
                text = text.push(ty::caption(s).style(st::dim));
            }
            line = line.cell(r.title_flex, container(text).width(Length::Fill));
        }
        for (flex, align, cell) in r.columns {
            line = line.cell(flex, container(cell).width(Length::Fill).align_x(align));
        }
        for part in r.suffix {
            line = line.cell(Flex::fixed(SUFFIX_WIDTH), part);
        }
        let fits = r.height == Length::Shrink;
        let pad = r.padding.unwrap_or(if r.inset {
            Padding {
                left: m.space_s,
                right: m.space_m,
                top: if fits { 4.0 } else { 0.0 },
                bottom: if fits { 4.0 } else { 0.0 },
            }
        } else {
            Padding::ZERO
        });
        container(container(line).padding(pad))
            .width(Length::Fill)
            .center_y(r.height)
            .height(r.height)
            .into()
    }
}

#[derive(Clone)]
struct Def<K> {
    key: Option<K>,
    label: String,
    width: Length,
    align: Horizontal,
}

/// The columns of an item list, **written once** for both the [`SortHeader`] and the rows, so the
/// labels sit over their cells without matching widths by hand.
///
/// ```ignore
/// let cols = ItemColumns::new()
///     .picture(true)                                   // PICTURE_WIDTH, over the cover
///     .title(Col::Title, "Title")
///     .column(Col::Album, "Album").width(Length::Fixed(160.0))
///     .column(Col::Time, "Time").width(Length::Fixed(TIME_WIDTH)).align(Horizontal::Right)
///     .suffix(true);                                   // SUFFIX_WIDTH, over the menu
///
/// cols.header(&self.sort, Msg::Sort)                   // the header
/// cols.row(&t.title).subtitle(&t.artist)               // each row, cells in order
///     .picture(cover).cell(caption(&t.album)).cell(caption(t.time())).suffix(menu)
/// ```
#[derive(Clone)]
pub struct ItemColumns<K> {
    title: Option<(Option<K>, String)>,
    subtitle: Option<(Option<K>, String)>,
    cols: Vec<Def<K>>,
    picture: bool,
    suffix: bool,
    compact: bool,
}

impl<K> Default for ItemColumns<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> ItemColumns<K> {
    pub fn new() -> Self {
        Self {
            title: None,
            subtitle: None,
            cols: Vec::new(),
            picture: false,
            suffix: false,
            compact: false,
        }
    }
    /// The title column, sortable under `key`.
    pub fn title(mut self, key: K, label: impl Into<String>) -> Self {
        self.title = Some((Some(key), label.into()));
        self
    }
    /// The subtitle column, shown in the header only in [compact](Self::compact) mode (where the
    /// subtitle has a column of its own). Sortable under `key`.
    pub fn subtitle(mut self, key: K, label: impl Into<String>) -> Self {
        self.subtitle = Some((Some(key), label.into()));
        self
    }
    /// An extra column after the title, sortable under `key`. Like the title it **grows** to
    /// cover as much width as the others leave (`flex-grow: 1`), so the free width is shared;
    /// [`width`](Self::width) fixes it instead.
    pub fn column(mut self, key: K, label: impl Into<String>) -> Self {
        self.cols.push(Def {
            key: Some(key),
            label: label.into(),
            width: Length::Fill,
            align: Horizontal::Left,
        });
        self
    }
    /// An extra column whose label does not sort. It grows like any column.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.cols.push(Def {
            key: None,
            label: label.into(),
            width: Length::Fill,
            align: Horizontal::Left,
        });
        self
    }
    /// The width of the column just added, in place of growing: fixed (a time, a count), or a
    /// `FillPortion` to take more or less of the free width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        if let Some(last) = self.cols.last_mut() {
            last.width = width.into();
        }
        self
    }
    /// Aligns the column just added, header label and cells alike.
    pub fn align(mut self, align: Horizontal) -> Self {
        if let Some(last) = self.cols.last_mut() {
            last.align = align;
        }
        self
    }
    /// Room at the start for a picture ([`PICTURE_WIDTH`]); ignored in compact mode.
    pub fn picture(mut self, picture: bool) -> Self {
        self.picture = picture;
        self
    }
    /// Room at the end for a trailing part ([`SUFFIX_WIDTH`]).
    pub fn suffix(mut self, suffix: bool) -> Self {
        self.suffix = suffix;
        self
    }
    /// No picture, the subtitle in a column of its own.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// A row in these columns: compact as the columns are, and ready for [`ItemRow::cell`].
    pub fn row<'a, M: 'a>(&self, title: impl Into<String>) -> ItemRow<'a, M> {
        let mut r = ItemRow::new(title).compact(self.compact).inset();
        // The flex the header gives each column (down to its label's floor), so cells agree.
        r.spec = self
            .cols
            .iter()
            .map(|c| (flex_of(c.width, &c.label), c.align))
            .collect();
        if let Some((_, label)) = &self.title {
            r.title_flex = flex_of(Length::Fill, label);
        }
        if let Some((_, label)) = &self.subtitle {
            r.subtitle_flex = flex_of(Length::Fill, label);
        }
        r
    }
}

impl<K: PartialEq + Clone> ItemColumns<K> {
    /// The header over these columns, in the same order and widths as the rows. `on_sort` gets
    /// the key of a pressed label.
    pub fn header<'a, M: Clone + 'a>(
        &self,
        state: &'a SortState<K>,
        on_sort: impl Fn(K) -> M + 'a,
    ) -> SortHeader<'a, K, M>
    where
        K: 'a,
    {
        let mut h = SortHeader::new(state, on_sort);
        if self.picture && !self.compact {
            h = h.space(Length::Fixed(PICTURE_WIDTH));
        }
        let put = |h: SortHeader<'a, K, M>, def: &(Option<K>, String), w: Length| match &def.0 {
            Some(k) => h.column(k.clone(), def.1.clone()).width(w),
            None => h.label(def.1.clone()).width(w),
        };
        if let Some(t) = &self.title {
            h = put(h, t, Length::Fill);
        }
        if self.compact
            && let Some(s) = &self.subtitle
        {
            h = put(h, s, Length::Fill);
        }
        for c in &self.cols {
            let def = (c.key.clone(), c.label.clone());
            h = put(h, &def, c.width).align(c.align);
        }
        if self.suffix {
            h = h.space(Length::Fixed(SUFFIX_WIDTH));
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_its_parts() {
        let r: ItemRow<'_, ()> = ItemRow::new("Song")
            .subtitle("Artist")
            .column(Length::Fixed(44.0), iced::widget::space())
            .suffix(iced::widget::space())
            .compact(true)
            .height(Length::Fill);
        assert_eq!(r.columns.len(), 1);
        assert_eq!(r.suffix.len(), 1);
        assert!(r.compact && r.height == Length::Fill && r.subtitle.as_deref() == Some("Artist"));
    }

    #[test]
    fn a_row_is_content_with_no_inset_unless_asked() {
        let r: ItemRow<'_, u8> = ItemRow::new("x");
        assert!(r.padding.is_none() && !r.inset);
        let _: Element<'_, u8> = r.into();
        let _: Element<'_, u8> = ItemRow::<u8>::new("x").padding(6.0).into();
        let _: Element<'_, u8> = ItemRow::<u8>::new("x").inset().into();
    }

    #[test]
    fn a_row_gives_a_small_column_the_room_its_header_needs() {
        let cols = ItemColumns::<u8>::new()
            .title(0, "Title")
            .column(1, "Duration")
            .width(Length::Fixed(20.0));
        let r: ItemRow<'_, ()> = cols.row("x");
        // The cell's minimum is its header's label, so the two agree at any width.
        assert_eq!(
            r.spec[0].0.0.min,
            crate::widget::sort_header::label_floor("Duration")
        );
        assert_eq!(
            r.title_flex.0.min,
            crate::widget::sort_header::label_floor("Title")
        );
    }

    #[test]
    fn rows_take_their_widths_from_the_shared_columns() {
        let cols = ItemColumns::<u8>::new()
            .title(0, "Title")
            .column(1, "Album")
            .width(Length::FillPortion(2))
            .column(2, "Time")
            .width(Length::Fixed(TIME_WIDTH))
            .align(Horizontal::Right);
        let r: ItemRow<'_, ()> = cols
            .row("Song")
            .cell(iced::widget::space())
            .cell(iced::widget::space());
        assert_eq!(r.columns[0].0.0.grow, 2.0);
        assert_eq!(r.columns[0].1, Horizontal::Left);
        assert!(r.columns[1].0.0.grow == 0.0 && r.columns[1].0.0.basis >= TIME_WIDTH);
        assert_eq!(r.columns[1].1, Horizontal::Right);
    }

    #[test]
    fn the_header_leaves_room_for_the_picture_and_the_suffix() {
        let state = SortState::<u8>::new();
        let cols = ItemColumns::<u8>::new()
            .picture(true)
            .title(0, "Title")
            .suffix(true);
        // space, title, space
        assert_eq!(cols.header::<()>(&state, |_| ()).len(), 3);
        // No picture in compact mode.
        assert_eq!(cols.compact(true).header::<()>(&state, |_| ()).len(), 2);
    }
}
