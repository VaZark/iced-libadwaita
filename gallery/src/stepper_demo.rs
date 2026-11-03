//! The Stepper page: horizontal and vertical steppers driven by Back / Next, click-to-jump with
//! and without `allow_next_steps_select`, descriptions and icons, an invalid and a loading
//! step, icon position, a narrow width, and a static strip of every status.
//!
//! Screenshot switches: `GALLERY_STEPPER=horizontal|vertical|jump|icons|invalid|loading|right|
//! narrow|strip|completed` shows one demo only, `GALLERY_STEPPER_STEP=n` starts the interactive
//! demos on step n (3 is the completed screen), `GALLERY_STEPPER_SIZE=xs|sm|md|lg|xl` sets the
//! size of the "horizontal" demo.

use iced_adaptive_design as iad;

use iad::Icon;
use iad::widget::{
    self as w, Stepper, StepperIconPosition as Pos, StepperOrientation as Orient, StepperSize,
    StepperState, StepperStatus, StepperStep as S, typography as ty,
};
use iad::{Element, Metrics};
use iced::Length;
use iced::widget::{column, container, row};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    Horizontal,
    Vertical,
    Jump,
    Icons,
    Invalid,
    Loading,
    Right,
    Narrow,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Go(Which, usize),
    Next(Which),
    Back(Which),
    AllowNext(bool),
}

pub struct State {
    horizontal: StepperState,
    vertical: StepperState,
    jump: StepperState,
    icons: StepperState,
    invalid: StepperState,
    loading: StepperState,
    right: StepperState,
    narrow: StepperState,
}

impl Default for State {
    fn default() -> Self {
        let at = std::env::var("GALLERY_STEPPER_STEP")
            .ok()
            .and_then(|s| s.parse::<usize>().ok());
        let mk = |n: usize, default: usize| StepperState::new(n).at(at.unwrap_or(default));
        let completed = std::env::var("GALLERY_STEPPER").as_deref() == Ok("completed");
        Self {
            horizontal: mk(3, if completed { 3 } else { 1 }),
            vertical: mk(3, 1),
            jump: mk(4, 1),
            icons: mk(4, 2),
            invalid: mk(4, 2),
            loading: mk(4, 1),
            right: mk(3, 1),
            narrow: mk(4, 1),
        }
    }
}

const NAMES: [&str; 3] = ["Account", "Profile", "Review"];

