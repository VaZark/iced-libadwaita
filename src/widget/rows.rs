//! List rows: `AdwActionRow`, `AdwSwitchRow`, `AdwEntryRow`, `AdwComboRow`,
//! `AdwSpinRow`, `AdwExpanderRow`, `AdwButtonRow`, and `.boxed-list`.

use super::Element;
use crate::style::{button as sb, container as sc, pick_list as sp, text as st, text_input as si};
use crate::widget::typography;
use crate::{Icon, Metrics, Role, Theme, Typography};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, pick_list, row, space, text, text_input, toggler};
use iced::Length;

/// `.boxed-list`: rows in a rounded card separated by hairlines.
pub fn boxed_list<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let mut col = column![];
    for (i, r) in rows.into_iter().enumerate() {
        if i > 0 {
            col = col.push(container(space().height(1)).width(Length::Fill).style(sc::separator));
        }
        col = col.push(r);
    }
    container(col).width(Length::Fill).clip(true).style(sc::boxed_list).into()
}

fn row_frame<'a, M: 'a>(
    inner: impl Into<Element<'a, M>>,
    m: &Metrics,
) -> iced::widget::Row<'a, M, Theme> {
    row![
        space().height(m.row_min_height - m.row_padding.y()),
        inner.into()
    ]
    .spacing(0)
    .align_y(Vertical::Center)
}

/// `AdwActionRow`.
pub struct ActionRow<'a, M> {
    title: String,
    subtitle: Option<String>,
    prefix: Option<Element<'a, M>>,
    suffix: Option<Element<'a, M>>,
    on_press: Option<M>,
    chevron: bool,
    metrics: Metrics,
    typography: Typography,
}

/// Shorthand for [`ActionRow::new`].
pub fn action_row<'a, M>(title: impl Into<String>) -> ActionRow<'a, M> {
    ActionRow::new(title)
}

impl<'a, M> ActionRow<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            prefix: None,
            suffix: None,
            on_press: None,
            chevron: false,
            metrics: Metrics::default(),
            typography: Typography::default(),
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    /// `add_prefix`.
    pub fn prefix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.prefix = Some(e.into());
        self
    }
    /// `add_suffix`.
    pub fn suffix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.suffix = Some(e.into());
        self
    }
    /// Makes the row activatable.
    pub fn on_activate(mut self, m: M) -> Self {
        self.on_press = Some(m);
        self
    }
    /// Shows the navigation chevron (`go-next-symbolic`) at the end.
    pub fn chevron(mut self, show: bool) -> Self {
        self.chevron = show;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    pub fn typography(mut self, t: Typography) -> Self {
        self.typography = t;
        self
    }
}

impl<'a, M: Clone + 'a> From<ActionRow<'a, M>> for Element<'a, M> {
    fn from(r: ActionRow<'a, M>) -> Self {
        let m = r.metrics;
        let mut titles = column![r.typography.body(r.title)].spacing(2);
        if let Some(s) = r.subtitle {
            titles = titles.push(r.typography.caption(s).size(r.typography.body * 0.93).style(st::dim));
        }
        let mut line = row![].spacing(m.row_spacing).align_y(Vertical::Center);
        if let Some(p) = r.prefix {
            line = line.push(p);
        }
        line = line.push(titles.width(Length::Fill));
        if let Some(s) = r.suffix {
            line = line.push(s);
        }
        if r.chevron {
            line = line.push(Icon::GoNext.view(m.icon_size).style(|t: &Theme, _| {
                iced::widget::svg::Style { color: Some(t.semantic.dim_solid()) }
            }));
        }
        let framed = container(row_frame(line.width(Length::Fill), &m))
            .padding(m.row_padding)
            .width(Length::Fill);
        match r.on_press {
            Some(msg) => button(framed)
                .padding(0)
                .width(Length::Fill)
                .style(sb::list_row)
                .on_press(msg)
                .into(),
            None => framed.into(),
        }
    }
}

/// `AdwSwitchRow`.
pub struct SwitchRow<'a, M> {
    row: ActionRow<'a, M>,
    active: bool,
    on_toggle: Box<dyn Fn(bool) -> M + 'a>,
    metrics: Metrics,
}

