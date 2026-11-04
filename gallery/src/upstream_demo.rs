//! Components that came up from the weave music app: the virtual list, editable text, the detail
//! header and the icon set.

use iced_adaptive_design as iad;

use iad::primitives::history::History;
use iad::primitives::reorder::{Event as ReorderEvent, ReorderState};
use iad::primitives::sorting::{Direction, SortState, cmp_str};
use iad::primitives::virtualize::Viewport;
use iad::widget::{
    self as w, Avatar, AvatarShape, CardGrid, ChipGroup, ChipOption, ChipSize, DetailHeader,
    EditableText, MediaColumns, PreferencesGroup, SectionHeading, VirtualList,
};
use iad::{Element, Icon, Metrics};
use iced::widget::{column, container, row, text};
use iced::{Color, Length};

const TOTAL: usize = 10_000;
const ROW: f32 = 40.0;
const BIO: &str = "Recorded over three winters in a converted barn. The songs were written on the road and finished in the barn, where the whole band could play at once. Most of what you hear is the first or second take, with the room left in.";

#[derive(Debug, Clone)]
pub enum Msg {
    Scroll(Viewport),
    Edit,
    Input(String),
    Done,
    Expand(bool),
    NickEdit,
    NickInput(String),
    NickDone,
    Narrow(bool),
    Noop,
    Sort(Col),
    Compact(bool),
    Scroll2(Viewport),
    ChipToggle(u8),
    Query(String),
    Open(&'static str),
    Go(&'static str),
    Back,
    Flow(bool),
    FlowText(String),
    CardsCompact(bool),
    Queue(ReorderEvent),
    Remove(usize),
}

/// The columns of the media list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Col {
    Title,
    Artist,
    Album,
    Time,
}

/// A track of the media list demo.
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
    flow_editing: bool,
    flow_text: String,
    cards_compact: bool,
    queue: ReorderState<&'static str>,
    viewport: Viewport,
    bio: String,
    editing: bool,
    expanded: bool,
    nick: String,
    nick_editing: bool,
    narrow: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            viewport: Viewport::new(320.0),
            bio: BIO.into(),
            editing: false,
            expanded: false,
            nick: String::new(),
            nick_editing: false,
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
            flow_editing: false,
            cards_compact: false,
            queue: ReorderState::live(vec![
                "Winter Barn",
                "Long Road",
                "Low Light",
                "Second Take",
                "Harbor",
            ]),
            flow_text: "A paragraph the content decides the height of.".into(),
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Scroll(v) => self.viewport = v,
            Msg::Edit => self.editing = true,
            Msg::Input(s) => self.bio = s,
            Msg::Done => self.editing = false,
            Msg::Expand(e) => self.expanded = e,
            Msg::NickEdit => self.nick_editing = true,
            Msg::NickInput(s) => self.nick = s,
            Msg::NickDone => self.nick_editing = false,
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
            Msg::Queue(e) => {
                self.queue.apply(e);
            }
            Msg::Remove(i) => {
                self.queue.remove(i);
            }
            Msg::Flow(e) => self.flow_editing = e,
            Msg::FlowText(t) => self.flow_text = t,
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
            .style(iad::style::button::flat_row(8.0, 0.08, 0.14))
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
        let bio = EditableText::new(&self.bio, Msg::Input)
            .placeholder("Add a description")
            .editing(self.editing)
            .expanded(self.expanded)
            .max_height(60.0)
            .on_edit(Msg::Edit)
            .on_done(Msg::Done)
            .on_expand(Msg::Expand);
        let nick = EditableText::new(&self.nick, Msg::NickInput)
            .placeholder("Add a nickname")
            .editing(self.nick_editing)
            .on_edit(Msg::NickEdit)
            .on_done(Msg::NickDone);
        column![
            Self::group(
                "Clamped text",
                "A long text is clamped; expanding it pushes what is below down. The pencil swaps it for an entry.",
                m,
            )
            .push(w::card(column![bio, text("Content below the text").size(12)].spacing(m.space_m))),
            Self::group("Empty", "The placeholder is the edit button.", m).push(w::card(nick)),
        ]
        .spacing(m.group_spacing)
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
            .primary(w::suggested_circular_button(Icon::Play.view(m.icon_size)).on_press(Msg::Noop))
            .action(w::icon_button(Icon::Heart).on_press(Msg::Noop))
            .action(w::icon_button(Icon::ViewMore).on_press(Msg::Noop))
            .end(w::flat_button(text("Share")).on_press(Msg::Noop))
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
            w::scrolled_window(body).width(Length::Fill).into()
        });
        Self::group(
            "Icons",
            "The built-in symbolic icons, revealed a row at a time. Anything that takes an icon takes an `Icon` or your own SVG (`IconSource`).",
            m,
        )
        .push(w::card(container(grid).height(Length::Fixed(380.0))))
        .into()
    }

    pub fn media_view(&self, m: Metrics) -> Element<'_, Msg> {
        let row_h = if self.compact { 36.0 } else { 56.0 };
        let compact = self.compact;
        // The columns, written once: the header and every row take their widths from them.
        let cols = MediaColumns::new()
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
                .cell(iad::widget::typography::caption(t.album))
                .cell(iad::widget::typography::caption(time))
                .suffix(w::icon_button(Icon::ViewMore).on_press(Msg::Noop))
                .on_press(Msg::Noop);
            if !compact {
                r = r.picture(
                    Avatar::new(w::PICTURE_WIDTH, t.title.clone())
                        .icon(Icon::MusicNote)
                        .shape(AvatarShape::Rounded),
                );
            }
            r.into()
        })
        .on_scroll(Msg::Scroll2);
        let header = cols.header(&self.sort, Msg::Sort);
        column![
            Self::group(
                "Media list",
                "A virtual list of media rows under a sort header. The columns are written once and shared by the header and the rows; the time is right-aligned.",
                m,
            )
            .push(w::SwitchRow::new("Compact", self.compact, Msg::Compact).metrics(m)),
            w::card(
                column![header, container(list).height(Length::Fixed(320.0))].spacing(m.space_xs),
            ),
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
        let cards = (0..9).map(|i| {
            let name = ALBUMS[i % ALBUMS.len()];
            w::MediaCard::new(format!("{name} {}", i + 1))
                .subtitle(ARTISTS[i % ARTISTS.len()])
                .picture(
                    Avatar::new(if compact { 36.0 } else { 72.0 }, format!("{name}{i}"))
                        .icon(Icon::MusicNote)
                        .shape(AvatarShape::Rounded),
                )
                .compact(compact)
                .on_press(Msg::Noop)
                .into()
        });
        column![
            SectionHeading::new("Recently played")
                .subtitle("A media card is a card, or a tight row when compact")
                .link("View all", Msg::Noop),
            w::SwitchRow::new("Compact", compact, Msg::CardsCompact).metrics(m),
            CardGrid::new(cards).max_width(180.0).compact(compact),
        ]
        .spacing(m.space_m)
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
            w::icon_button(Icon::Delete).on_press(Msg::Remove(i)).into()
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
        let btn = |label: &'static str, msg: Msg| w::raised_button(text(label)).on_press(msg);
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
                    w::suggested_button(text("Back"))
                        .on_press_maybe(self.history.can_back().then_some(Msg::Back)),
                ]
                .spacing(m.space_s)
                .wrap(),
            ]
            .spacing(m.space_m),
        ))
        .into()
    }

    /// `MetaComponent::flow`: the variant is built without measuring the container, so it takes
    /// its content's height and what is below follows it.
    pub fn flow_view(&self, m: Metrics) -> Element<'_, Msg> {
        #[derive(Clone, Copy, PartialEq)]
        enum Mode {
            Entry,
            Text,
        }
        let text_value = self.flow_text.clone();
        let flowing = w::MetaComponent::new()
            .selected(self.flow_editing.then_some(Mode::Entry))
            .when_selected(Mode::Entry, {
                let v = self.flow_text.clone();
                move |_| w::entry("Text", v.clone()).on_input(Msg::FlowText).metrics(m).into()
            })
            .variant(Mode::Text, w::When::always(), move |_| {
                iad::widget::typography::body(
                    format!("{text_value} It is long enough to wrap onto a second line when the pane is narrow, and the height follows it."),
                )
                .into()
            })
            .flow(0.0);
        Self::group(
            "MetaComponent::flow",
            "The usual conversion fills its container (it is built on responsive). `.flow(width)` builds the chosen variant as it is, so it takes its content's size and the line below moves with it.",
            m,
        )
        .push(w::SwitchRow::new("Editing", self.flow_editing, Msg::Flow).metrics(m))
        .push(w::card(
            column![flowing, text("Content below the variant").size(12)].spacing(m.space_m),
        ))
        .into()
    }

    /// The page surfaces: the gutter is in one place (`Metrics::gutter`).
    pub fn surface_view(&self, m: Metrics) -> Element<'_, Msg> {
        use iad::widget::surface;
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
