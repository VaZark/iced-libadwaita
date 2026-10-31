//! List rows: `AdwActionRow`, `AdwSwitchRow`, `AdwEntryRow`, `AdwComboRow`,
//! `AdwSpinRow`, `AdwExpanderRow`, `AdwButtonRow`, and `.boxed-list`.

use super::Element;
use crate::style::{button as sb, container as sc, text as st, text_input as si};
use crate::widget::typography;
use crate::{CollapseBehavior, Icon, Metrics, Role, Theme, Typography};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, space, text, text_input};
use iced::Length;

/// `.boxed-list`: rows in a rounded card separated by hairlines (a [`Card`](super::Card)
/// of rows with no padding).
pub fn boxed_list<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let mut rows = rows.into_iter();
    let Some(first) = rows.next() else { return space().into() };
    rows.fold(super::Card::new(first).padding(0), |card, r| card.push(r)).into()
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
    wrap: bool,
    collapse: CollapseBehavior,
    suffix_align: Horizontal,
    suffix_basis: Option<f32>,
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
            wrap: true,
            collapse: Metrics::desktop().row_collapse,
            suffix_align: Horizontal::Left,
            suffix_basis: None,
            metrics: Metrics::desktop(),
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
    /// Whether the suffix wraps to its own line when it can't share one with the
    /// title (default true, like CSS `flex-wrap`).
    pub fn wrap_suffix(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    /// What the suffix does when it wraps: [`CollapseBehavior::Stacked`] (100%
    /// width on its own line, the default) or [`CollapseBehavior::Flex`] (keeps
    /// its own width).
    pub fn collapse(mut self, c: CollapseBehavior) -> Self {
        self.collapse = c;
        self
    }
    /// Where a *wrapped* suffix sits on its own line: `Horizontal::Left` (start,
    /// default) or `Horizontal::Right` (locked to the end edge, like `flex-end`).
    /// Only matters for [`CollapseBehavior::Flex`]; a stacked suffix is 100% wide.
    /// While sharing a line with the title, the suffix is always pinned to the end.
    pub fn suffix_align(mut self, a: Horizontal) -> Self {
        self.suffix_align = a;
        self
    }
    /// The width a `Fill`-width suffix (e.g. a slider) takes while sharing a line
    /// with the title. Required for such suffixes so the row can tell when they fit.
    pub fn suffix_basis(mut self, px: f32) -> Self {
        self.suffix_basis = Some(px);
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
        let mut titles = column![r.typography.body(r.title).wrapping(text::Wrapping::WordOrGlyph)].spacing(2);
        if let Some(s) = r.subtitle {
            titles = titles.push(
                r.typography
                    .caption(s)
                    .size(r.typography.body * 0.93)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .style(st::dim),
            );
        }
        let mut main = row![].spacing(m.row_spacing).align_y(Vertical::Center);
        if let Some(p) = r.prefix {
            main = main.push(p);
        }
        main = main.push(titles.width(Length::Fill));
        if r.chevron {
            main = main.push(Icon::GoNext.view(m.icon_size).style(|t: &Theme, _| {
                iced::widget::svg::Style { color: Some(t.semantic.dim_solid()) }
            }));
        }
        let line: Element<'a, M> = match r.suffix {
            Some(suffix) if r.wrap => {
                let mut f = super::FlexRow::new(main.width(Length::Fill), suffix)
                    .spacing(m.row_spacing)
                    .line_spacing(m.space_s)
                    .main_basis(m.row_wrap_basis)
                    .collapse(r.collapse)
                    .aside_align(r.suffix_align);
                if let Some(b) = r.suffix_basis {
                    f = f.aside_basis(b);
                }
                f.into()
            }
            Some(suffix) => main.push(suffix).width(Length::Fill).into(),
            None => main.width(Length::Fill).into(),
        };
        let framed = container(row_frame(line, &m))
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
    disabled: bool,
    metrics: Metrics,
}

