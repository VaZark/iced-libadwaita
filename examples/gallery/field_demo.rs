//! The Field page: every control type wrapped in a `Field`, live-validated, plus a static
//! matrix of states (pristine / valid / warning / invalid / disabled / required) and a
//! submit button. The touched / submitted bookkeeping is one `FormState` and one `FormSpec`:
//! the way to use forms (see docs/lib/components.md, "How to use forms").

use iced_libadwaita as adw;

use adw::primitives::form::{FormSpec, FormState, Outcome};
use adw::widget::validation::{chain, required};
use adw::widget::{self as w, Field, FieldLayout, Validity, typography as ty};
use adw::{Element, Metrics};
use iced::Length;
use iced::widget::{column, container, row};

const FRUITS: [&str; 3] = ["Apple", "Banana", "Cherry"];
const FRUITS5: [&str; 5] = ["Apple", "Banana", "Cherry", "Damson", "Elderberry"];

#[derive(Debug, Clone)]
pub enum Msg {
    Name(String),
    Password(String),
    Bio(w::TextAction),
    Fruit(&'static str),
    Multi(Vec<&'static str>),
    Date(String),
    Agree(bool),
    Plan(u8),
    Notify(bool),
    Volume(f32),
    Submit,
    Reset,
    Noop,
}

pub struct State {
    name: String,
    password: String,
    bio: w::TextContent,
    fruit: Option<&'static str>,
    multi: Vec<&'static str>,
    date: String,
    agree: bool,
    plan: Option<u8>,
    notify: bool,
    volume: f32,
    /// What the user touched or submitted: the only interaction state the page keeps.
    form: FormState<&'static str>,
}

/// The form's field keys: one single-page step.
const KEYS: [&str; 9] = [
    "name", "password", "bio", "fruit", "multi", "date", "agree", "plan", "volume",
];

fn spec() -> FormSpec<&'static str> {
    FormSpec::single(KEYS)
}

impl Default for State {
    fn default() -> Self {
        Self {
            name: String::new(),
            password: String::new(),
            bio: w::TextContent::new(),
            fruit: None,
            multi: vec!["Apple"],
            date: String::from("2026-11-02"),
            agree: false,
            plan: None,
            notify: false,
            volume: 10.0,
            form: FormState::new(),
        }
    }
}

fn name_rules(s: &str) -> Validity {
    chain(
        s,
        &[
            |s| required(s, "Enter your name."),
            |s| {
                if s.len() < 3 {
                    Validity::warning("That looks short.")
                } else {
                    Validity::Ok
                }
            },
        ],
    )
}

fn password_rules(s: &str) -> Validity {
    chain(
        s,
        &[
            |s| required(s, "Choose a password."),
            |s| {
                if s.len() < 6 {
                    Validity::error("Use at least 6 characters.")
                } else if s.len() < 10 {
                    Validity::warning("Longer is safer.")
                } else {
                    Validity::Ok
                }
            },
        ],
    )
}

impl State {
    fn touch(&mut self, k: &'static str) {
        self.form.touch(k);
    }

    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Name(v) => {
                self.name = v;
                self.touch("name");
            }
            Msg::Password(v) => {
                self.password = v;
                self.touch("password");
            }
            Msg::Bio(a) => {
                let edit = a.is_edit();
                self.bio.perform(a);
                if edit {
                    self.touch("bio");
                }
            }
            Msg::Fruit(v) => {
                self.fruit = Some(v);
                self.touch("fruit");
            }
            Msg::Multi(v) => {
                self.multi = v;
                self.touch("multi");
            }
            Msg::Date(v) => {
                self.date = v;
                self.touch("date");
            }
            Msg::Agree(v) => {
                self.agree = v;
                self.touch("agree");
            }
            Msg::Plan(v) => {
                self.plan = Some(v);
                self.touch("plan");
            }
            Msg::Notify(v) => {
                self.notify = v;
                self.touch("notify");
            }
            Msg::Volume(v) => {
                self.volume = v;
                self.touch("volume");
            }
            Msg::Submit => match self.submit_form() {
                // Every error now shows; a real app would also send the values here.
                Outcome::Submitted => {}
                // A real app would focus `key` (the toolkit entries take no widget id yet).
                Outcome::Blocked { .. } | Outcome::Moved { .. } | Outcome::Ignored => {}
            },
            Msg::Reset => {
                // The form forgets what was touched; the app resets its own values.
                self.form.reset();
                let form = std::mem::take(&mut self.form);
                *self = Self {
                    form,
                    ..Self::default()
                };
            }
            Msg::Noop => {}
        }
    }

    /// Opens the page already submitted (screenshots).
    pub fn submit(&mut self) {
        self.submit_form();
    }

    fn submit_form(&mut self) -> Outcome<&'static str> {
        let mut form = std::mem::take(&mut self.form);
        let outcome = form.submit(&spec(), &|k| self.validity(k));
        self.form = form;
        outcome
    }

    /// The validity of the field `key`: all the form needs from the app's values.
    fn validity(&self, key: &&'static str) -> Validity {
        match *key {
            "name" => name_rules(&self.name),
            "password" => password_rules(&self.password),
            "bio" => required(&self.bio.text(), "Tell us a little."),
            "fruit" => self
                .fruit
                .map_or(Validity::error("Pick a fruit."), |_| Validity::Ok),
            "multi" if self.multi.len() < 2 => Validity::error("Pick at least two."),
            "date" => w::validate_date(&self.date, None, None),
            "agree" if !self.agree => Validity::error("You must agree to continue."),
            "plan" => self
                .plan
                .map_or(Validity::error("Choose a plan."), |_| Validity::Ok),
            "volume" if self.volume > 80.0 => Validity::warning("That is loud."),
            _ => Validity::Ok,
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let spec = spec();
        // Each field: its validity and its interaction, both by key.
        fn mk<'a>(
            s: &State,
            spec: &FormSpec<&'static str>,
            m: Metrics,
            label: &'static str,
            k: &'static str,
            c: Element<'a, Msg>,
        ) -> Field<'a, Msg> {
            Field::new(label, c)
                .validity(s.validity(&k))
                .interaction(s.form.interaction(spec, &k))
                .metrics(m)
        }
        let f = |label, k, c| mk(self, &spec, m, label, k, c);

        let live = column![
            f(
                "Name",
                "name",
                w::entry("Ada", &self.name)
                    .on_input(Msg::Name)
                    .metrics(m)
                    .into()
            )
            .required(true)
            .description("As shown on your profile."),
            f(
                "Password",
                "password",
                w::password_entry("Password", &self.password)
                    .on_input(Msg::Password)
                    .metrics(m)
                    .into()
            )
            .required(true),
            f(
                "Bio",
                "bio",
                w::text_view(&self.bio)
                    .on_action(Msg::Bio)
                    .placeholder("About you")
                    .into()
            )
            .required(true),
            f(
                "Favorite fruit",
                "fruit",
                w::dropdown(&FRUITS, self.fruit, Msg::Fruit).into()
            )
            .required(true),
            f(
                "Toppings",
                "multi",
                w::MultiSelect::selecting(
                    &FRUITS5,
                    &self.multi,
                    w::Selection::Multiple,
                    Msg::Multi
                )
                .into()
            )
            .description("At least two."),
            f(
                "Birthday",
                "date",
                w::DateInput::new(&self.date, w::Date::new(2026, 11, 1).unwrap(), Msg::Date)
                    .revealed(self.form.revealed(&spec, &"date"))
                    .into()
            ),
            f(
                "Terms",
                "agree",
                w::check_button("I agree to the terms", self.agree)
                    .on_toggle(Msg::Agree)
                    .into()
            )
            .inline(true)
            .required(true),
            f(
                "Plan",
                "plan",
                row![
                    w::radio_button("Free", 0u8, self.plan, Msg::Plan),
                    w::radio_button("Pro", 1u8, self.plan, Msg::Plan),
                ]
                .spacing(sp)
                .into()
            )
            .inline(true)
            .required(true),
            f(
                "Volume",
                "volume",
                w::scale(0.0..=100.0, self.volume, Msg::Volume).into()
            )
            .inline(true),
            Field::new(
                "Notifications",
                w::switch(self.notify).on_toggle(Msg::Notify)
            )
            .metrics(m)
            .layout(FieldLayout::Row)
            .description("Row layout: label at the start, control at the end."),
            row![
                w::suggested_button(ty::label("Submit")).on_press(Msg::Submit),
                w::flat_button(ty::label("Reset")).on_press(Msg::Reset),
            ]
            .spacing(sp),
        ]
        .spacing(sp * 1.5);

        // Static matrix: the same entry in each state.
        let bad = Validity::error("This value is not allowed.");
        let warn = Validity::warning("This works, but check it.");
        let cell = |label: &'static str, value: &'static str| {
            Field::new(
                label,
                w::entry("", value).on_input(|_| Msg::Noop).metrics(m),
            )
            .metrics(m)
        };
        let states = column![
            cell("Pristine (invalid value, untouched)", "nope")
                .validity(bad.clone())
                .description("Nothing is shown until the field is touched."),
            cell("Valid (nothing special is drawn)", "ada@example.com").touched(true),
            cell("Valid, opted in with show_valid", "ada@example.com")
                .touched(true)
                .show_valid(true),
            cell("Warning", "ada").validity(warn).touched(true),
            cell("Invalid", "nope").validity(bad.clone()).touched(true),
            cell("Required", "")
                .required(true)
                .description("Marked with an asterisk."),
            cell("Invalid after submit", "nope")
                .validity(bad.clone())
                .submitted(true),
            Field::new("Disabled", w::entry("", "read only").metrics(m))
                .disabled(true)
                .required(true)
                .validity(bad)
                .touched(true)
                .description("Validation is hidden while disabled.")
                .metrics(m),
            Field::new(
                "Row layout",
                w::entry("", "nope").on_input(|_| Msg::Noop).metrics(m)
            )
            .layout(FieldLayout::Row)
            .validity(Validity::error("Row layout shows the message below."))
            .touched(true)
            .metrics(m),
        ]
        .spacing(sp * 1.5);

        let g = |t: &'static str| w::PreferencesGroup::<Msg>::new().title(t).metrics(m);
        let states_card =
            g("States").push(w::card(container(states).padding(sp).width(Length::Fill)));
        // GALLERY_FIELD=states: the states only (a screenshot is only one window tall).
        if std::env::var("GALLERY_FIELD").as_deref() == Ok("states") {
            return states_card.into();
        }
        column![
            g("Live form")
                .description("Fields turn valid or invalid after the first edit, or on Submit.")
                .push(w::card(container(live).padding(sp).width(Length::Fill))),
            states_card,
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
