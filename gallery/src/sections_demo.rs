//! Entry sections: left and right slots inside an entry, and a Nautilus-style search with a
//! filter popover whose button is the right section.

use iced_adaptive_design as iad;

use iad::components::typography as ty;
use iad::{Element, Icon, Metrics, Role};
use iad::{components as cmp, metas as met, widgets as w};
use iced::widget::{column, container, row, text};
use iced::{Length, Padding};

fn dim<'a>(c: impl iced::widget::text::IntoFragment<'a>) -> iced::widget::Text<'a, iad::Theme> {
    ty::caption(c).style(iad::styles::text::dim)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    Any,
    Today,
    Yesterday,
    Week,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Any,
    Folder,
    Document,
    Image,
    Audio,
    Video,
}

impl When {
    const ALL: [When; 4] = [When::Any, When::Today, When::Yesterday, When::Week];
    fn label(self) -> &'static str {
        match self {
            When::Any => "Any time",
            When::Today => "Today",
            When::Yesterday => "Yesterday",
            When::Week => "Last 7 days",
        }
    }
    /// Age limit in days (inclusive range).
    fn matches(self, age: u32) -> bool {
        match self {
            When::Any => true,
            When::Today => age == 0,
            When::Yesterday => age == 1,
            When::Week => age <= 7,
        }
    }
}

impl Kind {
    const ALL: [Kind; 6] = [
        Kind::Any,
        Kind::Folder,
        Kind::Document,
        Kind::Image,
        Kind::Audio,
        Kind::Video,
    ];
    fn label(self) -> &'static str {
        match self {
            Kind::Any => "Any type",
            Kind::Folder => "Folder",
            Kind::Document => "Document",
            Kind::Image => "Image",
            Kind::Audio => "Audio",
            Kind::Video => "Video",
        }
    }
}

/// (name, kind, age in days)
const FILES: [(&str, Kind, u32); 9] = [
    ("Holiday photos", Kind::Folder, 12),
    ("budget-2026.ods", Kind::Document, 0),
    ("meeting-notes.txt", Kind::Document, 1),
    ("holiday-beach.png", Kind::Image, 3),
    ("logo.svg", Kind::Image, 30),
    ("podcast-ep12.mp3", Kind::Audio, 6),
    ("demo-reel.mp4", Kind::Video, 0),
    ("Documents", Kind::Folder, 90),
    ("holiday-plans.odt", Kind::Document, 20),
];

#[derive(Debug, Clone)]
pub enum Msg {
    Left(String),
    Right(String),
    Both(String),
    Clearable(String),
    Clear,
    Search(String),
    When(When),
    Kind(Kind),
    Reset,
    ToggleSearch,
    Noop,
}

