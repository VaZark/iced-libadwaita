//! `Stepper`: a sequence of steps with the current one's content, after Mantine's `Stepper`.
//! The behavior (statuses from `active`, click rules, next / back, connector fill) is
//! [`primitives::stepper`](crate::primitives::stepper); this widget is its look.
//!
//! Controlled: the app owns `active` (`0..steps`, or `steps` for the completed screen) and
//! moves it from `on_step_click` and its own Back / Next buttons.
//!
//! ```ignore
//! // state: `step: usize` (see `StepperState` for next / prev / goto)
//! Stepper::new()
//!     .active(self.step)
//!     .on_step_click(Msg::Step)
//!     .allow_next_steps_select(false)
//!     .step(StepperStep::new("Account").description("Name and password").content(|| account()))
//!     .step(StepperStep::new("Verify").loading(self.checking).content(|| verify()))
//!     .step(StepperStep::new("Done"))
//!     .completed(|| text("All set").into())     // shown when active == 3
//! ```
//!
//! * a step's circle shows its number, its `icon`, a check (or its `completed_icon`), an error
//!   mark (`invalid`), or a loading mark (`loading`); the lines between steps fill with the
//!   accent up to the active step;
//! * **horizontal** puts the steps in a row with the content of the active step below;
//!   **vertical** stacks them, the line running down beside the active step's content;
//! * only the active step's content closure is called;
//! * on a narrow width a horizontal stepper [`collapse`](Stepper::collapse)s to the current
//!   step's label (the others keep their circle), or, with [`wrap`](Stepper::wrap), flows onto
//!   several lines;
//! * [`statuses`](Stepper::statuses) and [`clickable`](Stepper::clickable) replace the
//!   derived statuses and click rules (a `Form` gives its own: see
//!   [`StepIndicator`](super::StepIndicator)).

use super::{Element, flat_button};
use crate::primitives::stepper::{self as prim, Fill, Status};
use crate::{Icon, Metrics, Theme};
use iced::widget::{column, container, responsive, row, space, svg, text};
use iced::{Alignment, Border, Color, Length, Padding};

/// How the steps are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepperOrientation {
    /// In a row, the active step's content below.
    #[default]
    Horizontal,
    /// In a column, the active step's content under its label.
    Vertical,
}

/// Which side of its label a step's circle sits on (Mantine `iconPosition`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepperIconPosition {
    #[default]
    Left,
    Right,
}

/// The size of the circles and text (Mantine `size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepperSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl StepperSize {
    /// The circle's diameter, logical px.
    pub fn diameter(self) -> f32 {
        match self {
            Self::Xs => 20.0,
            Self::Sm => 24.0,
            Self::Md => 26.0,
            Self::Lg => 32.0,
            Self::Xl => 40.0,
        }
    }
    fn text_scale(self) -> f32 {
        match self {
            Self::Xs | Self::Sm => 0.92,
            Self::Md => 1.0,
            Self::Lg => 1.1,
            Self::Xl => 1.2,
        }
    }
}

/// One step: label, optional description and icons, state flags and, optionally, its own
/// content (a closure, called only while the step is active).
pub struct StepperStep<'a, M> {
    label: String,
    description: Option<String>,
    icon: Option<Icon>,
    completed_icon: Option<Icon>,
    loading: bool,
    invalid: bool,
    disabled: bool,
    allow_click: Option<bool>,
    build: Option<Box<dyn Fn() -> Element<'a, M> + 'a>>,
}

