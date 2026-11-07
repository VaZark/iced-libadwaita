//! The Form page: a single-page form and a three-step wizard on `primitives::form`, plus a
//! static strip of the step-indicator states.
//!
//! Screenshot switches: `GALLERY_FORM=single|wizard|strip` shows one demo only,
//! `GALLERY_FORM_STEP=n` opens the wizard on step n (earlier steps pre-filled), and
//! `GALLERY_FORM_ATTEMPT=1` starts with the current step attempted (its errors showing).

use iced_adaptive_design as iad;

use iad::components::typography as ty;
use iad::primitives::form::{FormSpec, FormState, Outcome as FormOutcome, StepStatus};
use iad::primitives::validation::Validity;
use iad::primitives::validation::{chain, required};
use iad::widgets::{Form, Step, StepIndicator};
use iad::{Element, Metrics};
use iad::{components as cmp, widgets as w};
use iced::Length;
use iced::widget::{column, container, text};

const FRUITS: [&str; 3] = ["Apple", "Banana", "Cherry"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Name,
    Password,
    Fruit,
    Birthday,
    Agree,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Single(Edit),
    Wizard(Edit),
}

#[derive(Debug, Clone)]
pub enum Edit {
    Name(String),
    Password(String),
    Fruit(&'static str),
    Date(String),
    Agree(bool),
    Next,
    Back,
    Step(usize),
    Submit,
    Reset,
}

#[derive(Default)]
struct Values {
    name: String,
    password: String,
    fruit: Option<&'static str>,
    date: String,
    agree: bool,
}

impl Values {
    fn validity(&self, k: &Key) -> Validity {
        match k {
            Key::Name => chain(
                &self.name,
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
            ),
            Key::Password => chain(
                &self.password,
                &[
                    |s| required(s, "Choose a password."),
                    |s| {
                        if s.len() < 6 {
                            Validity::error("Use at least 6 characters.")
                        } else {
                            Validity::Ok
                        }
                    },
                ],
            ),
            Key::Fruit => self
                .fruit
                .map_or(Validity::error("Pick a fruit."), |_| Validity::Ok),
            Key::Birthday => chain(
                &self.date,
                &[
                    |s| required(s, "Enter your birthday."),
                    |s| iad::primitives::date::validate(s, None, None),
                ],
            ),
            Key::Agree => {
                if self.agree {
                    Validity::Ok
                } else {
                    Validity::error("You must agree to continue.")
                }
            }
        }
    }

    fn filled() -> Self {
        Self {
            name: "Ada Lovelace".into(),
            password: "analytical".into(),
            fruit: Some("Cherry"),
            date: "1815-12-10".into(),
            agree: false,
        }
    }

    fn summary(&self) -> String {
        format!(
            "name = {}, password = {}, fruit = {}, birthday = {}, agreed = {}",
            self.name,
            "*".repeat(self.password.len()),
            self.fruit.unwrap_or("-"),
            self.date,
            self.agree
        )
    }
}

/// One form: the app's values, the primitive's state and the last result.
struct Demo {
    values: Values,
    form: FormState<Key>,
    spec: FormSpec<Key>,
    result: Option<String>,
}

impl Demo {
    fn new(spec: FormSpec<Key>) -> Self {
        Self {
            values: Values::default(),
            form: FormState::new(),
            spec,
            result: None,
        }
    }