pub struct State {
    left: String,
    right: String,
    both: String,
    clearable: String,
    search: String,
    when: When,
    kind: Kind,
    searching: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            left: String::new(),
            right: String::new(),
            both: String::new(),
            clearable: String::from("type, then clear"),
            search: String::new(),
            when: When::Any,
            kind: Kind::Any,
            searching: std::env::var("GALLERY_OPEN").is_ok_and(|v| v.starts_with("meta-search")),
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Left(v) => self.left = v,
            Msg::Right(v) => self.right = v,
            Msg::Both(v) => self.both = v,
            Msg::Clearable(v) => self.clearable = v,
            Msg::Clear => self.clearable.clear(),
            Msg::Search(v) => self.search = v,
            Msg::When(v) => self.when = v,
            Msg::Kind(v) => self.kind = v,
            Msg::Reset => {
                self.when = When::Any;
                self.kind = Kind::Any;
            }
            Msg::ToggleSearch => self.searching = !self.searching,
            Msg::Noop => {}
        }
    }

    /// Escape: back from the search bar to the path bar.
    pub fn leave_search(&mut self) {
        self.searching = false;
    }

    fn filtered(&self) -> bool {
        self.when != When::Any || self.kind != Kind::Any
    }

    fn results(&self) -> Vec<(&'static str, Kind, u32)> {
        let q = self.search.to_lowercase();
        FILES
            .iter()
            .copied()
            .filter(|(n, k, age)| {
                n.to_lowercase().contains(&q)
                    && self.when.matches(*age)
                    && (self.kind == Kind::Any || self.kind == *k)
            })
            .collect()
    }

    fn filter_popover(&self) -> Element<'_, Msg> {
        let m = Metrics::desktop();
        let head = |t: &'static str| -> Element<'_, Msg> {
            container(dim(t)).padding([m.space_xs, m.space_s]).into()
        };
        let mut items: Vec<Element<'_, Msg>> = vec![head("When")];
        for v in When::ALL {
            items.push(radio_row(v.label(), v, Some(self.when), Msg::When));
        }
        items.push(head("What"));
        for v in Kind::ALL {
            items.push(radio_row(v.label(), v, Some(self.kind), Msg::Kind));
        }
        items.push(
            container(
                iced::widget::button(text("Reset").width(Length::Fill).center())
                    .padding(m.space_s)
                    .width(Length::Fill)
                    .style(iad::styles::button::default)
                    .on_press_maybe(self.filtered().then_some(Msg::Reset)),
            )
            .padding(Padding::new(m.space_xs).top(m.space_s))
            .into(),
        );
        w::popover(items, 440.0)
    }

    /// The search entry: `left` section, and the filter button (with its popover) as the right
    /// section.
    fn search_entry<'a>(&'a self, m: Metrics, left: Element<'a, Msg>) -> Element<'a, Msg> {
        let filter = w::Popover::new(
            cmp::SectionButton::<Msg>::face(Icon::Filter, self.filtered()),
            self.filter_popover(),
        )
        .appearance(w::PopoverAppearance::Flat)
        .padding(0)
        .align(w::PopoverAlign::End)
        .keep_open(true)
        .popover_width(220.0)
        .open_initially(std::env::var("GALLERY_OPEN").as_deref() == Ok("filter"));
        cmp::entry("Search files", &self.search)
            .on_input(Msg::Search)
            .left_section(left)
            .right_section(filter)
            .metrics(m)
            .into()
    }

    /// The active-filter chips and the filtered file list.
    fn results_view(&self, m: Metrics) -> Element<'_, Msg> {
        let mut chips = row![].spacing(m.space_s);
        if self.when != When::Any {
            chips = chips.push(
                cmp::Chip::new(self.when.label())
                    .role(Role::Accent)
                    .on_remove(Msg::When(When::Any)),
            );
        }
        if self.kind != Kind::Any {
            chips = chips.push(
                cmp::Chip::new(self.kind.label())
                    .role(Role::Accent)
                    .on_remove(Msg::Kind(Kind::Any)),
            );
        }
        let results = self.results();
        let mut list = column![].spacing(m.space_xs);
        if results.is_empty() {
            list = list.push(dim("No files match."));
        }
        for (name, kind, age) in results {
            let today = iad::primitives::date::Date::today();
            let when = iad::primitives::Relative::from_dates(today.add_days(-(age as i64)), today);
            list = list.push(
                row![
                    text(name).width(Length::Fill),
                    dim(format!("{} · {when}", kind.label())),
                ]
                .spacing(m.space_m),
            );
        }
        let mut body = column![].spacing(m.space_m);
        if self.filtered() {
            body = body.push(chips);
        }
        body.push(list).into()
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let g = |t: &'static str| w::PreferencesGroup::<Msg>::new().title(t).metrics(m);
        let pad = |c| padded(c, sp);
        let search_icon = || cmp::section_icon(Icon::Search).metrics(m);
        let clear: Element<'_, Msg> = if self.clearable.is_empty() {
            iced::widget::space().into()
        } else {
            cmp::section_button(Icon::WindowClose, Msg::Clear)
                .metrics(m)
                .into()
        };

        let sections = pad(column![
            cmp::entry("Left icon", &self.left)
                .on_input(Msg::Left)
                .left_section(search_icon())
                .metrics(m),
            cmp::entry("Right icon", &self.right)
                .on_input(Msg::Right)
                .right_section(cmp::section_icon(Icon::Edit).metrics(m))
                .metrics(m),
            cmp::entry("Both sides", &self.both)
                .on_input(Msg::Both)
                .left_section(search_icon())
                .right_section(cmp::section_icon(Icon::Settings).metrics(m))
                .metrics(m),
            cmp::entry("Clear button", &self.clearable)
                .on_input(Msg::Clearable)
                .left_section(search_icon())
                .right_section(clear)
                .metrics(m),
            cmp::entry("Unit label", "72")
                .on_input(|_| Msg::Noop)
                .right_section(dim("kg"))
                .right_section_width(28.0)
                .metrics(m),
            cmp::entry("Disabled", "read only")
                .left_section(cmp::section_icon(Icon::Search).disabled(true).metrics(m))
                .right_section(
                    cmp::section_button(Icon::WindowClose, Msg::Noop)
                        .disabled(true)
                        .metrics(m)
                )
                .metrics(m),
            cmp::entry("Invalid", "not a valid value")
                .on_input(|_| Msg::Noop)
                .state(cmp::EntryState::Error)
                .left_section(search_icon())
                .right_section(cmp::section_icon(Icon::DialogError).metrics(m))
                .metrics(m),
        ]
        .spacing(sp));

        // Search with filters.
        let search = self.search_entry(m, search_icon().into());

        let body = column![search, self.results_view(m)].spacing(sp);

        column![
            g("Search with filters").push(pad(body)),
            g("Sections").push(sections),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}