impl<'a, M> StepperStep<'a, M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            icon: None,
            completed_icon: None,
            loading: false,
            invalid: false,
            disabled: false,
            allow_click: None,
            build: None,
        }
    }
    /// A line of dim text under the label.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Shown in the circle instead of the number (until the step completes).
    pub fn icon(mut self, i: Icon) -> Self {
        self.icon = Some(i);
        self
    }
    /// Shown instead of the check once the step is completed.
    pub fn completed_icon(mut self, i: Icon) -> Self {
        self.completed_icon = Some(i);
        self
    }
    /// A loading mark in the circle (work in flight).
    pub fn loading(mut self, on: bool) -> Self {
        self.loading = on;
        self
    }
    /// An error mark and the error color; wins over `loading`.
    pub fn invalid(mut self, on: bool) -> Self {
        self.invalid = on;
        self
    }
    /// Dim, and never responds to a click (Mantine `allowStepClick={false}`).
    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }
    /// Responds to a click even past the active step with `allow_next_steps_select(false)`.
    pub fn allow_click(mut self, on: bool) -> Self {
        self.allow_click = Some(on);
        self
    }
    /// The step's content, built only while it is active.
    pub fn content(mut self, f: impl Fn() -> Element<'a, M> + 'a) -> Self {
        self.build = Some(Box::new(f));
        self
    }
}

/// See the [module](self).
pub struct Stepper<'a, M> {
    steps: Vec<StepperStep<'a, M>>,
    active: usize,
    on_step_click: Option<Box<dyn Fn(usize) -> M + 'a>>,
    allow_next: bool,
    orientation: StepperOrientation,
    icon_position: StepperIconPosition,
    size: StepperSize,
    icon_size: Option<f32>,
    wrap: bool,
    collapse: bool,
    progress: f32,
    statuses: Option<Vec<Status>>,
    clickable: Option<Vec<bool>>,
    completed: Option<Box<dyn Fn() -> Element<'a, M> + 'a>>,
    content_spacing: Option<f32>,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> Default for Stepper<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: Clone + 'a> Stepper<'a, M> {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            active: 0,
            on_step_click: None,
            allow_next: true,
            orientation: StepperOrientation::Horizontal,
            icon_position: StepperIconPosition::Left,
            size: StepperSize::Md,
            icon_size: None,
            wrap: false,
            collapse: true,
            progress: 0.0,
            statuses: None,
            clickable: None,
            completed: None,
            content_spacing: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn step(mut self, s: StepperStep<'a, M>) -> Self {
        self.steps.push(s);
        self
    }
    /// The active step; `steps.len()` shows the completed screen.
    pub fn active(mut self, a: usize) -> Self {
        self.active = a;
        self
    }
    /// A click on a step that responds (see the primitive's `can_click`).
    pub fn on_step_click(mut self, f: impl Fn(usize) -> M + 'a) -> Self {
        self.on_step_click = Some(Box::new(f));
        self
    }
    /// Whether steps after the active one respond to a click (default on).
    pub fn allow_next_steps_select(mut self, allow: bool) -> Self {
        self.allow_next = allow;
        self
    }
    pub fn orientation(mut self, o: StepperOrientation) -> Self {
        self.orientation = o;
        self
    }
    pub fn icon_position(mut self, p: StepperIconPosition) -> Self {
        self.icon_position = p;
        self
    }
    pub fn size(mut self, s: StepperSize) -> Self {
        self.size = s;
        self
    }
    /// The circle's diameter in px, overriding [`size`](Self::size)'s.
    pub fn icon_size(mut self, px: f32) -> Self {
        self.icon_size = Some(px);
        self
    }
    /// Horizontal only: flow onto several lines when the steps do not fit (the lines between
    /// steps then have a fixed length).
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }
    /// Horizontal, not wrapping: when the steps do not fit, only the current step keeps its
    /// label (on by default; the width of text is estimated, not measured).
    pub fn collapse(mut self, on: bool) -> Self {
        self.collapse = on;
        self
    }
    /// How far the active step is, `0.0..=1.0`: the line leaving it is filled that much.
    pub fn step_progress(mut self, p: f32) -> Self {
        self.progress = p;
        self
    }
    /// Each step's status, replacing the one derived from `active` (the lines then fill after
    /// completed steps only). Missing entries are upcoming.
    pub fn statuses(mut self, s: Vec<Status>) -> Self {
        self.statuses = Some(s);
        self
    }
    /// Which steps respond to a click, replacing the click rules.
    pub fn clickable(mut self, c: Vec<bool>) -> Self {
        self.clickable = Some(c);
        self
    }
    /// The final screen, shown when `active == steps.len()` (Mantine `Stepper.Completed`).
    pub fn completed(mut self, f: impl Fn() -> Element<'a, M> + 'a) -> Self {
        self.completed = Some(Box::new(f));
        self
    }
    /// The gap between the steps and the content.
    pub fn content_spacing(mut self, px: f32) -> Self {
        self.content_spacing = Some(px);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

// ---- drawing ----

/// One step with everything the drawing needs, owned so the horizontal layout can be rebuilt
/// on every width.
struct Item<M> {
    label: String,
    description: Option<String>,
    icon: Option<Icon>,
    completed_icon: Option<Icon>,
    status: Status,
    disabled: bool,
    click: Option<M>,
}

struct Look<M> {
    items: Vec<Item<M>>,
    fills: Vec<Fill>,
    diameter: f32,
    scale: f32,
    right: bool,
    m: Metrics,
}

fn bold() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    }
}