impl State {
    fn of(&mut self, w: Which) -> &mut StepperState {
        match w {
            Which::Horizontal => &mut self.horizontal,
            Which::Vertical => &mut self.vertical,
            Which::Jump => &mut self.jump,
            Which::Icons => &mut self.icons,
            Which::Invalid => &mut self.invalid,
            Which::Loading => &mut self.loading,
            Which::Right => &mut self.right,
            Which::Narrow => &mut self.narrow,
        }
    }

    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Go(w, i) => {
                self.of(w).goto(i);
            }
            Msg::Next(w) => {
                self.of(w).forward();
            }
            Msg::Back(w) => {
                self.of(w).back();
            }
            Msg::AllowNext(a) => self.jump.allow_next = a,
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let g = |t: &'static str, d: &'static str, body: Element<'static, Msg>| {
            w::PreferencesGroup::<Msg>::new()
                .title(t)
                .description(d)
                .metrics(m)
                .push(w::card(container(body).padding(sp).width(Length::Fill)))
        };
        // Back / Next under a stepper, disabled at the ends.
        let nav = |which: Which, s: &StepperState| -> Element<'static, Msg> {
            let back = w::raised_button(ty::label("Back"))
                .on_press_maybe((!s.is_first()).then_some(Msg::Back(which)));
            let next = w::suggested_button(ty::label(if s.is_last() { "Finish" } else { "Next" }))
                .on_press_maybe((!s.is_completed()).then_some(Msg::Next(which)));
            let reset = w::raised_button(ty::label("Reset"))
                .on_press_maybe(s.is_completed().then_some(Msg::Go(which, 0)));
            row![back, next, reset].spacing(m.space_s).into()
        };
        let body = |t: &'static str| -> Element<'static, Msg> {
            ty::body(t).style(iad::style::text::dim).into()
        };
        let done = || -> Element<'static, Msg> { ty::heading("All steps completed").into() };
        let wrap =
            |s: &StepperState, which: Which, st: Stepper<'static, Msg>| -> Element<'static, Msg> {
                column![st, nav(which, s)].spacing(sp).into()
            };

        let size = match std::env::var("GALLERY_STEPPER_SIZE").as_deref() {
            Ok("xs") => StepperSize::Xs,
            Ok("sm") => StepperSize::Sm,
            Ok("lg") => StepperSize::Lg,
            Ok("xl") => StepperSize::Xl,
            _ => StepperSize::Md,
        };

        let s = &self.horizontal;
        let horizontal = g(
            "Horizontal",
            "Back / Next move active; past the last step the completed screen shows. Clicking a completed step goes back.",
            wrap(
                s,
                Which::Horizontal,
                Stepper::new()
                    .active(s.active())
                    .size(size)
                    .metrics(m)
                    .on_step_click(|i| Msg::Go(Which::Horizontal, i))
                    .step(S::new(NAMES[0]).content(move || body("Name and password.")))
                    .step(S::new(NAMES[1]).content(move || body("Favorite fruit and birthday.")))
                    .step(S::new(NAMES[2]).content(move || body("Check everything and submit.")))
                    .completed(done),
            ),
        );

        let s = &self.vertical;
        let vertical = g(
            "Vertical",
            "The active step's content sits under its label, the line running down beside it.",
            wrap(
                s,
                Which::Vertical,
                Stepper::new()
                    .active(s.active())
                    .orientation(Orient::Vertical)
                    .metrics(m)
                    .on_step_click(|i| Msg::Go(Which::Vertical, i))
                    .step(
                        S::new("Account")
                            .description("Name and password")
                            .content(move || body("Pick a name of at least three letters.")),
                    )
                    .step(
                        S::new("Profile")
                            .description("Fruit and birthday")
                            .content(move || body("Everything here is optional.")),
                    )
                    .step(
                        S::new("Review")
                            .description("Check and submit")
                            .content(move || body("Last look before sending.")),
                    )
                    .completed(done),
            ),
        );

        let s = &self.jump;
        let jump = {
            let st = Stepper::new()
                .active(s.active())
                .allow_next_steps_select(s.allow_next)
                .metrics(m)
                .on_step_click(|i| Msg::Go(Which::Jump, i))
                .step(S::new("Cart"))
                .step(S::new("Address"))
                .step(
                    S::new("Payment")
                        .allow_click(true)
                        .description("always clickable"),
                )
                .step(S::new("Done").disabled(true).description("disabled"));
            let sw = w::SwitchRow::new("Allow next steps select", s.allow_next, Msg::AllowNext)
                .metrics(m);
            g(
                "Click to jump",
                "Off: only steps before the active one respond, except a step that opts in. A disabled step never responds.",
                column![w::boxed_list([sw.into()]), wrap(s, Which::Jump, st)]
                    .spacing(sp)
                    .into(),
            )
        };

        let s = &self.icons;
        let icons = g(
            "Descriptions and custom icons",
            "An icon replaces the number; a completed icon replaces the check.",
            wrap(
                s,
                Which::Icons,
                Stepper::new()
                    .active(s.active())
                    .metrics(m)
                    .on_step_click(|i| Msg::Go(Which::Icons, i))
                    .step(
                        S::new("Settings")
                            .description("Preferences")
                            .icon(Icon::Settings),
                    )
                    .step(S::new("Search").description("Find it").icon(Icon::Search))
                    .step(
                        S::new("Edit")
                            .description("Make changes")
                            .icon(Icon::Edit)
                            .completed_icon(Icon::ListAdd),
                    )
                    .step(S::new("Home").description("Finish").icon(Icon::Home)),
            ),
        );

        let s = &self.invalid;
        let invalid = g(
            "Invalid step",
            "Step 2 failed validation: an error mark and the error color, whatever its position.",
            wrap(
                s,
                Which::Invalid,
                Stepper::new()
                    .active(s.active())
                    .metrics(m)
                    .on_step_click(|i| Msg::Go(Which::Invalid, i))
                    .step(S::new("Account"))
                    .step(
                        S::new("Profile")
                            .description("Needs attention")
                            .invalid(true),
                    )
                    .step(S::new("Review"))
                    .step(S::new("Submit")),
            ),
        );

        let s = &self.loading;
        let loading = g(
            "Loading step",
            "Step 2 is waiting on something: a loading mark in its circle.",
            wrap(
                s,
                Which::Loading,
                Stepper::new()
                    .active(s.active())
                    .metrics(m)
                    .on_step_click(|i| Msg::Go(Which::Loading, i))
                    .step(S::new("Account"))
                    .step(S::new("Verify").description("Checking...").loading(true))
                    .step(S::new("Review"))
                    .step(S::new("Submit")),
            ),
        );

        let s = &self.right;
        let right = g(
            "Icon position right",
            "The circle follows the label; vertical keeps the line on the same side.",
            wrap(
                s,
                Which::Right,
                Stepper::new()
                    .active(s.active())
                    .icon_position(Pos::Right)
                    .metrics(m)
                    .on_step_click(|i| Msg::Go(Which::Right, i))
                    .step(S::new("Account").description("Who you are"))
                    .step(S::new("Profile").description("What you like"))
                    .step(S::new("Review").description("Last look")),
            ),
        );
        let right_v = Stepper::<Msg>::new()
            .active(1)
            .orientation(Orient::Vertical)
            .icon_position(Pos::Right)
            .metrics(m)
            .step(S::new("Account").description("Who you are"))
            .step(S::new("Profile").description("What you like").content(|| {
                ty::body("Content stays on the label's side.")
                    .style(iad::style::text::dim)
                    .into()
            }))
            .step(S::new("Review"));
        let right = column![
            right,
            g(
                "Right, vertical",
                "The circle and the line on the right, content on the label side.",
                right_v.into()
            )
        ]
        .spacing(m.group_spacing);

        let s = &self.narrow;
        let narrow_steps = |st: Stepper<'static, Msg>| {
            st.active(s.active())
                .metrics(m)
                .on_step_click(|i| Msg::Go(Which::Narrow, i))
                .step(S::new("Account").description("Who"))
                .step(S::new("Profile").description("What"))
                .step(S::new("Shipping").description("Where"))
                .step(S::new("Review").description("Check"))
        };
        let narrow_box = |e: Element<'static, Msg>| -> Element<'static, Msg> {
            container(e).width(Length::Fixed(300.0)).into()
        };
        let narrow = g(
            "Narrow width",
            "300px wide. By default the row collapses: only the current step keeps its label. With wrap the steps flow onto more lines.",
            column![
                ty::caption_heading("Collapse (default)"),
                narrow_box(narrow_steps(Stepper::new()).into()),
                ty::caption_heading("Wrap"),
                narrow_box(narrow_steps(Stepper::new().wrap(true)).into()),
                nav(Which::Narrow, s),
            ]
            .spacing(sp)
            .into(),
        );

        use StepperStatus::*;
        let strip_row =
            |states: [StepperStatus; 5], names: [&'static str; 5]| -> Element<'static, Msg> {
                let mut st = Stepper::<Msg>::new().metrics(m).statuses(states.to_vec());
                for n in names {
                    st = st.step(S::new(n));
                }
                st.into()
            };
        let sizes = {
            let mut c = column![].spacing(sp);
            for (n, sz) in [
                ("xs", StepperSize::Xs),
                ("sm", StepperSize::Sm),
                ("md", StepperSize::Md),
                ("lg", StepperSize::Lg),
                ("xl", StepperSize::Xl),
            ] {
                c = c.push(
                    Stepper::<Msg>::new()
                        .active(1)
                        .size(sz)
                        .metrics(m)
                        .step(S::new(format!("Size {n}")))
                        .step(S::new("Next"))
                        .step(S::new("Last")),
                );
            }
            c
        };
        let disabled = Stepper::<Msg>::new()
            .active(1)
            .metrics(m)
            .step(S::new("Done").disabled(true))
            .step(S::new("Current").disabled(true))
            .step(S::new("Upcoming").disabled(true));
        let strip = g(
            "States",
            "Each status, then the sizes, then disabled steps.",
            column![
                strip_row(
                    [Completed, Current, Upcoming, Invalid, Loading],
                    ["Completed", "Current", "Upcoming", "Invalid", "Loading"]
                ),
                strip_row(
                    [Completed, Completed, Current, Upcoming, Upcoming],
                    ["Cart", "Address", "Payment", "Review", "Done"]
                ),
                sizes,
                disabled,
            ]
            .spacing(sp)
            .into(),
        );

        let demos: Vec<(&str, Element<'_, Msg>)> = vec![
            ("horizontal", horizontal.into()),
            ("vertical", vertical.into()),
            ("jump", jump.into()),
            ("icons", icons.into()),
            ("invalid", invalid.into()),
            ("loading", loading.into()),
            ("right", right.into()),
            ("narrow", narrow.into()),
            ("strip", strip.into()),
        ];
        let only = std::env::var("GALLERY_STEPPER").ok();
        let only = only
            .as_deref()
            .map(|o| if o == "completed" { "horizontal" } else { o });
        let mut col = column![].spacing(m.group_spacing);
        for (name, el) in demos {
            if only.is_none_or(|o| o == name) {
                col = col.push(el);
            }
        }
        col.into()
    }
}