fn radio_row<'a, V: Copy + Eq + 'a>(
    label: &'static str,
    value: V,
    selected: Option<V>,
    f: fn(V) -> Msg,
) -> Element<'a, Msg> {
    container(cmp::radio_button(label, value, selected, f))
        .padding([
            iad::Metrics::desktop().space_xs,
            iad::Metrics::desktop().space_s,
        ])
        .width(Length::Fill)
        .into()
}

fn padded<'a>(c: iced::widget::Column<'a, Msg, iad::Theme>, _sp: f32) -> Element<'a, Msg> {
    w::card(c)
}

// ------------------------------------------------------------ MetaComponent demos

#[derive(Clone, PartialEq)]
enum Bar {
    Search,
    Path,
}

#[derive(Clone, PartialEq)]
enum Layout {
    Table,
    Cards,
}

impl State {
    /// The "Meta component" page: a responsive trigger and an on-select trigger.
    pub fn meta_view(&self, m: Metrics) -> Element<'_, Msg> {
        let g = |t: &'static str| w::PreferencesGroup::<Msg>::new().title(t).metrics(m);
        let people = |width: f32| -> Element<'_, Msg> {
            let table = met::MetaComponent::new()
                .variant(
                    Layout::Table,
                    iad::primitives::When::min_width(380.0),
                    move |_| {
                        let mut rows = column![
                            row![
                                ty::caption_heading("Name").width(Length::FillPortion(2)),
                                ty::caption_heading("Role").width(Length::FillPortion(2)),
                                ty::caption_heading("Email").width(Length::FillPortion(3)),
                            ]
                            .spacing(8)
                        ]
                        .spacing(10)
                        .padding(12);
                        for p in crate::PEOPLE.iter() {
                            rows = rows.push(
                                row![
                                    ty::heading(p.name).width(Length::FillPortion(2)),
                                    ty::body(p.role).width(Length::FillPortion(2)),
                                    ty::body(p.email).width(Length::FillPortion(3)),
                                ]
                                .spacing(8),
                            );
                        }
                        w::card(rows)
                    },
                )
                .variant(Layout::Cards, iad::primitives::When::always(), |_| {
                    let mut cards = column![].spacing(8).padding(4);
                    for p in crate::PEOPLE.iter() {
                        cards = cards.push(w::card(
                            column![
                                row![
                                    ty::heading(p.name),
                                    cmp::Badge::new(p.role).role(Role::Accent)
                                ]
                                .spacing(8)
                                .align_y(iced::Alignment::Center),
                                ty::caption(p.email),
                            ]
                            .spacing(4)
                            .padding(8),
                        ));
                    }
                    iced::widget::scrollable(cards).into()
                });
            container(Element::from(table))
                .width(Length::Fixed(width))
                .height(Length::Fixed(240.0))
                .into()
        };
        let sides = row![people(400.0), people(270.0)].spacing(m.space_m).wrap();

        let toggle = cmp::section_button(Icon::Search, Msg::ToggleSearch)
            .active(self.searching)
            .metrics(m);
        let bar = met::MetaComponent::new()
            .selected(self.searching.then_some(Bar::Search))
            .when_selected(Bar::Search, move |_| {
                let back = cmp::section_button(Icon::GoPrevious, Msg::ToggleSearch).metrics(m);
                container(self.search_entry(m, back.into()))
                    .center_y(Length::Fill)
                    .into()
            })
            .variant(Bar::Path, iad::primitives::When::always(), |_| {
                container(w::path_bar(
                    ["Home", "Projects", "iced-adaptive-design", "docs"]
                        .map(String::from)
                        .to_vec(),
                    3,
                    |_| Msg::Noop,
                ))
                .center_y(Length::Fill)
                .into()
            })
            .height(Length::Fixed(44.0));
        let header = row![Element::from(bar), Element::from(toggle)]
            .spacing(m.space_s)
            .align_y(iced::Alignment::Center);
        let search = column![
            header,
            if self.searching {
                self.results_view(m)
            } else {
                dim("Press the magnifier to search this folder.").into()
            }
        ]
        .spacing(m.space_m);

        column![
            g("On-select trigger")
                .description("The path bar becomes a search entry when the app selects it; the back arrow or the magnifier returns.")
                .push(w::card(container(search).padding(m.space_m))),
            g("Responsive trigger")
                .description("A table while 380 px fit, cards below: the container decides, not the window.")
                .push(sides),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