/// A solid tint for an svg (iced ignores a tint's alpha): `c` mixed towards the window
/// background when disabled.
fn solid(t: &Theme, c: Color, disabled: bool) -> Color {
    let c = Color { a: 1.0, ..c };
    if disabled {
        crate::color::mix(t.semantic.window.bg, c, t.semantic.opacity.disabled)
    } else {
        c
    }
}

fn dim_fg(t: &Theme, disabled: bool) -> Color {
    let c = crate::color::fade(t.semantic.window.fg, 0.7);
    if disabled {
        crate::color::fade(c, t.semantic.opacity.disabled)
    } else {
        c
    }
}

impl<M: Clone> Look<M> {
    fn glyph(&self) -> f32 {
        (self.m.space_m + 2.0) * self.diameter / 26.0
    }

    /// The circle of step `n`.
    fn mark<'a>(&self, n: usize, it: &Item<M>) -> Element<'a, M>
    where
        M: 'a,
    {
        let (status, dis) = (it.status, it.disabled);
        let size = self.glyph();
        let svg_of = |icon: Icon, tint: fn(&Theme, bool) -> Color| -> Element<'a, M> {
            icon.view(size)
                .style(move |t: &Theme, _| svg::Style {
                    color: Some(tint(t, dis)),
                })
                .into()
        };
        let inner: Element<'a, M> = match status {
            Status::Completed => svg_of(it.completed_icon.unwrap_or(Icon::Check), |t, d| {
                solid(t, t.semantic.accent.color, d)
            }),
            Status::Invalid => text("!")
                .size(size + 2.0)
                .font(bold())
                .style(|t: &Theme| text::Style {
                    color: Some(t.semantic.error.color),
                })
                .into(),
            Status::Loading => {
                let dot = move || {
                    container(space())
                        .width(Length::Fixed(size / 5.0))
                        .height(Length::Fixed(size / 5.0))
                        .style(move |t: &Theme| container::Style {
                            background: Some(
                                crate::color::fade(
                                    t.semantic.accent.color,
                                    if dis {
                                        t.semantic.opacity.disabled
                                    } else {
                                        1.0
                                    },
                                )
                                .into(),
                            ),
                            border: Border::default().rounded(size),
                            ..Default::default()
                        })
                };
                row![dot(), dot(), dot()]
                    .spacing(size / 8.0)
                    .align_y(Alignment::Center)
                    .into()
            }
            Status::Current | Status::Upcoming => match it.icon {
                Some(icon) => {
                    if status == Status::Current {
                        svg_of(icon, |t, d| solid(t, t.semantic.accent.fg, d))
                    } else {
                        svg_of(icon, |t, d| {
                            solid(
                                t,
                                crate::color::mix(t.semantic.window.bg, t.semantic.window.fg, 0.7),
                                d,
                            )
                        })
                    }
                }
                None => text(prim::number(n).to_string())
                    .size(size)
                    .font(bold())
                    .style(move |t: &Theme| text::Style {
                        color: Some(if status == Status::Current {
                            t.semantic.accent.fg
                        } else {
                            dim_fg(t, dis)
                        }),
                    })
                    .into(),
            },
        };
        let d = self.diameter;
        container(inner)
            .center(Length::Fixed(d))
            .style(move |t: &Theme| {
                let s = &t.semantic;
                let (bg, border) = match status {
                    Status::Current => (Some(s.accent.bg), s.accent.bg),
                    Status::Completed | Status::Loading => {
                        (Some(crate::color::fade(s.accent.bg, 0.18)), s.accent.bg)
                    }
                    Status::Invalid => (Some(crate::color::fade(s.error.bg, 0.18)), s.error.bg),
                    Status::Upcoming => (None, s.border),
                };
                let k = if dis { s.opacity.disabled } else { 1.0 };
                container::Style {
                    background: bg.map(|c| crate::color::fade(c, k).into()),
                    border: Border {
                        color: crate::color::fade(border, k),
                        width: 1.5,
                        radius: (d / 2.0).into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    }

    /// The label and description of one step.
    fn body<'a>(&self, it: &Item<M>, wrapping: bool) -> Element<'a, M>
    where
        M: 'a,
    {
        let (status, dis) = (it.status, it.disabled);
        let size = (self.m.space_m + 1.0) * self.scale;
        let wrap = if wrapping {
            text::Wrapping::Word
        } else {
            text::Wrapping::None
        };
        let title = text(it.label.clone())
            .size(size)
            .wrapping(wrap)
            .font(if status == Status::Current {
                bold()
            } else {
                iced::Font::DEFAULT
            })
            .style(move |t: &Theme| {
                let c = match status {
                    Status::Invalid => t.semantic.error.color,
                    Status::Upcoming => crate::color::fade(t.semantic.window.fg, 0.7),
                    _ => t.semantic.window.fg,
                };
                text::Style {
                    color: Some(if dis {
                        crate::color::fade(c, t.semantic.opacity.disabled)
                    } else {
                        c
                    }),
                }
            });
        let mut col = column![title].spacing(self.m.space_xxs);
        if let Some(d) = &it.description {
            col = col.push(text(d.clone()).size(size - 2.0).wrapping(wrap).style(
                move |t: &Theme| text::Style {
                    color: Some(if dis {
                        crate::color::fade(t.semantic.window.fg, 0.4 * t.semantic.opacity.disabled)
                    } else {
                        crate::color::fade(t.semantic.window.fg, t.semantic.opacity.dim + 0.1)
                    }),
                },
            ));
        }
        col.into()
    }

    /// Circle and text of step `i` (the text left out when `compact`), as a button when the
    /// step responds.
    fn head<'a>(&self, i: usize, compact: bool, wrapping: bool, fill: bool) -> Element<'a, M>
    where
        M: 'a,
    {
        let it = &self.items[i];
        let mark = self.mark(i, it);
        let mut r = row![].spacing(self.m.space_s).align_y(Alignment::Center);
        if compact {
            r = r.push(mark);
        } else {
            let body = self.body(it, wrapping);
            let body: Element<'a, M> = if fill {
                container(body).width(Length::Fill).into()
            } else {
                body
            };
            r = if self.right {
                r.push(body).push(mark)
            } else {
                r.push(mark).push(body)
            };
        }
        if fill {
            r = r.width(Length::Fill);
        }
        let pad = Padding::from([self.m.space_xs, self.m.space_s]);
        match &it.click {
            Some(msg) => {
                let b = flat_button(r).padding(pad).on_press(msg.clone());
                if fill { b.width(Length::Fill) } else { b }.into()
            }
            None => {
                let c = container(r).padding(pad);
                if fill { c.width(Length::Fill) } else { c }.into()
            }
        }
    }

    /// A line segment, `thick` px across, along `horizontal`; filled by `fill`.
    fn line<'a>(&self, fill: Fill, horizontal: bool, fixed: Option<f32>) -> Element<'a, M>
    where
        M: 'a,
    {
        let seg = move |lit: bool, portion: u16| {
            let c = container(space()).style(move |t: &Theme| container::Style {
                background: Some(
                    if lit {
                        t.semantic.accent.bg
                    } else {
                        t.semantic.border
                    }
                    .into(),
                ),
                border: Border::default().rounded(1.0),
                ..Default::default()
            });
            if horizontal {
                c.width(Length::FillPortion(portion)).height(Length::Fill)
            } else {
                c.height(Length::FillPortion(portion)).width(Length::Fill)
            }
        };
        let inner: Element<'a, M> = match fill {
            Fill::Empty => seg(false, 1).into(),
            Fill::Full => seg(true, 1).into(),
            Fill::Partial(f) => {
                let a = ((f * 1000.0) as u16).max(1);
                let b = (((1.0 - f) * 1000.0) as u16).max(1);
                if horizontal {
                    row![seg(true, a), seg(false, b)].into()
                } else {
                    column![seg(true, a), seg(false, b)].into()
                }
            }
        };
        if horizontal {
            let c = container(inner).height(Length::Fixed(2.0));
            match fixed {
                Some(w) => c.width(Length::Fixed(w)),
                None => c.width(Length::Fill),
            }
            .into()
        } else {
            container(inner)
                .width(Length::Fixed(2.0))
                .height(Length::Fill)
                .into()
        }
    }

    /// Estimated width of step `i` with its text.
    fn estimate(&self, i: usize) -> f32 {
        let it = &self.items[i];
        let size = (self.m.space_m + 1.0) * self.scale;
        let label = it.label.chars().count() as f32 * size * 0.58;
        let desc = it
            .description
            .as_ref()
            .map_or(0.0, |d| d.chars().count() as f32 * (size - 2.0) * 0.55);
        self.diameter + self.m.space_s * 3.0 + label.max(desc)
    }

    /// The row of steps joined by lines.
    fn horizontal<'a>(&self, compact: bool, wrap: bool) -> Element<'a, M>
    where
        M: 'a,
    {
        let n = self.items.len();
        let mut r = row![].align_y(Alignment::Center).spacing(self.m.space_s);
        for i in 0..n {
            let show_text = !compact || self.items[i].status == Status::Current;
            r = r.push(self.head(i, !show_text, false, false));
            if i + 1 < n {
                let w = if wrap {
                    Some(self.m.space_xl)
                } else {
                    compact.then_some(self.m.space_s * 2.0)
                };
                r = r.push(self.line(self.fills[i], true, w));
            }
        }
        if wrap {
            r.wrap().vertical_spacing(self.m.space_xs).into()
        } else {
            r.width(Length::Fill).into()
        }
    }

    /// The height of the one-line horizontal indicator.
    fn line_height(&self) -> f32 {
        let size = (self.m.space_m + 1.0) * self.scale;
        let has_desc = self.items.iter().any(|i| i.description.is_some());
        let text_h = size * 1.4
            + if has_desc {
                (size - 2.0) * 1.4 + self.m.space_xxs
            } else {
                0.0
            };
        self.diameter.max(text_h) + 2.0 * self.m.space_xs
    }

    /// The steps stacked, each followed by its connector line and `contents[i]`.
    fn vertical<'a>(&self, mut contents: Vec<Option<Element<'a, M>>>) -> Element<'a, M>
    where
        M: 'a,
    {
        let n = self.items.len();
        let m = self.m;
        let mut col = column![].width(Length::Fill);
        for i in 0..n {
            col = col.push(self.head(i, false, true, true));
            let last = i + 1 == n;
            let content = contents.get_mut(i).and_then(Option::take);
            // Under the circle: the line (none after the last step).
            let rail: Element<'a, M> = if last {
                space().width(Length::Fixed(self.diameter)).into()
            } else {
                container(self.line(self.fills[i], false, None))
                    .width(Length::Fixed(self.diameter))
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .into()
            };
            let gap = if last { 0.0 } else { m.space_m };
            let body: Element<'a, M> = match content {
                Some(c) => container(c)
                    .width(Length::Fill)
                    .padding(Padding {
                        top: m.space_xs,
                        bottom: m.space_m + gap,
                        left: m.space_m,
                        right: m.space_m,
                    })
                    .into(),
                None => space()
                    .width(Length::Fill)
                    .height(Length::Fixed(m.space_m + gap))
                    .into(),
            };
            let (rail_pad, parts) = if self.right {
                (Padding::default().right(m.space_s), vec![body, rail])
            } else {
                (Padding::default().left(m.space_s), vec![rail, body])
            };
            let mut r = row![].width(Length::Fill);
            for p in parts {
                r = r.push(p);
            }
            col = col.push(container(r).padding(rail_pad).width(Length::Fill));
        }
        col.into()
    }
}