impl<'a, M> SwitchRow<'a, M> {
    pub fn new(
        title: impl Into<String>,
        active: bool,
        on_toggle: impl Fn(bool) -> M + 'a,
    ) -> Self {
        Self {
            // Compact fixed-size suffix: pin it to the end, no flex wrapping.
            row: ActionRow::new(title).wrap_suffix(false),
            active,
            on_toggle: Box::new(on_toggle),
            disabled: false,
            metrics: Metrics::desktop(),
        }
    }
    /// Insensitive: the switch is drawn disabled and the row cannot be activated.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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
        if s.disabled {
            return s.row.suffix(super::switch::<M>(s.active)).into();
        }
        let toggle_to = (s.on_toggle)(!s.active);
        let sw = super::switch(s.active).on_toggle(s.on_toggle);
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
            metrics: Metrics::desktop(),
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
        let label = typography::label(b.title).style(move |t: &Theme| iced::widget::text::Style {
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
    collapse: CollapseBehavior,
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
            collapse: Metrics::desktop().row_collapse,
            metrics: Metrics::desktop(),
        }
    }
    /// Behavior of the suffix when it wraps (see [`CollapseBehavior`]).
    pub fn collapse(mut self, c: CollapseBehavior) -> Self {
        self.collapse = c;
        self
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
        let main = column![t.caption(e.title).style(st::dim), input]
            .spacing(2)
            .width(Length::Fill);
        let line: Element<'a, M> = match e.suffix {
            Some(s) => super::FlexRow::new(main, s)
                .spacing(m.row_spacing)
                .line_spacing(m.space_s)
                .main_basis(m.row_wrap_basis)
                .collapse(e.collapse)
                .into(),
            None => main.into(),
        };
        container(row_frame(line, &m))
            .padding(m.row_padding)
            .width(Length::Fill)
            .into()
    }
}

type OptionPredicate<'a, T> = Box<dyn Fn(&T) -> bool + 'a>;

/// `AdwComboRow`.
pub struct ComboRow<'a, T, M>
where
    T: ToString + PartialEq + Clone,
{
    row: ActionRow<'a, M>,
    options: &'a [T],
    selected: Option<T>,
    on_select: Box<dyn Fn(T) -> M + 'a>,
    disabled: Option<OptionPredicate<'a, T>>,
    insensitive: bool,
    open_initially: bool,
}

impl<'a, T: ToString + PartialEq + Clone, M> ComboRow<'a, T, M> {
    pub fn new(
        title: impl Into<String>,
        options: &'a [T],
        selected: Option<T>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        Self {
            // Compact fixed-size suffix: pin it to the end, no flex wrapping.
            row: ActionRow::new(title).wrap_suffix(false),
            options,
            selected,
            on_select: Box::new(on_select),
            disabled: None,
            insensitive: false,
            open_initially: false,
        }
    }
    /// Insensitive: the value is dimmed and the popover cannot open.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.insensitive = disabled;
        self
    }
    /// Options for which `f` is true are shown dimmed and cannot be chosen.
    pub fn disabled_when(mut self, f: impl Fn(&T) -> bool + 'a) -> Self {
        self.disabled = Some(Box::new(f));
        self
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
    /// Start with the popover open (screenshots, demos).
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }
}