    fn update(&mut self, e: Edit) {
        let v = &self.values;
        let validity = |k: &Key| v.validity(k);
        match e {
            Edit::Name(s) => {
                self.values.name = s;
                self.form.touch(Key::Name);
            }
            Edit::Password(s) => {
                self.values.password = s;
                self.form.touch(Key::Password);
            }
            Edit::Fruit(f) => {
                self.values.fruit = Some(f);
                self.form.touch(Key::Fruit);
            }
            Edit::Date(s) => {
                self.values.date = s;
                self.form.touch(Key::Birthday);
            }
            Edit::Agree(a) => {
                self.values.agree = a;
                self.form.touch(Key::Agree);
            }
            Edit::Next => {
                self.form.next(&self.spec, &validity);
            }
            Edit::Back => {
                self.form.back();
            }
            Edit::Step(i) => {
                self.form.goto(&self.spec, i, &validity);
            }
            Edit::Submit => {
                self.result = (self.form.submit(&self.spec, &validity) == FormOutcome::Submitted)
                    .then(|| self.values.summary());
            }
            Edit::Reset => {
                self.form.reset();
                self.values = Values::default();
                self.result = None;
            }
        }
    }

    fn account<'a>(&'a self, m: Metrics) -> Element<'a, Edit> {
        let v = &self.values;
        column![
            w::TextEntry::new("Name", &v.name, Edit::Name)
                .validity(v.validity(&Key::Name))
                .interaction(self.form.interaction(&self.spec, &Key::Name))
                .required(true)
                .metrics(m),
            w::TextEntry::from_input(
                "Password",
                cmp::password_entry("Password", &v.password)
                    .on_input(Edit::Password)
                    .metrics(m)
            )
            .validity(v.validity(&Key::Password))
            .interaction(self.form.interaction(&self.spec, &Key::Password))
            .required(true)
            .metrics(m),
        ]
        .spacing(m.space_m)
        .into()
    }

    fn profile<'a>(&'a self, m: Metrics) -> Element<'a, Edit> {
        let v = &self.values;
        column![
            w::SelectEntry::new("Favorite fruit", &FRUITS, v.fruit, Edit::Fruit)
                .validity(v.validity(&Key::Fruit))
                .interaction(self.form.interaction(&self.spec, &Key::Fruit))
                .required(true)
                .metrics(m),
            w::DateEntry::from_input(
                "Birthday",
                w::DateInput::new(
                    &v.date,
                    iad::primitives::date::Date::new(2026, 11, 1).unwrap(),
                    Edit::Date
                )
                .revealed(self.form.revealed(&self.spec, &Key::Birthday))
                .metrics(m)
            )
            .validity(v.validity(&Key::Birthday))
            .interaction(self.form.interaction(&self.spec, &Key::Birthday))
            .description("YYYY-MM-DD")
            .metrics(m),
        ]
        .spacing(m.space_m)
        .into()
    }

    fn agree<'a>(&'a self, m: Metrics) -> Element<'a, Edit> {
        w::SwitchEntry::new("I agree to the terms", self.values.agree, Edit::Agree)
            .validity(self.values.validity(&Key::Agree))
            .interaction(self.form.interaction(&self.spec, &Key::Agree))
            .required(true)
            .metrics(m)
            .into()
    }

    fn review<'a>(&'a self, m: Metrics) -> Element<'a, Edit> {
        let v = &self.values;
        let line = |k: &'static str, val: String| {
            w::EntryRow::<Edit>::new(k, w::EntryInput::Custom(iced::widget::text(val).into())).metrics(m)
        };
        column![
            w::boxed_list([
                line("Name", v.name.clone()).into(),
                line("Password", "*".repeat(v.password.len())).into(),
                line("Favorite fruit", v.fruit.unwrap_or("-").to_owned()).into(),
                line("Birthday", v.date.clone()).into(),
            ]),
            self.agree(m),
        ]
        .spacing(m.space_m * 1.5)
        .into()
    }

    fn view<'a>(&'a self, m: Metrics, wizard: bool) -> Element<'a, Edit> {
        let v = &self.values;
        let form = Form::new()
            .metrics(m)
            .state(&self.form, &self.spec, |k| v.validity(k))
            .on_submit(Edit::Submit)
            .on_reset(Edit::Reset);
        let form = if wizard {
            form.title("Create an account")
                .description("Three steps; Next is refused while the step has an error.")
                .step(Step::new("Account", move || self.account(m)))
                .step(Step::new("Profile", move || self.profile(m)))
                .step(Step::new("Review", move || self.review(m)))
                .on_next(Edit::Next)
                .on_back(Edit::Back)
                .on_step(Edit::Step)
                .submit_label("Create account")
        } else {
            form.title("Sign up").step(Step::new("Details", move || {
                column![self.account(m), self.profile(m), self.agree(m)]
                    .spacing(m.space_m * 1.5)
                    .into()
            }))
        };
        let result: Element<'a, Edit> = match &self.result {
            Some(s) => w::card(
                container(
                    column![
                        ty::heading("Submitted with"),
                        text(s.clone()).size(m.space_m + 1.0)
                    ]
                    .spacing(m.space_xs),
                )
                .padding(m.space_m)
                .width(Length::Fill),
            ),
            None => iced::widget::space().into(),
        };
        column![form, result].spacing(m.space_l).into()
    }
}