impl<'a, M: Clone + 'a> From<Stepper<'a, M>> for Element<'a, M> {
    fn from(s: Stepper<'a, M>) -> Self {
        let m = s.metrics;
        let count = s.steps.len();
        let active = prim::clamp(s.active, count);
        let diameter = s.icon_size.unwrap_or_else(|| s.size.diameter());
        let click_enabled = s.on_step_click.is_some();

        let items: Vec<Item<M>> = s
            .steps
            .iter()
            .enumerate()
            .map(|(i, st)| {
                let status = match &s.statuses {
                    Some(v) => v.get(i).copied().unwrap_or(Status::Upcoming),
                    None => prim::status(i, active, st.invalid, st.loading),
                };
                let allowed = match &s.clickable {
                    Some(v) => v.get(i).copied().unwrap_or(false) && i != active,
                    None => prim::can_click(
                        i,
                        active,
                        count,
                        s.allow_next,
                        if st.disabled {
                            Some(false)
                        } else {
                            st.allow_click
                        },
                    ),
                };
                Item {
                    label: st.label.clone(),
                    description: st.description.clone(),
                    icon: st.icon,
                    completed_icon: st.completed_icon,
                    status,
                    disabled: st.disabled,
                    click: if allowed && click_enabled {
                        s.on_step_click.as_ref().map(|f| f(i))
                    } else {
                        None
                    },
                }
            })
            .collect();
        let fills: Vec<Fill> = (0..count)
            .map(|i| {
                if s.statuses.is_some() {
                    if items[i].status == Status::Completed && i + 1 < count {
                        Fill::Full
                    } else {
                        Fill::Empty
                    }
                } else {
                    prim::connector_fill(i, active, count, s.progress)
                }
            })
            .collect();

        let content: Option<Element<'a, M>> = if prim::is_completed_screen(active, count) {
            s.completed.as_ref().map(|f| f())
        } else {
            s.steps
                .get(active)
                .and_then(|st| st.build.as_ref())
                .map(|f| f())
        };