impl<'a, T, M> From<ComboRow<'a, T, M>> for Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    fn from(c: ComboRow<'a, T, M>) -> Self {
        let m = Metrics::desktop();
        let current = c.selected.as_ref().map(ToString::to_string).unwrap_or_default();
        let value = row![
            // The chosen value is live content, in the normal text color (like text in an
            // entry); dimmed is reserved for secondary text and the disabled state.
            if c.insensitive { typography::label(current).style(st::dim) } else { typography::label(current) },
            Icon::GoDown.view(m.icon_size).style(move |t: &Theme, _| iced::widget::svg::Style {
                color: Some(if c.insensitive {
                    crate::color::fade(t.semantic.window.fg, t.semantic.opacity.disabled)
                } else {
                    t.semantic.window.fg
                }),
            }),
        ]
        .spacing(m.space_s)
        .align_y(Vertical::Center);
        let header: Element<'a, M> = c.row.suffix(value).into();
        let on_select = c.on_select;
        let selected = c.selected;
        let disabled = c.disabled;
        let items = c.options.iter().map(|o| {
            let is_selected = selected.as_ref() == Some(o);
            if disabled.as_ref().is_some_and(|f| f(o)) {
                super::menu_item_disabled(o.to_string(), is_selected)
            } else {
                super::menu_item(o.to_string(), is_selected, on_select(o.clone()))
            }
        });
        super::Dropdown::custom(header, super::popover(items, 280.0))
            .appearance(super::DropdownAppearance::Row)
            .align(super::PopoverAlign::End)
            .popover_width(240.0)
            .open_initially(c.open_initially)
            .disabled(c.insensitive)
            .into()
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
            // Compact fixed-size suffix: pin it to the end, no flex wrapping.
            row: ActionRow::new(title).wrap_suffix(false),
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
            // Compact fixed-size suffix: pin it to the end, no flex wrapping.
            row: ActionRow::new(title).wrap_suffix(false),
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
        if !x.children.is_empty() {
            let mut inner = column![];
            for c in x.children {
                inner = inner
                    .push(container(space().height(1)).width(Length::Fill).style(sc::card_separator))
                    .push(c);
            }
            let list = container(inner).width(Length::Fill).style(|t: &Theme| {
                iced::widget::container::Style {
                    background: Some(iced::Background::Color(crate::color::alpha(
                        t.semantic.card.fg,
                        0.03,
                    ))),
                    ..Default::default()
                }
            });
            // Always mounted: the revealer slides it open/closed (250ms, ADW_EASE).
            col = col.push(Element::from(super::reveal(list, x.expanded)));
        }
        col.into()
    }
}

#[allow(dead_code)]
fn _unused() {
    let _ = sb::default;
}

/// A row with a slider (`GtkScale`) as its suffix: `title`, optional `subtitle`, and a
/// `Fill`-width scale. While the title and scale can't share a line the scale wraps
/// below and stretches to 100% (see [`CollapseBehavior`]).
pub struct SliderRow<'a, M> {
    row: ActionRow<'a, M>,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    step: f32,
    on_change: Box<dyn Fn(f32) -> M + 'a>,
    basis: f32,
}

impl<'a, M> SliderRow<'a, M> {
    pub fn new(
        title: impl Into<String>,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        on_change: impl Fn(f32) -> M + 'a,
    ) -> Self {
        Self {
            row: ActionRow::new(title),
            value,
            range,
            step: 1.0,
            on_change: Box::new(on_change),
            basis: 160.0,
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }
    /// The width the scale takes while sharing a line with the title.
    pub fn basis(mut self, px: f32) -> Self {
        self.basis = px;
        self
    }
    pub fn collapse(mut self, c: CollapseBehavior) -> Self {
        self.row = self.row.collapse(c);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.row = self.row.metrics(m);
        self
    }
}

impl<'a, M: Clone + 'a> From<SliderRow<'a, M>> for Element<'a, M> {
    fn from(s: SliderRow<'a, M>) -> Self {
        let scale = iced::widget::slider(s.range, s.value, s.on_change)
            .step(s.step)
            .width(Length::Fill);
        s.row.suffix(scale).suffix_basis(s.basis).into()
    }
}

/// A row with a toggle group (segmented control) as its suffix. The group stretches to
/// 100% (equal `flex: 1` segments) when it wraps below the title.
pub struct ToggleGroupRow<'a, T, M> {
    row: ActionRow<'a, M>,
    group: super::ToggleGroup<'a, T, M>,
}

impl<'a, T: Clone + PartialEq, M> ToggleGroupRow<'a, T, M> {
    pub fn new(
        title: impl Into<String>,
        items: Vec<super::ToggleItem<'a, T, M>>,
        active: Option<T>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        Self {
            row: ActionRow::new(title),
            group: super::toggle_group(items, active, on_select),
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
    pub fn homogeneous(mut self, h: bool) -> Self {
        self.group = self.group.homogeneous(h);
        self
    }
    pub fn collapse(mut self, c: CollapseBehavior) -> Self {
        self.row = self.row.collapse(c);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.row = self.row.metrics(m);
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, M: Clone + 'a> From<ToggleGroupRow<'a, T, M>> for Element<'a, M> {
    fn from(t: ToggleGroupRow<'a, T, M>) -> Self {
        t.row.suffix(Element::from(t.group)).into()
    }
}
