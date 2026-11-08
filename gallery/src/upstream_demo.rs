//! Components that came up from the weave music app: the virtual list, editable text, the detail
//! header and the icon set.

use crate::example_detail_header::DetailHeader;
use iced_adaptive_design as iad;

use iad::components::{Avatar, AvatarShape, ChipSize};
use iad::config::Density;
use iad::layouts::virtualize::Viewport;
use iad::layouts::{FlowGrid, RowHeight, VirtualList};
use iad::primitives::history::History;
use iad::primitives::reorder::{Event as ReorderEvent, ReorderState};
use iad::primitives::sorting::{Direction, SortState, cmp_str};
use iad::widgets::{ChipGroup, ChipOption, ItemColumns, PreferencesGroup, SectionHeading};
use iad::{Element, Icon, Metrics};
use iad::{components as cmp, layouts as lay, widgets as w};
use iced::widget::{column, container, row, text};
use iced::{Color, Length};

const TOTAL: usize = 10_000;
const ROW: f32 = 40.0;
const BIO: &str = "Recorded over three winters in a converted barn. The songs were written on the road and finished in the barn, where the whole band could play at once. Most of what you hear is the first or second take, with the room left in.";

#[derive(Debug, Clone)]
pub enum Msg {
    Scroll(Viewport),
    Input(iced::widget::text_editor::Action),
    NickInput(iced::widget::text_editor::Action),
    Narrow(bool),
    Noop,
    Sort(Col),
    Compact(bool),
    RowPress(bool),
    RowPad(bool),
    RowPadPx(f32),
    Scroll2(Viewport),
    ChipToggle(u8),
    Query(String),
    Open(&'static str),
    Go(&'static str),
    Back,
    FlowText(iced::widget::text_editor::Action),
    CardsCompact(bool),
    CardsHeight(usize),
    CardsCols(usize),
    Queue(ReorderEvent),
    Remove(usize),
}

/// The columns of the item list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Col {
    Title,
    Artist,
    Album,
    Time,
}

/// A track of the item list demo.
struct Track {
    title: String,
    artist: &'static str,
    album: &'static str,
    secs: u32,
}

const ARTISTS: [&str; 5] = ["Nina Vale", "The Barn Band", "Okoro", "Lumen", "Harbor"];
const ALBUMS: [&str; 4] = ["Winter Barn", "Long Road", "Low Light", "Second Take"];
const TAGS: [&str; 12] = [
    "Late night",
    "Focus",
    "Workout",
    "Jazz",
    "Acoustic",
    "Road trip",
    "Rainy day",
    "Party",
    "Study",
    "Chill",
    "Sleep",
    "Morning",
];

pub struct State {
    sort: SortState<Col>,
    tracks: Vec<Track>,
    compact: bool,
    media_vp: Viewport,
    chips: Vec<u8>,
    query: String,
    history: History<&'static str>,
    flow_text: iced::widget::text_editor::Content,
    cards_compact: bool,
    /// 0 shared, 1 own, 2 fixed: how tall the items of a row are.
    cards_height: usize,
    row_press: bool,
    row_pad: bool,
    row_pad_px: f32,
    cards_cols: usize,
    queue: ReorderState<&'static str>,
    viewport: Viewport,
    bio: iced::widget::text_editor::Content,
    nick: iced::widget::text_editor::Content,
    narrow: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            viewport: Viewport::new(320.0),
            bio: iced::widget::text_editor::Content::with_text(BIO),
            nick: iced::widget::text_editor::Content::new(),
            narrow: false,
            sort: SortState::by(Col::Title, Direction::Asc),
            tracks: (0..400)
                .map(|i| Track {
                    title: format!("Song {}", (i * 37) % 400 + 1),
                    artist: ARTISTS[i % ARTISTS.len()],
                    album: ALBUMS[i % ALBUMS.len()],
                    secs: 150 + ((i * 53) % 200) as u32,
                })
                .collect(),
            compact: false,
            media_vp: Viewport::new(320.0),
            chips: vec![0, 3],
            query: String::new(),
            history: History::new("Home"),
            cards_compact: false,
            cards_height: 0,
            row_press: true,
            row_pad: true,
            row_pad_px: 0.0,
            cards_cols: 0,
            queue: ReorderState::live(vec![
                "Winter Barn",
                "Long Road",
                "Low Light",
                "Second Take",
                "Harbor",
            ]),
            flow_text: iced::widget::text_editor::Content::with_text(
                "A paragraph the content decides the height of.",
            ),
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Scroll(v) => self.viewport = v,
            Msg::Input(action) => self.bio.perform(action),
            Msg::NickInput(action) => self.nick.perform(action),
            Msg::Narrow(n) => self.narrow = n,
            Msg::Noop => {}
            Msg::Sort(c) => {
                self.sort.activate(c, false);
                let sort = &self.sort;
                sort.apply(&mut self.tracks, |k, a, b| match k {
                    Col::Title => cmp_str(&a.title, &b.title),
                    Col::Artist => cmp_str(a.artist, b.artist),
                    Col::Album => cmp_str(a.album, b.album),
                    Col::Time => a.secs.cmp(&b.secs),
                });
            }
            Msg::Compact(c) => self.compact = c,
            Msg::RowPress(b) => self.row_press = b,
            Msg::RowPad(b) => self.row_pad = b,
            Msg::RowPadPx(v) => self.row_pad_px = v,
            Msg::Scroll2(v) => self.media_vp = v,
            Msg::ChipToggle(k) => match self.chips.iter().position(|c| *c == k) {
                Some(i) => {
                    self.chips.remove(i);
                }
                None => self.chips.push(k),
            },
            Msg::Query(q) => self.query = q,
            Msg::Open(p) => self.history.open(p),
            Msg::Go(p) => self.history.go(p),
            Msg::CardsCompact(c) => self.cards_compact = c,
            Msg::CardsHeight(h) => self.cards_height = h,
            Msg::CardsCols(n) => self.cards_cols = n,
            Msg::Queue(e) => {
                self.queue.apply(e);
            }
            Msg::Remove(i) => {
                self.queue.remove(i);
            }
            Msg::FlowText(action) => self.flow_text.perform(action),
            Msg::Back => {
                self.history.back();
            }
        }
    }

    fn group<'a>(title: &'static str, desc: &'static str, m: Metrics) -> PreferencesGroup<'a, Msg> {
        PreferencesGroup::<Msg>::new()
            .title(title)
            .description(desc)
            .metrics(m)
    }

    pub fn virtual_view(&self, m: Metrics) -> Element<'_, Msg> {
        let built = self.viewport.window(TOTAL, ROW, 4);
        let list = VirtualList::new(TOTAL, ROW, &self.viewport, move |i| {
            // The whole row is the target: rounded, with a hover that reads on a plain page.
            iced::widget::button(
                row![
                    text(format!("Track {}", i + 1)).width(Length::Fill),
                    text(format!("{}:{:02}", 2 + i % 4, (i * 7) % 60)).size(12),
                ]
                .padding([0.0, 4.0])
                .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iad::styles::button::flat_row(8.0, 0.08, 0.14))
            .on_press(Msg::Noop)
            .into()
        })
        .on_scroll(Msg::Scroll);
        Self::group(
            "Virtual list",
            "Ten thousand rows of one height. Only the rows in view are built.",
            m,
        )
        .push(w::card(
            column![
                container(list).height(Length::Fixed(320.0)),
                text(format!(
                    "Built rows {} to {} of {TOTAL}",
                    built.start + 1,
                    built.end
                ))
                .size(12),
            ]
            .spacing(m.space_s),
        ))
        .into()
    }

    pub fn editable_view(&self, m: Metrics) -> Element<'_, Msg> {
        column![
            w::MultilineEntry::new("Description", &self.bio, Msg::Input).metrics(m),
            w::MultilineEntry::from_input(
                "Nickname",
                cmp::text_view(&self.nick)
                    .single_line(true)
                    .placeholder("Add a nickname")
                    .on_action(Msg::NickInput)
            )
            .metrics(m),
            text("Content below the text moves as the description grows"),
        ]
        .spacing(m.space_m)
        .into()
    }

    pub fn header_view(&self, m: Metrics) -> Element<'_, Msg> {
        let tint = Color::from_rgb8(0x9c, 0x4a, 0xd1);
        let cover = container(text(""))
            .width(Length::Fixed(if self.narrow { 160.0 } else { 180.0 }))
            .height(Length::Fixed(if self.narrow { 160.0 } else { 180.0 }))
            .style(move |_: &iad::Theme| container::Style {
                background: Some(tint.into()),
                border: iced::Border::default().rounded(10),
                ..Default::default()
            });
        let header = DetailHeader::new("Evening drive")
            .kind("Playlist")
            .cover(cover)
            .description(text("Slow songs for a long road."))
            .facts(text("24 songs, 1 hr 32 min").size(12))
            .primary(
                cmp::suggested_circular_button(Icon::Play.view(m.icon_size)).on_press(Msg::Noop),
            )
            .action(cmp::icon_button(Icon::Heart).on_press(Msg::Noop))
            .action(cmp::icon_button(Icon::ViewMore).on_press(Msg::Noop))
            .end(cmp::flat_button(text("Share")).on_press(Msg::Noop))
            .tint(tint)
            .narrow(self.narrow)
            .metrics(m);
        column![
            Self::group(
                "Detail header",
                "Wide or narrow, with a wash of the cover's color fading down from the top.",
                m
            )
            .push(w::SwitchRow::new("Narrow arrangement", self.narrow, Msg::Narrow).metrics(m)),
            header,
        ]
        .spacing(m.group_spacing)
        .into()
    }

    /// An outline where an icon will be.
    fn placeholder<'a>(cell: f32) -> Element<'a, Msg> {
        container(
            container(text(""))
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .style(|t: &iad::Theme| container::Style {
                    background: Some(iad::color::alpha(t.semantic.window.fg, 0.08).into()),
                    border: iced::Border::default().rounded(6),
                    ..Default::default()
                }),
        )
        .width(Length::Fixed(cell))
        .height(Length::Fixed(64.0))
        .center_x(Length::Fixed(cell))
        .into()
    }

    /// The icons in a plain grid, revealed a row a frame (the first draw of an SVG is its cost).
    pub fn icons_view(&self, m: Metrics, revealed: usize) -> Element<'_, Msg> {
        const CELL: f32 = 110.0;
        let grid = iced::widget::responsive(move |size| {
            let per_row = ((size.width / CELL).floor() as usize).max(1);
            let rows = Icon::ALL.len().div_ceil(per_row);
            // A plain grid. Rows not yet revealed hold an outline: their icons are drawn a row a
            // frame, so no frame rasterizes them all.
            let body = column((0..rows).map(|r| {
                if r >= revealed {
                    return row((0..per_row).map(|_| Self::placeholder(CELL))).into();
                }
                let cells = Icon::ALL.iter().skip(r * per_row).take(per_row).map(|i| {
                    container(
                        column![i.view(24.0), text(format!("{i:?}")).size(11)]
                            .spacing(4)
                            .align_x(iced::Alignment::Center),
                    )
                    .width(Length::Fixed(CELL))
                    .height(Length::Fixed(64.0))
                    .center_x(Length::Fixed(CELL))
                    .into()
                });
                row(cells).into()
            }));
            lay::scrolled_window(body).width(Length::Fill).into()
        });
        Self::group(
            "Icons",
            "Bundled control and media SVGs alongside installed symbolic icons, revealed a row at a time. Configure [icons] theme for system artwork. Global icon overrides replace predefined defaults; IconSource also accepts explicit system names and application SVGs.",
            m,
        )
        .push(w::card(container(grid).height(Length::Fixed(380.0))))
        .into()
    }

    /// A configurable `ItemRow`: content, optionally wrapped in a `as_button`.
    fn row_playground(&self, m: Metrics) -> Element<'_, Msg> {
        let mut row = w::ItemRow::new("Late Night Mix")
            .subtitle("dynamic \u{b7} 24 tracks")
            .picture(
                Avatar::new(w::PICTURE_WIDTH, "Late Night Mix")
                    .icon(Icon::MusicNote)
                    .shape(AvatarShape::Rounded),
            );
        if self.row_pad {
            row = row.padding(self.row_pad_px);
        }
        // The row is content; the caller wraps it in the surface that makes it a control.
        let stage: Element<'_, Msg> = match self.row_press {
            false => w::card(row),
            true => iced::widget::button(w::card(row))
                .padding(0)
                .style(iad::styles::button::events)
                .width(Length::Fill)
                .on_press(Msg::Noop)
                .into(),
        };
        let mut group = Self::group(
            "Item row: content, wrapped by the caller",
            "The row is content: no hover, press or padding. as_button around it makes it a control; padding(..) insets it; inset() is the classic list inset.",
            m,
        )
        .push(w::SwitchRow::new("Wrap in a as_button", self.row_press, Msg::RowPress).metrics(m))
        .push(w::SwitchRow::new("padding(..)", self.row_pad, Msg::RowPad).metrics(m));
        if self.row_pad {
            group = group.push(
                w::SliderRow::new("Padding", self.row_pad_px, 0.0..=24.0, Msg::RowPadPx).metrics(m),
            );
        }
        column![group, stage].spacing(m.group_spacing).into()
    }

    pub fn media_view(&self, m: Metrics) -> Element<'_, Msg> {
        let row_h = if self.compact { 36.0 } else { 56.0 };
        let compact = self.compact;
        // The columns, written once: the header and every row take their widths from them.
        let cols = ItemColumns::new()
            .picture(true)
            .compact(compact)
            .title(Col::Title, "Title")
            .subtitle(Col::Artist, "Artist")
            .column(Col::Album, "Album")
            .width(Length::Fixed(160.0))
            .column(Col::Time, "Time")
            .width(Length::Fixed(w::TIME_WIDTH))
            .align(iced::alignment::Horizontal::Right)
            .suffix(true);
        let tracks = &self.tracks;
        let spec = cols.clone();
        let list = VirtualList::new(tracks.len(), row_h, &self.media_vp, move |i| {
            let t = &tracks[i];
            let time = format!("{}:{:02}", t.secs / 60, t.secs % 60);
            let mut r = spec
                .row(t.title.clone())
                .subtitle(t.artist)
                .height(Length::Fill)
                .cell(cmp::typography::caption(t.album))
                .cell(cmp::typography::caption(time))
                .suffix(cmp::icon_button(Icon::ViewMore).on_press(Msg::Noop));
            if !compact {
                r = r.picture(
                    Avatar::new(w::PICTURE_WIDTH, t.title.clone())
                        .icon(Icon::MusicNote)
                        .shape(AvatarShape::Rounded),
                );
            }
            iced::widget::button(r)
                .padding(0)
                .width(Length::Fill)
                .style(iad::styles::button::flat_row(8.0, 0.08, 0.14))
                .height(Length::Fill)
                .on_press(Msg::Noop)
                .into()
        })
        .on_scroll(Msg::Scroll2);
        let header = cols.header(&self.sort, Msg::Sort);
        column![
            Self::group(
                "Item list",
                "A virtual list of item rows under a sort header. The columns are written once and shared by the header and the rows; the time is right-aligned.",
                m,
            )
            .push(w::SwitchRow::new("Compact", self.compact, Msg::Compact).metrics(m)),
            w::card(
                column![header, container(list).height(Length::Fixed(320.0))].spacing(m.space_xs),
            ),
            self.row_playground(m),
        ]
        .spacing(m.group_spacing)
        .into()
    }

    pub fn chips_view(&self, m: Metrics) -> Element<'_, Msg> {
        let options = |from: usize, to: usize| {
            (from..to).map(|i| {
                let mut o = ChipOption::new(i as u8, TAGS[i]).color(iad::color::from_index(i + 3));
                if i % 3 == 0 {
                    o = o.icon(Icon::Tag);
                }
                o
            })
        };
        let first = ChipGroup::new(options(0, 6), &self.chips, Msg::ChipToggle)
            .search(&self.query, Msg::Query)
            .placeholder("Find a tag")
            .on_color(iced::Color::WHITE);
        // The second group shares the first one's query and draws no entry of its own.
        let second = ChipGroup::new(options(6, 12), &self.chips, Msg::ChipToggle)
            .query(&self.query)
            .on_color(iced::Color::WHITE)
            .size(ChipSize::Small);
        column![
            Self::group(
                "Chip group",
                "Toggle chips that wrap, filtered as you type. Each has its own color and icon; selected ones use white text so they read alike.",
                m,
            )
            .push(w::card(first)),
            Self::group(
                "Sharing one query",
                "`.query(..)` filters without drawing an entry: this group follows the field above.",
                m,
            )
            .push(w::card(second)),
        ]
        .spacing(m.group_spacing)
        .into()
    }

    pub fn grid_view(&self, m: Metrics) -> Element<'_, Msg> {
        let compact = self.cards_compact;
        let density = if compact {
            Density::Compact
        } else {
            Density::Comfort
        };
        let cards = (0..9).map(|i| {
            let name = ALBUMS[i % ALBUMS.len()];
            // Every third title is long, so it wraps and the heights differ.
            let title = if i % 3 == 1 {
                format!("{name} (Deluxe Anniversary Edition) {}", i + 1)
            } else {
                format!("{name} {}", i + 1)
            };
            w::ItemCard::new(title)
                .subtitle(ARTISTS[i % ARTISTS.len()])
                .picture(
                    Avatar::new(if compact { 36.0 } else { 72.0 }, format!("{name}{i}"))
                        .icon(Icon::MusicNote)
                        .shape(AvatarShape::Rounded),
                )
                .density(density)
                .on_press(Msg::Noop)
                .into()
        });
        let grid = FlowGrid::new(cards)
            .max_width(180.0)
            .row_height(match self.cards_height {
                1 => RowHeight::Own,
                2 => RowHeight::Fixed(150.0),
                _ => RowHeight::Shared,
            });
        let grid = match self.cards_cols {
            0 => grid,
            n => grid.cols(n),
        };
        column![
            SectionHeading::new("Recently played")
                .subtitle("The item sets its density; the grid sets the columns")
                .link("View all", Msg::Noop),
            w::SwitchRow::new("Compact", compact, Msg::CardsCompact).metrics(m),
            w::ToggleGroupRow::new(
                "Columns",
                vec![
                    cmp::ToggleItem::new(0, "Fit"),
                    cmp::ToggleItem::new(1, "1"),
                    cmp::ToggleItem::new(2, "2"),
                    cmp::ToggleItem::new(3, "3"),
                ],
                Some(self.cards_cols),
                Msg::CardsCols,
            )
            .subtitle("Fit adds columns as the width allows; one column is a list")
            .metrics(m),
            w::ToggleGroupRow::new(
                "Row height",
                vec![
                    cmp::ToggleItem::new(0, "Shared"),
                    cmp::ToggleItem::new(1, "Own"),
                    cmp::ToggleItem::new(2, "Fixed"),
                ],
                Some(self.cards_height),
                Msg::CardsHeight,
            )
            .subtitle("Shared: a row is as tall as its tallest item. Own: each keeps its height. Fixed: one height for all")
            .metrics(m),
            grid,
        ]
        .spacing(m.space_m)
        .into()
    }

    /// A 12-column grid: cells of different spans, a wrap and an offset.
    pub fn table_grid_view(&self, m: Metrics) -> Element<'_, Msg> {
        let cell = |label: &'static str, span, offset| {
            let body = w::card(
                container(cmp::typography::label(label))
                    .center_x(Length::Fill)
                    .padding(m.space_m),
            );
            lay::TableCell::new(body).span(span).offset(offset)
        };
        column![
            Self::group("Spans", "Each cell takes some of the 12 columns.", m).push(
                lay::TableGrid::new([
                    cell("8", 8, 0),
                    cell("4", 4, 0),
                    cell("4", 4, 0),
                    cell("4", 4, 0),
                    cell("4", 4, 0)
                ])
                .spacing(m.space_s),
            ),
            Self::group("Offset", "Columns left empty before a cell.", m).push(
                lay::TableGrid::new([cell("6, offset 3", 6, 3), cell("3", 3, 0)])
                    .spacing(m.space_s),
            ),
            Self::group(
                "Wrap",
                "A cell that does not fit in the rest of a row starts the next one.",
                m
            )
            .push(
                lay::TableGrid::new([cell("7", 7, 0), cell("7", 7, 0), cell("5", 5, 0)])
                    .spacing(m.space_s),
            ),
        ]
        .spacing(m.group_spacing)
        .into()
    }

    /// A queue: always editable, every drop committed, a delete button before the grip.
    pub fn queue_view(&self, m: Metrics) -> Element<'_, Msg> {
        let items = self.queue.items();
        let list = w::ReorderableList::of(&self.queue, move |name| {
            w::ActionRow::new(*name).metrics(m).into()
        })
        .row_action(|name| {
            let i = items.iter().position(|n| n == name).unwrap_or(0);
            cmp::icon_button(Icon::Delete)
                .on_press(Msg::Remove(i))
                .into()
        })
        .on_event(Msg::Queue);
        Self::group(
            "Live list",
            "Always editable: no Edit, Cancel or Done. Every drop is committed, and a delete button sits before the grip.",
            m,
        )
        .push(w::card(column![
            list,
            text(format!("Committed: {}", self.queue.committed().join(", "))).size(12),
        ].spacing(m.space_s)))
        .into()
    }

    pub fn heading_view(&self, m: Metrics) -> Element<'_, Msg> {
        column![
            SectionHeading::new("Made for you").link("View all", Msg::Noop),
            SectionHeading::new("New releases")
                .subtitle("Fresh this week")
                .link("See more", Msg::Noop),
            SectionHeading::<Msg>::new("Without a link"),
        ]
        .spacing(m.group_spacing)
        .into()
    }

    pub fn history_view(&self, m: Metrics) -> Element<'_, Msg> {
        let btn = |label: &'static str, msg: Msg| cmp::raised_button(text(label)).on_press(msg);
        let trail = self
            .history
            .trail()
            .iter()
            .chain([self.history.current()])
            .copied()
            .collect::<Vec<_>>()
            .join("  \u{203a}  ");
        Self::group(
            "History",
            "Open pushes the place you leave and Back returns to it; Go jumps to a top-level place and forgets the trail.",
            m,
        )
        .push(w::card(
            column![
                text(format!("Now: {}", self.history.current())).size(18),
                text(format!("Trail: {trail}")).size(12),
                row![
                    btn("Open Search", Msg::Open("Search")),
                    btn("Open Album", Msg::Open("Album")),
                    btn("Open Artist", Msg::Open("Artist")),
                ]
                .spacing(m.space_s)
                .wrap(),
                row![
                    btn("Go Home", Msg::Go("Home")),
                    btn("Go Library", Msg::Go("Library")),
                    cmp::suggested_button(text("Back"))
                        .on_press_maybe(self.history.can_back().then_some(Msg::Back)),
                ]
                .spacing(m.space_s)
                .wrap(),
            ]
            .spacing(m.space_m),
        ))
        .into()
    }

    /// A growing EntryRow editor: following content moves with its height.
    pub fn flow_view(&self, m: Metrics) -> Element<'_, Msg> {
        Self::group(
            "Growing textarea",
            "The editor grows with its text, then scrolls at twelve lines.",
            m,
        )
        .push(w::MultilineEntry::new("Notes", &self.flow_text, Msg::FlowText).metrics(m))
        .push(text("Content below the editor"))
        .into()
    }

    /// The page surfaces: the gutter is in one place (`Metrics::gutter`).
    pub fn surface_view(&self, m: Metrics) -> Element<'_, Msg> {
        use iad::layouts::surface;
        let tint = |t: &iad::Theme| container::Style {
            background: Some(iad::color::alpha(t.semantic.accent.bg, 0.25).into()),
            border: iced::Border::default().rounded(4),
            ..Default::default()
        };
        let sample = |label: &'static str| -> Element<'static, Msg> {
            container(text(label).size(12))
                .padding(6)
                .width(Length::Fill)
                .style(tint)
                .into()
        };
        let frame = |content: Element<'static, Msg>, h: f32| -> Element<'static, Msg> {
            container(content)
                .height(Length::Fixed(h))
                .width(Length::Fill)
                .style(|t: &iad::Theme| container::Style {
                    border: iced::Border::default()
                        .rounded(6)
                        .width(1)
                        .color(t.semantic.border),
                    ..Default::default()
                })
                .into()
        };
        let case = |title: &'static str, body: Element<'static, Msg>| {
            PreferencesGroup::<Msg>::new()
                .title(title)
                .metrics(m)
                .push(w::card(body))
        };
        column![
            Self::group(
                "Page surfaces",
                "Every page keeps the same distance from the window edge, because nothing else writes a gutter: a page is built from these. The gutter is the `gutter` token (16 px, 12 compact).",
                m,
            )
            .push(w::card(text(format!(
                "gutter {} px, band air {} px",
                surface::gutter(),
                Metrics::desktop().band_air
            )))),
            case("scrolling", frame(surface::scrolling(sample("The gutter on every side, inside the scrollable")), 90.0)),
            case("full", frame(surface::full(sample("The gutter on every side, no scrolling")), 90.0)),
            case("band", frame(surface::band(sample("The side gutter: a heading, a section")), 50.0)),
            case("toolbar", frame(surface::toolbar(sample("The side gutter and a little air")), 60.0)),
            case("row", frame(surface::row(sample("Inset so a row's hover surface starts on the gutter")), 50.0)),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