impl<'a, M> SwitchRow<'a, M> {
    pub fn new(
        title: impl Into<String>,
        active: bool,
        on_toggle: impl Fn(bool) -> M + 'a,
    ) -> Self {
        Self {
            row: ActionRow::new(title),
            active,
            on_toggle: Box::new(on_toggle),
            metrics: Metrics::default(),
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
    pub fn prefix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.row = self.row.prefix(e);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self.row = self.row.metrics(m);
        self
    }
}

impl<'a, M: Clone + 'a> From<SwitchRow<'a, M>> for Element<'a, M> {
    fn from(s: SwitchRow<'a, M>) -> Self {
        let toggle_to = (s.on_toggle)(!s.active);
        let sw = toggler(s.active)
            .on_toggle(s.on_toggle)
            .size(s.metrics.switch_size);
        s.row.suffix(sw).on_activate(toggle_to).into()
    }
}

/// `AdwButtonRow`: a centered action row.
pub struct ButtonRow<M> {
    title: String,
    on_press: Option<M>,
    role: Role,
    metrics: Metrics,
}

impl<M> ButtonRow<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            on_press: None,
            role: Role::Accent,
            metrics: Metrics::default(),
        }
    }
    pub fn on_activate(mut self, m: M) -> Self {
        self.on_press = Some(m);
        self
    }
    /// `.suggested-action` ([`Role::Accent`]) or `.destructive-action`
    /// ([`Role::Destructive`]) coloring.
    pub fn role(mut self, r: Role) -> Self {
        self.role = r;
        self
    }
}

impl<'a, M: Clone + 'a> From<ButtonRow<M>> for Element<'a, M> {
    fn from(b: ButtonRow<M>) -> Self {
        let m = b.metrics;
        let role = b.role;
        let label = text(b.title).style(move |t: &Theme| iced::widget::text::Style {
            color: Some(t.semantic.swatch(role).color),
        });
        let inner = container(row_frame(label, &m))
            .padding(m.row_padding)
            .width(Length::Fill)
            .align_x(Horizontal::Center);
        let mut btn = button(inner).padding(0).width(Length::Fill).style(sb::list_row);
        if let Some(msg) = b.on_press {
            btn = btn.on_press(msg);
        }
        btn.into()
    }
}

/// `AdwEntryRow`: a title that floats above an inline entry.
pub struct EntryRow<'a, M> {
    title: String,
    value: String,
    on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
    on_submit: Option<M>,
    secure: bool,
    suffix: Option<Element<'a, M>>,
    metrics: Metrics,
}

