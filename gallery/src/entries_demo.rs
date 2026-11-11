//! Interactive application entries, all composed from EntryRow and InputWrapper.
use iad::primitives::{
    calendar::CalendarView,
    date::Date,
    draft::Draft,
    numeric::Decimal,
    tag_groups::{GroupEvent, TagGroups},
    validation::{Interaction, Validity},
};
use iad::{Element, EntryIcon, Metrics, components as cmp, widgets as w};
use iced::Length;
use iced::widget::{column, container, row, text, text_editor};
use iced_adaptive_design as iad;

/// Tags with their own icon and color, as a tag vocabulary has.
const GENRES: [(&str, iad::Icon, iced::Color); 5] = [
    (
        "rock",
        iad::Icon::Bolt,
        iced::Color::from_rgb(0.75, 0.22, 0.17),
    ),
    (
        "jazz",
        iad::Icon::MusicNote,
        iced::Color::from_rgb(0.2, 0.52, 0.89),
    ),
    (
        "calm",
        iad::Icon::Moon,
        iced::Color::from_rgb(0.33, 0.45, 0.6),
    ),
    (
        "latin",
        iad::Icon::Heart,
        iced::Color::from_rgb(0.9, 0.38, 0.0),
    ),
    (
        "energetic",
        iad::Icon::Star,
        iced::Color::from_rgb(0.6, 0.45, 0.0),
    ),
];
/// Sample tracks the tag filters are tried on.
const TRACKS: [(&str, &[&str]); 4] = [
    ("Blue Train", &["jazz", "calm"]),
    ("Back in Black", &["rock", "energetic"]),
    ("Ojos Asi", &["latin", "energetic"]),
    ("Kind of Blue", &["jazz", "calm", "latin"]),
];
const OPTIONS: [&str; 4] = ["Design", "Development", "Documentation", "Testing"];