        let look = Look {
            items,
            fills,
            diameter,
            scale: s.size.text_scale(),
            right: s.icon_position == StepperIconPosition::Right,
            m,
        };
        let gap = s.content_spacing.unwrap_or(m.space_l);

        match s.orientation {
            StepperOrientation::Vertical => {
                let mut contents: Vec<Option<Element<'a, M>>> = (0..count).map(|_| None).collect();
                let mut after: Option<Element<'a, M>> = None;
                if prim::is_completed_screen(active, count) {
                    after = content;
                } else if let Some(slot) = contents.get_mut(active) {
                    *slot = content;
                }
                let mut col = column![look.vertical(contents)]
                    .spacing(gap)
                    .width(Length::Fill);
                if let Some(c) = after {
                    col = col.push(c);
                }
                col.into()
            }
            StepperOrientation::Horizontal => {
                let indicator: Element<'a, M> = if s.wrap {
                    look.horizontal(false, true)
                } else if s.collapse && count > 1 {
                    let height = look.line_height();
                    responsive(move |size| {
                        let need: f32 =
                            (0..look.items.len()).map(|i| look.estimate(i)).sum::<f32>()
                                + look.m.space_l * look.items.len().saturating_sub(1) as f32;
                        container(look.horizontal(need > size.width, false))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_y(Alignment::Center)
                            .into()
                    })
                    .width(Length::Fill)
                    .height(Length::Fixed(height))
                    .into()
                } else {
                    look.horizontal(false, false)
                };
                match content {
                    Some(c) => column![indicator, container(c).width(Length::Fill)]
                        .spacing(gap)
                        .width(Length::Fill)
                        .into(),
                    None => indicator,
                }
            }
        }
    }
}
