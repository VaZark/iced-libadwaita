//! Interactive application entries, all composed from EntryRow and InputWrapper.
use iad::primitives::{
    calendar::CalendarView,
    date::Date,
    numeric::Decimal,
    validation::{Interaction, Validity},
};
use iad::{Element, EntryIcon, Metrics, components as cmp, widgets as w};
use iced::widget::{column, text, text_editor};
use iced_adaptive_design as iad;

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
            notes: text_editor::Content::with_text(
                "Write a description. This editor grows as you type or paste more lines.",
            ),
            single_line: text_editor::Content::new(),
            touched: false,
            status: "Ready".into(),
        }
    }
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
            Msg::Notes(action) => self.notes.perform(action),
            Msg::SingleLine(action) => self.single_line.perform(action),
            Msg::Action => self.status = "Action completed".into(),
            Msg::Navigate => self.status = "Navigation destination opened".into(),
        }
    }
    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let group = |title| w::PreferencesGroup::new().title(title).metrics(m);
        let date = |modal: bool| {
            let i = usize::from(modal);
            w::DateEntry::from_input(
                if modal {
                    "Date · modal"
                } else {
                    "Date · dropdown"
                },
                w::DateInput::new(&self.dates[i], self.calendars[i].date, move |v| {
                    Msg::Date(modal, v)
                })
                .calendar(self.calendars[i])
                .selected(Date::parse(&self.dates[i]))
                .on_pick(move |d| Msg::Pick(modal, d))
                .on_nav(move |v| Msg::Calendar(modal, v))
                .dialog(modal)
                .metrics(m),
            )
            .metrics(m)
        };
        column![
            group("Flat entries")
                .description("Transparent page rows, like Explore navigation. Typing stays available; the date modal is a picking helper.")
                .flat(true)
                .push(w::TextEntry::new("Display name", &self.name, Msg::Name).flat(true).metrics(m))
                .push(w::NumberEntry::new("Amount", &self.number, Msg::Number).flat(true).metrics(m))
                .push(date(true).flat(true))
                .push(w::SelectEntry::new("Team", &OPTIONS, self.selected, Msg::Select).flat(true).metrics(m))
                .push(w::SwitchEntry::new("Notifications", self.enabled, Msg::Enabled).flat(true).metrics(m)),
            group("Text entries")
                .description(
                    "Full-width application controls. Clear the name to reveal validation."
                )
                .push(
                    w::TextEntry::new("Display name", &self.name, Msg::Name)
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
                    w::TextEntry::from_input(
                        "Password",
                        cmp::password_entry("Password", &self.password)
                            .on_input(Msg::Password)
                            .metrics(m)
                    )
                    .metrics(m)
                )
                .push(
                    w::TextEntry::from_input(
                        "Prefix and suffix",
                        cmp::entry("Name", &self.name)
                            .on_input(Msg::Name)
                            .left_section(EntryIcon::Search.view(m.icon_size))
                            .right_section(text("@app"))
                            .right_section_width(52.0)
                            .metrics(m)
                    )
                    .metrics(m)
                ),
            group("Number, date and switch entries")
                .push(
                    w::NumberEntry::from_input(
                        "Amount",
                        w::NumberInput::new(&self.number, Msg::Number)
                            .decimals(2)
                            .min(0)
                            .max(100)
                            .step(Decimal::new(25, 2))
                            .metrics(m)
                    )
                    .metrics(m)
                )
                .push(date(false))
                .push(date(true))
                .push(w::SwitchEntry::new("Notifications", self.enabled, Msg::Enabled).metrics(m)),
            group("Selection entries")
                .description("Select anywhere in the input area; values start at the left edge.")
                .push(w::SelectEntry::new("Team", &OPTIONS, self.selected, Msg::Select).metrics(m))
                .push(
                    w::MultiselectEntry::new(
                        "Responsibilities",
                        &OPTIONS,
                        &self.multiple,
                        Msg::Toggle
                    )
                    .metrics(m)
                )
                .push(
                    w::ChipGroupEntry::new(
                        "Tags",
                        OPTIONS.iter().map(|v| w::ChipOption::new(*v, *v)),
                        &self.chips,
                        Msg::Chip
                    )
                    .metrics(m)
                ),
            group("Growing text")
                .push(w::MultilineEntry::new("Description", &self.notes, Msg::Notes).metrics(m))
                .push(
                    w::MultilineEntry::from_input(
                        "Single-line editor",
                        cmp::text_view(&self.single_line)
                            .placeholder("Enter is ignored; pasted line breaks become spaces")
                            .single_line(true)
                            .on_action(Msg::SingleLine)
                    )
                    .metrics(m)
                ),
            group("Actions and navigation")
                .push(
                    w::ActionEntry::new("Status", &self.status, EntryIcon::Action, Msg::Action)
                        .metrics(m)
                )
                .push(
                    w::NavigationRow::new("Open destination", Msg::Navigate)
                        .subtitle("Navigation has its own row; value actions stay in ActionEntry.")
                        .metrics(m)
                ),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