impl<'a, M> EntryRow<'a, M> {
    pub fn new(title: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            on_input: None,
            on_submit: None,
            secure: false,
            suffix: None,
            metrics: Metrics::default(),
        }
    }
    pub fn on_input(mut self, f: impl Fn(String) -> M + 'a) -> Self {
        self.on_input = Some(Box::new(f));
        self
    }
    pub fn on_submit(mut self, m: M) -> Self {
        self.on_submit = Some(m);
        self
    }
    /// `AdwPasswordEntryRow`.
    pub fn password(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
    pub fn suffix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.suffix = Some(e.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<EntryRow<'a, M>> for Element<'a, M> {
    fn from(e: EntryRow<'a, M>) -> Self {
        let m = e.metrics;
        let t = Typography::default();
        let mut input = text_input("", &e.value)
            .padding(0)
            .secure(e.secure)
            .style(si::embedded);
        if let Some(f) = e.on_input {
            input = input.on_input(f);
        }
        if let Some(s) = e.on_submit {
            input = input.on_submit(s);
        }
        let mut line = row![column![
            t.caption(e.title).style(st::dim),
            input
        ]
        .spacing(2)
        .width(Length::Fill)]
        .spacing(m.row_spacing)
        .align_y(Vertical::Center);
        if let Some(s) = e.suffix {
            line = line.push(s);
        }
        container(row_frame(line.width(Length::Fill), &m))
            .padding(m.row_padding)
            .width(Length::Fill)
            .into()
    }
}

/// `AdwComboRow`.
pub struct ComboRow<'a, T, M>
where
    T: ToString + PartialEq + Clone,
{
    row: ActionRow<'a, M>,
    options: &'a [T],
    selected: Option<T>,
    on_select: Box<dyn Fn(T) -> M + 'a>,
}

impl<'a, T: ToString + PartialEq + Clone, M> ComboRow<'a, T, M> {
    pub fn new(
        title: impl Into<String>,
        options: &'a [T],
        selected: Option<T>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        Self {
            row: ActionRow::new(title),
            options,
            selected,
            on_select: Box::new(on_select),
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
}

impl<'a, T, M> From<ComboRow<'a, T, M>> for Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    fn from(c: ComboRow<'a, T, M>) -> Self {
        let pl = pick_list(c.options, c.selected, c.on_select).style(sp::flat);
        c.row.suffix(pl).into()
    }
}

/// `AdwSpinRow`.
pub struct SpinRow<'a, M> {
    row: ActionRow<'a, M>,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    digits: usize,
    on_change: Box<dyn Fn(f64) -> M + 'a>,
}

impl<'a, M> SpinRow<'a, M> {
    pub fn new(
        title: impl Into<String>,
        value: f64,
        range: std::ops::RangeInclusive<f64>,
        step: f64,
        on_change: impl Fn(f64) -> M + 'a,
    ) -> Self {
        Self {
            row: ActionRow::new(title),
            value,
            min: *range.start(),
            max: *range.end(),
            step,
            digits: 0,
            on_change: Box::new(on_change),
        }
    }
    pub fn digits(mut self, d: usize) -> Self {
        self.digits = d;
        self
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
}

impl<'a, M: Clone + 'a> From<SpinRow<'a, M>> for Element<'a, M> {
    fn from(s: SpinRow<'a, M>) -> Self {
        let dec = (s.value - s.step).max(s.min);
        let inc = (s.value + s.step).min(s.max);
        let mut minus = super::circular_button(Icon::ListRemove);
        let mut plus = super::circular_button(Icon::ListAdd);
        if s.value > s.min {
            minus = minus.on_press((s.on_change)(dec));
        }
        if s.value < s.max {
            plus = plus.on_press((s.on_change)(inc));
        }
        let ctl = row![
            minus,
            container(typography::body(format!("{:.*}", s.digits, s.value)))
                .width(Length::Fixed(44.0))
                .align_x(Horizontal::Center),
            plus
        ]
        .align_y(Vertical::Center);
        s.row.suffix(ctl).into()
    }
}

/// `AdwExpanderRow`. Stateless: the app owns `expanded` and handles `on_toggle`.
pub struct ExpanderRow<'a, M> {
    row: ActionRow<'a, M>,
    expanded: bool,
    on_toggle: Box<dyn Fn(bool) -> M + 'a>,
    children: Vec<Element<'a, M>>,
}

impl<'a, M> ExpanderRow<'a, M> {
    pub fn new(
        title: impl Into<String>,
        expanded: bool,
        on_toggle: impl Fn(bool) -> M + 'a,
    ) -> Self {
        Self {
            row: ActionRow::new(title),
            expanded,
            on_toggle: Box::new(on_toggle),
            children: Vec::new(),
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
    pub fn prefix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.row = self.row.prefix(e);
        self
    }
    /// `add_row`.
    pub fn push(mut self, row: impl Into<Element<'a, M>>) -> Self {
        self.children.push(row.into());
        self
    }
}

impl<'a, M: Clone + 'a> From<ExpanderRow<'a, M>> for Element<'a, M> {
    fn from(x: ExpanderRow<'a, M>) -> Self {
        let icon = if x.expanded { Icon::GoUp } else { Icon::GoDown };
        let header = x
            .row
            .suffix(icon.view(16.0))
            .on_activate((x.on_toggle)(!x.expanded));
        let mut col = column![Element::from(header)];
        if x.expanded && !x.children.is_empty() {
            let mut inner = column![];
            for c in x.children {
                inner = inner
                    .push(container(space().height(1)).width(Length::Fill).style(sc::separator))
                    .push(c);
            }
            col = col.push(container(inner).width(Length::Fill).style(|t: &Theme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(crate::color::alpha(
                        t.semantic.card.fg,
                        0.03,
                    ))),
                    ..Default::default()
                }
            }));
        }
        col.into()
    }
}

#[allow(dead_code)]
fn _unused() {
    let _ = sb::default;
}