#[derive(Debug, Clone)]
pub enum Msg {
    Name(String),
    Password(String),
    Number(String),
    Enabled(bool),
    Date(bool, String),
    Pick(bool, Date),
    Calendar(bool, CalendarView),
    Select(&'static str),
    Toggle(&'static str),
    Chip(&'static str),
    /// `false`: the single-group filter; `true`: the multiple-group one.
    Tags(bool, GroupEvent<&'static str>),
    TagQuery(bool, String),
    TagsReset(bool),
    Notes(text_editor::Action),
    SingleLine(text_editor::Action),
    Action,
    Navigate,
}

pub struct State {
    name: String,
    password: String,
    number: String,
    enabled: bool,
    dates: [String; 2],
    calendars: [CalendarView; 2],
    selected: Option<&'static str>,
    multiple: Vec<&'static str>,
    chips: Vec<&'static str>,
    tags: [Draft<TagGroups<&'static str>>; 2],
    tag_queries: [String; 2],
    notes: text_editor::Content,
    single_line: text_editor::Content,
    touched: bool,
    status: String,
}
impl Default for State {
    fn default() -> Self {
        let today = Date::today();
        Self {
            name: "Ada Lovelace".into(),
            password: "analytical".into(),
            number: "12.50".into(),
            enabled: true,
            dates: [today.format(), today.format()],
            calendars: [CalendarView::new(today, Default::default()); 2],
            selected: Some("Design"),
            multiple: vec!["Design", "Development"],
            chips: vec!["Documentation"],
            tags: sample_tags(),
            tag_queries: Default::default(),
            notes: text_editor::Content::with_text(
                "Write a description. This editor grows as you type or paste more lines.",
            ),
            single_line: text_editor::Content::new(),
            touched: false,
            status: "Ready".into(),
        }
    }
}
/// A single filter with one tag picked, and a multiple one with two groups.
fn sample_tags() -> [Draft<TagGroups<&'static str>>; 2] {
    let mut single = TagGroups::new();
    single.toggle("rock");
    let mut multiple = TagGroups::new();
    multiple.toggle("jazz");
    multiple.toggle("calm");
    multiple.add_group();
    multiple.toggle("latin");
    [Draft::new(single), Draft::new(multiple)]
}
fn toggle(values: &mut Vec<&'static str>, value: &'static str) {
    if let Some(i) = values.iter().position(|v| *v == value) {
        values.remove(i);
    } else {
        values.push(value);
    }
}
impl State {
    pub fn update(&mut self, message: Msg) {
        match message {
            Msg::Name(value) => {
                self.name = value;
                self.touched = true;
            }
            Msg::Password(value) => self.password = value,
            Msg::Number(value) => self.number = value,
            Msg::Enabled(value) => self.enabled = value,
            Msg::Date(modal, value) => {
                let i = usize::from(modal);
                if let Some(date) = Date::parse(&value) {
                    self.calendars[i].date = date;
                }
                self.dates[i] = value;
            }
            Msg::Pick(modal, date) => {
                let i = usize::from(modal);
                self.dates[i] = date.format();
                self.calendars[i].date = date;
            }
            Msg::Calendar(modal, calendar) => self.calendars[usize::from(modal)] = calendar,
            Msg::Select(value) => self.selected = Some(value),
            Msg::Toggle(value) => toggle(&mut self.multiple, value),
            Msg::Chip(value) => toggle(&mut self.chips, value),
            Msg::Tags(multiple, event) => self.tags[usize::from(multiple)].apply(event),
            Msg::TagsReset(multiple) => {
                let i = usize::from(multiple);
                self.tags[i].reset();
                self.tag_queries[i].clear();
            }
            Msg::TagQuery(multiple, query) => self.tag_queries[usize::from(multiple)] = query,
            Msg::Notes(action) => self.notes.perform(action),
            Msg::SingleLine(action) => self.single_line.perform(action),
            Msg::Action => self.status = "Action completed".into(),
            Msg::Navigate => self.status = "Navigation destination opened".into(),
        }
    }
    fn tag_filter(&self, multiple: bool, m: Metrics) -> Element<'_, Msg> {
        let i = usize::from(multiple);
        let groups: &TagGroups<_> = &self.tags[i];
        let dirty = self.tags[i].is_dirty();
        let matching = TRACKS
            .iter()
            .filter(|(_, tags)| groups.matches(tags))
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(", ");
        let filter = column![
            w::TagFilter::new(
                groups,
                GENRES
                    .iter()
                    .map(|(g, icon, color)| w::ChipOption::new(*g, *g).icon(*icon).color(*color)),
                &self.tag_queries[i],
                move |e| Msg::Tags(multiple, e),
                move |q| Msg::TagQuery(multiple, q),
            )
            .multiple(multiple)
            .label(if multiple {
                "Tags (multiple groups)"
            } else {
                "Tags (single group)"
            })
            .description(if multiple {
                "An item needs every tag of at least one group. Press + to start another group."
            } else {
                "An item needs every tag you pick."
            })
            .metrics(m),
            row![
                text(format!("Matches: {matching}"))
                    .size(12)
                    .width(Length::Fill),
                cmp::flat_button(text("Reset"))
                    .on_press_maybe(dirty.then_some(Msg::TagsReset(multiple))),
            ]
            .align_y(iced::Alignment::Center),
        ]
        .spacing(6);
        container(filter).padding([8, 12]).into()
    }
    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let group = |title| w::PreferencesGroup::new().title(title).metrics(m);
        let date = |modal: bool| {
            let i = usize::from(modal);
            w::EntryRow::new(
                if modal {
                    "Date · modal"
                } else {
                    "Date · dropdown"
                },
                w::EntryInput::Date(
                    w::DateInput::new(&self.dates[i], self.calendars[i].date, move |v| {
                        Msg::Date(modal, v)
                    })
                    .calendar(self.calendars[i])
                    .selected(Date::parse(&self.dates[i]))
                    .on_pick(move |d| Msg::Pick(modal, d))
                    .on_nav(move |v| Msg::Calendar(modal, v))
                    .dialog(modal)
                    .metrics(m),
                ),
            )
            .metrics(m)
        };
        column![
            group("Flat entries")
                .description("Transparent page rows, like Explore navigation. Typing stays available; the date modal is a picking helper.")
                .flat(true)
                .push(w::EntryRow::text("Display name", &self.name, Msg::Name).flat(true).metrics(m))
                .push(w::EntryRow::number("Amount", &self.number, Msg::Number).flat(true).metrics(m))
                .push(date(true).flat(true))
                .push(w::EntryRow::select("Team", &OPTIONS, self.selected, Msg::Select).flat(true).metrics(m))
                .push(w::EntryRow::switch("Notifications", self.enabled, Msg::Enabled).flat(true).metrics(m)),
            group("Text entries")
                .description(
                    "Full-width application controls. Clear the name to reveal validation."
                )
                .push(
                    w::EntryRow::text("Display name", &self.name, Msg::Name)
                        .required(true)
                        .validity(if self.name.trim().is_empty() {
                            Validity::error("Enter a display name.")
                        } else {
                            Validity::Ok
                        })
                        .interaction(Interaction {
                            touched: self.touched,
                            submitted: false
                        })
                        .metrics(m)
                )
                .push(
                    w::EntryRow::new(
                        "Password",
                        w::EntryInput::Text(
                            cmp::password_entry("Password", &self.password)
                                .on_input(Msg::Password)
                                .metrics(m)
                        )
                    )
                    .metrics(m)
                )
                .push(
                    w::EntryRow::new(
                        "Prefix and suffix",
                        w::EntryInput::Text(
                            cmp::entry("Name", &self.name)
                                .on_input(Msg::Name)
                                .left_section(EntryIcon::Search.view(m.icon_size))
                                .right_section(text("@app"))
                                .right_section_width(52.0)
                                .metrics(m)
                        )
                    )
                    .metrics(m)
                ),
            group("Number, date and switch entries")
                .push(
                    w::EntryRow::new(
                        "Amount",
                        w::EntryInput::Number(
                            w::NumberInput::new(&self.number, Msg::Number)
                                .decimals(2)
                                .min(0)
                                .max(100)
                                .step(Decimal::new(25, 2))
                                .metrics(m)
                        )
                    )
                    .metrics(m)
                )
                .push(date(false))
                .push(date(true))
                .push(w::EntryRow::switch("Notifications", self.enabled, Msg::Enabled).metrics(m)),
            group("Selection entries")
                .description("Select anywhere in the input area; values start at the left edge.")
                .push(w::EntryRow::select("Team", &OPTIONS, self.selected, Msg::Select).metrics(m))
                .push(
                    w::EntryRow::multiselect(
                        "Responsibilities",
                        &OPTIONS,
                        &self.multiple,
                        Msg::Toggle
                    )
                    .metrics(m)
                )
                .push(
                    w::EntryRow::chips(
                        "Tags",
                        OPTIONS.iter().map(|v| w::ChipOption::new(*v, *v)),
                        &self.chips,
                        Msg::Chip
                    )
                    .metrics(m)
                ),
            group("Tag filters")
                .description(
                    "One group by default: a track needs every picked tag. With multiple(true), \
                     groups are alternatives, separated by commas: click a group, use the arrow keys at \
                     the edges of the field, or press + to start another.",
                )
                .push(self.tag_filter(false, m))
                .push(self.tag_filter(true, m)),
            group("Growing text")
                .push(w::EntryRow::multiline("Description", &self.notes, Msg::Notes).metrics(m))
                .push(
                    w::EntryRow::new(
                        "Single-line editor",
                        w::EntryInput::Multiline(
                            cmp::text_view(&self.single_line)
                                .placeholder("Enter is ignored; pasted line breaks become spaces")
                                .single_line(true)
                                .on_action(Msg::SingleLine)
                        )
                    )
                    .metrics(m)
                ),
            group("Actions and navigation")
                .push(
                    w::EntryRow::action("Status", &self.status, EntryIcon::Action, Msg::Action)
                        .metrics(m)
                )
                .push(
                    w::ActionRow::navigation("Open destination", Msg::Navigate)
                        .subtitle("Navigation has its own row; value actions stay in EntryRow::action.")
                        .metrics(m)
                ),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