pub struct State {
    single: Demo,
    wizard: Demo,
}

impl Default for State {
    fn default() -> Self {
        let mut single = Demo::new(FormSpec::single([
            Key::Name,
            Key::Password,
            Key::Fruit,
            Key::Birthday,
            Key::Agree,
        ]));
        let mut wizard = Demo::new(FormSpec::steps([
            vec![Key::Name, Key::Password],
            vec![Key::Fruit, Key::Birthday],
            vec![Key::Agree],
        ]));
        let step = std::env::var("GALLERY_FORM_STEP")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        if step > 0 {
            wizard.values = Values::filled();
            if step < 2 {
                // Leave the profile empty so Next has something to refuse.
                wizard.values.fruit = None;
                wizard.values.date = String::new();
            }
            wizard.form = FormState::new().at_step(step.min(2));
        }
        if std::env::var_os("GALLERY_FORM_ATTEMPT").is_some() {
            for d in [&mut single, &mut wizard] {
                d.form = std::mem::take(&mut d.form).attempted();
            }
        }
        Self { single, wizard }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Single(e) => self.single.update(e),
            Msg::Wizard(e) => self.wizard.update(e),
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let g = |t: &'static str| w::PreferencesGroup::<Msg>::new().title(t).metrics(m);
        let sp = m.space_m;
        let single = g("Single page")
            .description(
                "One step: only Reset and Submit. Submit is refused until every field is valid.",
            )
            .push(w::card(
                container(self.single.view(m, false).map(Msg::Single))
                    .padding(sp)
                    .width(Length::Fill),
            ));
        let wizard = g("Wizard").description(
            "Click a completed step to go back; later steps open once the ones before are valid.",
        ).push(w::card(
            container(self.wizard.view(m, true).map(Msg::Wizard))
                .padding(sp)
                .width(Length::Fill),
        ));
        let names = ["Account", "Profile", "Review", "Payment"];
        let strip = |states: [StepStatus; 4]| -> Element<'_, Msg> {
            StepIndicator::new(
                names
                    .iter()
                    .zip(states)
                    .map(|(n, s)| ((*n).to_owned(), s))
                    .collect(),
            )
            .metrics(m)
            .into()
        };
        use StepStatus::*;
        let strips = g("Step indicator states")
            .description("Completed, current, upcoming and invalid (after a refused Next).")
            .push(w::card(
                container(
                    column![
                        strip([Current, Upcoming, Upcoming, Upcoming]),
                        strip([Completed, Current, Upcoming, Upcoming]),
                        strip([Completed, Invalid, Upcoming, Upcoming]),
                        strip([Completed, Completed, Completed, Current]),
                    ]
                    .spacing(sp),
                )
                .padding(sp)
                .width(Length::Fill),
            ));

        match std::env::var("GALLERY_FORM").as_deref() {
            Ok("single") => return single.into(),
            Ok("wizard") => return wizard.into(),
            Ok("strip") => return strips.into(),
            _ => {}
        }
        column![single, wizard, strips]
            .spacing(m.group_spacing)
            .into()
    }
}
