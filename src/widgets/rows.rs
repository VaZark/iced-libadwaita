//! List rows: `AdwActionRow`, `AdwSwitchRow`, `AdwEntryRow`, `AdwComboRow`,
//! `AdwSpinRow`, `AdwExpanderRow`, `AdwButtonRow`, and `.boxed-list`.

use crate::Element;
use crate::components::typography;
use crate::styles::{button as sb, container as sc, text as st};
use crate::{CollapseBehavior, Icon, Metrics, Role, Theme, Typography};
use iced::Length;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, space, text};

/// `.boxed-list`: rows in a rounded card separated by hairlines (a [`Card`](crate::widgets::Card)
/// of rows with no padding).
pub fn boxed_list<'a, M: 'a>(rows: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    let mut rows = rows.into_iter();
    let Some(first) = rows.next() else {
        return space().into();
    };
    rows.fold(crate::widgets::Card::new(first).padding(0), |card, r| {
        card.push(r)
    })
    .into()
}

pub(crate) fn row_frame<'a, M: 'a>(
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

/// The spacing and padding of the inner [`ActionRow`] of a row built on one.
macro_rules! forward_row_spacing {
    () => {
        pub fn label_props(mut self, props: crate::components::TextProps) -> Self {
            self.row = self.row.label_props(props);
            self
        }
        pub fn button_props(mut self, props: crate::components::ButtonProps) -> Self {
            self.row = self.row.button_props(props);
            self
        }
        pub fn row_props(mut self, props: ActionRowProps) -> Self {
            self.row = props.apply(self.row);
            self
        }
        /// The gap between the row's prefix, title block and suffix (default: the theme's
        /// `row_spacing`).
        pub fn spacing(mut self, px: f32) -> Self {
            self.row = self.row.spacing(px);
            self
        }
        /// The gap between the title and the subtitle (default: the theme's `space_xxs`).
        pub fn title_spacing(mut self, px: f32) -> Self {
            self.row = self.row.title_spacing(px);
            self
        }
        /// The padding around the row's content (default: the theme's `row_padding`).
        pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
            self.row = self.row.padding(p);
            self
        }
    };
}

/// `AdwActionRow`.
pub struct ActionRow<'a, M> {
    label_props: crate::components::TextProps,
    subtitle_props: crate::components::TextProps,
    button_props: crate::components::ButtonProps,
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
    spacing: Option<f32>,
    title_spacing: Option<f32>,
    line_spacing: Option<f32>,
    padding: Option<iced::Padding>,
    metrics: Metrics,
    typography: Typography,
}

/// Shorthand for [`ActionRow::new`].
pub fn action_row<'a, M>(title: impl Into<String>) -> ActionRow<'a, M> {
    ActionRow::new(title)
}

impl<'a, M> ActionRow<'a, M> {
    pub fn label_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = props;
        self
    }

    pub fn subtitle_props(mut self, props: crate::components::TextProps) -> Self {
        self.subtitle_props = props;
        self
    }

    pub fn button_props(mut self, props: crate::components::ButtonProps) -> Self {
        self.button_props = props;
        self
    }

    /// A row that opens a destination. Value actions belong in `EntryRow::action`.
    pub fn navigation(title: impl Into<String>, destination: M) -> Self {
        Self::new(title).chevron(true).on_activate(destination)
    }
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            label_props: Default::default(),
            subtitle_props: Default::default(),
            button_props: Default::default(),

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
            spacing: None,
            title_spacing: None,
            line_spacing: None,
            padding: None,
            metrics: Metrics::desktop(),
            typography: Typography::desktop(),
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
    /// The gap between the prefix, the title block, the chevron and the suffix (default: the
    /// theme's `row_spacing`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The gap between the title and the subtitle (default: the theme's `space_xxs`).
    pub fn title_spacing(mut self, px: f32) -> Self {
        self.title_spacing = Some(px.max(0.0));
        self
    }
    /// The gap between the title line and a suffix that wrapped below it (default: the theme's
    /// `space_s`).
    pub fn line_spacing(mut self, px: f32) -> Self {
        self.line_spacing = Some(px.max(0.0));
        self
    }
    /// The padding around the row's content (default: the theme's `row_padding`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
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
        let mut titles = column![
            r.label_props.apply(
                r.typography
                    .body(r.title)
                    .wrapping(text::Wrapping::WordOrGlyph)
            )
        ]
        .spacing(r.title_spacing.unwrap_or(m.space_xxs));
        if let Some(s) = r.subtitle {
            titles = titles.push(
                r.subtitle_props.apply(
                    r.typography
                        .caption(s)
                        .size(r.typography.body * 0.93)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .style(st::dim),
                ),
            );
        }
        let gap = r.spacing.unwrap_or(m.row_spacing);
        let mut main = row![].spacing(gap).align_y(Vertical::Center);
        if let Some(p) = r.prefix {
            main = main.push(p);
        }
        main = main.push(titles.width(Length::Fill));
        if r.chevron {
            main = main.push(Icon::GoNext.view(m.icon_size).style(|t: &Theme, _| {
                iced::widget::svg::Style {
                    color: Some(t.semantic.dim()),
                }
            }));
        }
        let line: Element<'a, M> = match r.suffix {
            Some(suffix) if r.wrap => {
                let mut f = crate::layouts::FlexRow::new(main.width(Length::Fill), suffix)
                    .spacing(gap)
                    .line_spacing(r.line_spacing.unwrap_or(m.space_s))
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
            .padding(r.padding.unwrap_or(m.row_padding))
            .width(Length::Fill);
        match r.on_press {
            Some(msg) => r
                .button_props
                .apply(
                    button(framed)
                        .padding(0)
                        .width(Length::Fill)
                        .style(sb::list_row)
                        .on_press(msg),
                )
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
    forward_row_spacing!();
    pub fn new(title: impl Into<String>, active: bool, on_toggle: impl Fn(bool) -> M + 'a) -> Self {
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
            return s
                .row
                .suffix(crate::components::switch::<M>(s.active))
                .into();
        }
        let toggle_to = (s.on_toggle)(!s.active);
        let sw = crate::components::switch(s.active).on_toggle(s.on_toggle);
        s.row.suffix(sw).on_activate(toggle_to).into()
    }
}

/// `AdwButtonRow`: a centered action row.
pub struct ButtonRow<M> {
    label_props: crate::components::TextProps,
    button_props: crate::components::ButtonProps,
    title: String,
    on_press: Option<M>,
    role: Role,
    padding: Option<iced::Padding>,
    metrics: Metrics,
}

impl<M> ButtonRow<M> {
    pub fn label_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = props;
        self
    }

    pub fn button_props(mut self, props: crate::components::ButtonProps) -> Self {
        self.button_props = props;
        self
    }

    pub fn new(title: impl Into<String>) -> Self {
        Self {
            label_props: Default::default(),
            button_props: Default::default(),
            title: title.into(),
            on_press: None,
            role: Role::Primary,
            padding: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn on_activate(mut self, m: M) -> Self {
        self.on_press = Some(m);
        self
    }
    /// The padding around the label (default: the theme's `row_padding`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// `.suggested-action` ([`Role::Primary`]) or `.destructive-action`
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
        let inner = container(row_frame(b.label_props.apply(label), &m))
            .padding(b.padding.unwrap_or(m.row_padding))
            .width(Length::Fill)
            .align_x(Horizontal::Center);
        let mut btn = button(inner)
            .padding(0)
            .width(Length::Fill)
            .style(sb::list_row);
        if let Some(msg) = b.on_press {
            btn = btn.on_press(msg);
        }
        b.button_props.apply(btn).into()
    }
}

type OptionPredicate<'a, T> = Box<dyn Fn(&T) -> bool + 'a>;

/// `AdwComboRow`.
pub struct ComboRow<'a, T, M>
where
    T: ToString + PartialEq + Clone,
{
    dropdown_props: super::dropdown::DropdownItemProps,
    label_props: crate::components::TextProps,
    row: ActionRow<'a, M>,
    options: &'a [T],
    selected: Option<T>,
    on_select: Box<dyn Fn(T) -> M + 'a>,
    disabled: Option<OptionPredicate<'a, T>>,
    insensitive: bool,
    open_initially: bool,
    value_spacing: Option<f32>,
}

impl<'a, T: ToString + PartialEq + Clone, M> ComboRow<'a, T, M> {
    pub fn dropdown_props(mut self, props: super::dropdown::DropdownItemProps) -> Self {
        self.dropdown_props = props;
        self
    }

    pub fn value_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = props;
        self
    }

    forward_row_spacing!();
    pub fn new(
        title: impl Into<String>,
        options: &'a [T],
        selected: Option<T>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        Self {
            dropdown_props: Default::default(),
            label_props: Default::default(),

            // Compact fixed-size suffix: pin it to the end, no flex wrapping.
            row: ActionRow::new(title).wrap_suffix(false),
            options,
            selected,
            on_select: Box::new(on_select),
            disabled: None,
            insensitive: false,
            open_initially: false,
            value_spacing: None,
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
    /// The gap between the chosen value and the arrow (default: the theme's `space_s`).
    pub fn value_spacing(mut self, px: f32) -> Self {
        self.value_spacing = Some(px.max(0.0));
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
        let current = c
            .selected
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        let value = row![
            // The chosen value is live content, in the normal text color (like text in an
            // entry); dimmed is reserved for secondary text and the disabled state.
            c.label_props.apply(if c.insensitive {
                typography::label(current).style(st::dim)
            } else {
                typography::label(current)
            }),
            Icon::GoDown
                .view(m.icon_size)
                .style(move |t: &Theme, _| iced::widget::svg::Style {
                    color: Some(if c.insensitive {
                        crate::color::fade(t.semantic.window.fg, t.semantic.opacity.disabled)
                    } else {
                        t.semantic.window.fg
                    }),
                }),
        ]
        .spacing(c.value_spacing.unwrap_or(m.space_s))
        .align_y(Vertical::Center);
        let header: Element<'a, M> = c.row.suffix(value).into();
        let on_select = c.on_select;
        let selected = c.selected;
        let disabled = c.disabled;
        let items = c.options.iter().map(|o| {
            let is_selected = selected.as_ref() == Some(o);
            if disabled.as_ref().is_some_and(|f| f(o)) {
                super::dropdown::menu_item_disabled_with_props(
                    o.to_string(),
                    is_selected,
                    c.dropdown_props.clone(),
                )
            } else {
                super::dropdown::menu_item_with_props(
                    o.to_string(),
                    is_selected,
                    on_select(o.clone()),
                    c.dropdown_props.clone(),
                )
            }
        });
        crate::widgets::Popover::new(header, crate::widgets::popover(items, 280.0))
            .appearance(crate::widgets::PopoverAppearance::Row)
            .align(crate::widgets::PopoverAlign::End)
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
    control_spacing: f32,
    value_width: f32,
}

impl<'a, M> SpinRow<'a, M> {
    forward_row_spacing!();
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
            control_spacing: 0.0,
            value_width: 44.0,
        }
    }
    pub fn digits(mut self, d: usize) -> Self {
        self.digits = d;
        self
    }
    /// The gap between the minus button, the value and the plus button (default 0).
    pub fn control_spacing(mut self, px: f32) -> Self {
        self.control_spacing = px.max(0.0);
        self
    }
    /// The width of the value between the buttons (default 44).
    pub fn value_width(mut self, px: f32) -> Self {
        self.value_width = px.max(0.0);
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
        let mut minus = crate::components::circular_button(Icon::ListRemove);
        let mut plus = crate::components::circular_button(Icon::ListAdd);
        if s.value > s.min {
            minus = minus.on_press((s.on_change)(dec));
        }
        if s.value < s.max {
            plus = plus.on_press((s.on_change)(inc));
        }
        let ctl = row![
            minus,
            container(typography::body(format!("{:.*}", s.digits, s.value)))
                .width(Length::Fixed(s.value_width))
                .align_x(Horizontal::Center),
            plus
        ]
        .spacing(s.control_spacing)
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
    indent: f32,
}

impl<'a, M> ExpanderRow<'a, M> {
    forward_row_spacing!();
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
            // The toolkit's default; `indent(..)` overrides it for this row.
            indent: Metrics::desktop().expander_indent,
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
    /// How far the nested rows are indented, in pixels. It starts at the toolkit's default,
    /// [`Metrics::expander_indent`] (the `[rows] expander-indent` desktop config); this
    /// overrides it for this row. 0 is libadwaita's flush nesting.
    pub fn indent(mut self, px: f32) -> Self {
        self.indent = px.max(0.0);
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
        if x.children.is_empty() {
            return Element::from(header);
        }
        let indent = x.indent;
        let mut inner = column![];
        for c in x.children {
            // The hairline spans the list; the row itself is indented.
            let c: Element<'a, M> = if indent > 0.0 {
                container(c)
                    .width(Length::Fill)
                    .padding(iced::Padding::ZERO.left(indent))
                    .into()
            } else {
                c
            };
            inner = inner
                .push(
                    container(space().height(1))
                        .width(Length::Fill)
                        .style(sc::card_separator),
                )
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
        // The disclosure primitive: always mounted, the revealer slides it (250ms, ADW_EASE).
        crate::widgets::Collapsible::new(Element::from(header), Element::from(list), x.expanded)
            .into()
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
    forward_row_spacing!();
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

/// `SliderRow` with two thumbs: a row with a title, an optional subtitle and a
/// [`RangeSlider`](crate::widgets::RangeSlider), with the current `lo – hi` shown next to it. Wraps
/// below the title like [`SliderRow`] (see [`CollapseBehavior`]).
pub struct RangeSliderRow<'a, M> {
    label_props: crate::components::TextProps,
    row: ActionRow<'a, M>,
    slider: crate::widgets::RangeSlider<'a, M>,
    value: (f32, f32),
    step: f32,
    format: Option<Box<dyn Fn(f32) -> String + 'a>>,
    bounds: (f32, f32),
    basis: f32,
    value_spacing: Option<f32>,
    metrics: Metrics,
}

impl<'a, M> RangeSliderRow<'a, M> {
    pub fn value_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = props;
        self
    }

    forward_row_spacing!();
    pub fn new(
        title: impl Into<String>,
        value: (f32, f32),
        range: std::ops::RangeInclusive<f32>,
        on_change: impl Fn((f32, f32)) -> M + 'a,
    ) -> Self {
        Self {
            label_props: Default::default(),
            row: ActionRow::new(title),
            bounds: (*range.start(), *range.end()),
            slider: crate::widgets::RangeSlider::new(range, value, on_change),
            value,
            step: 1.0,
            format: None,
            basis: 240.0,
            value_spacing: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.row = self.row.subtitle(s);
        self
    }
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self.slider = self.slider.step(step);
        self
    }
    pub fn min_gap(mut self, gap: f32) -> Self {
        self.slider = self.slider.min_gap(gap);
        self
    }
    pub fn collide(mut self, c: crate::primitives::range::Collide) -> Self {
        self.slider = self.slider.collide(c);
        self
    }
    pub fn marks(mut self, marks: impl IntoIterator<Item = f32>) -> Self {
        self.slider = self.slider.marks(marks);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.slider = self.slider.disabled(d);
        self
    }
    /// How one end is written (default: as many decimals as the step has).
    pub fn format(mut self, f: impl Fn(f32) -> String + 'a) -> Self {
        self.format = Some(Box::new(f));
        self
    }
    /// The gap between the slider and the `lo - hi` value (default: the theme's `space_m`).
    pub fn value_spacing(mut self, px: f32) -> Self {
        self.value_spacing = Some(px.max(0.0));
        self
    }
    /// The width the slider and the value take while sharing a line with the title.
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
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<RangeSliderRow<'a, M>> for Element<'a, M> {
    fn from(r: RangeSliderRow<'a, M>) -> Self {
        let decimals = {
            let mut d = 0;
            while d < 4 && (r.step * 10f32.powi(d) - (r.step * 10f32.powi(d)).round()).abs() > 1e-4
            {
                d += 1;
            }
            d as usize
        };
        let fmt = |v: f32| match &r.format {
            Some(f) => f(v),
            None => format!("{v:.decimals$}"),
        };
        let size = crate::Typography::desktop().small;
        // Wide enough for the longest end, so the slider does not move while dragging.
        let longest = fmt(r.bounds.0).len().max(fmt(r.bounds.1).len()) as f32;
        let label = container(
            r.label_props.apply(
                text(format!("{} \u{2013} {}", fmt(r.value.0), fmt(r.value.1)))
                    .size(size)
                    .wrapping(text::Wrapping::None)
                    .style(crate::styles::text::dim),
            ),
        )
        .width(Length::Fixed((2.0 * longest + 3.0) * size * 0.56))
        .align_x(Horizontal::Right);
        let both = row![r.slider, label]
            .spacing(r.value_spacing.unwrap_or(r.metrics.space_m))
            .align_y(Vertical::Center)
            .width(Length::Fill);
        r.row.suffix(both).suffix_basis(r.basis).into()
    }
}

/// A row with a toggle group (segmented control) as its suffix. The group stretches to
/// 100% (equal `flex: 1` segments) when it wraps below the title.
pub struct ToggleGroupRow<'a, T, M> {
    row: ActionRow<'a, M>,
    group: crate::components::ToggleGroup<'a, T, M>,
}

impl<'a, T: Clone + PartialEq, M> ToggleGroupRow<'a, T, M> {
    pub fn group_props(mut self, props: crate::components::button::ToggleGroupProps) -> Self {
        self.group = props.apply(self.group);
        self
    }
    forward_row_spacing!();
    pub fn new(
        title: impl Into<String>,
        items: Vec<crate::components::ToggleItem<'a, T, M>>,
        active: Option<T>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        Self {
            row: ActionRow::new(title),
            group: crate::components::toggle_group(items, active, on_select),
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

impl<'a, T: Clone + PartialEq + 'a, M: Clone + 'a> From<ToggleGroupRow<'a, T, M>>
    for Element<'a, M>
{
    fn from(t: ToggleGroupRow<'a, T, M>) -> Self {
        t.row.suffix(Element::from(t.group)).into()
    }
}

#[cfg(test)]
mod expander_indent_tests {
    use super::*;

    fn row() -> ExpanderRow<'static, u8> {
        ExpanderRow::new("A", true, |_| 0)
    }

    #[test]
    fn the_indent_starts_at_the_toolkit_default_and_a_row_can_override_it() {
        assert_eq!(row().indent, Metrics::desktop().expander_indent);
        assert_eq!(row().indent(12.0).indent, 12.0);
        assert_eq!(row().indent(0.0).indent, 0.0, "flush, like libadwaita");
        assert_eq!(row().indent(-5.0).indent, 0.0, "never negative");
    }

    #[test]
    fn the_toolkit_default_is_the_arrow_handle_and_the_config_can_change_it() {
        assert_eq!(
            Metrics::default().expander_indent,
            Metrics::default().icon_size + 8.0
        );
    }
}

#[cfg(test)]
mod spacing_params_tests {
    use super::*;

    fn each<'a>(row: impl Into<Element<'a, u8>>) {
        let _: Element<'a, u8> = row.into();
    }

    #[test]
    fn an_action_row_takes_its_gaps_and_padding() {
        let r = ActionRow::<u8>::new("A")
            .spacing(3.0)
            .title_spacing(1.0)
            .line_spacing(9.0)
            .padding([2.0, 4.0]);
        assert_eq!(
            (r.spacing, r.title_spacing, r.line_spacing),
            (Some(3.0), Some(1.0), Some(9.0))
        );
        assert_eq!(r.padding, Some(iced::Padding::from([2.0, 4.0])));
        assert_eq!(ActionRow::<u8>::new("A").spacing(-5.0).spacing, Some(0.0));
        each(r.subtitle("b").prefix(space()).suffix(space()));
    }

    #[test]
    fn rows_built_on_an_action_row_forward_the_params() {
        let o = [String::from("x")];
        each(SwitchRow::new("A", true, |_| 0).spacing(2.0).padding(3.0));
        each(
            ComboRow::new("A", &o, None, |_| 0)
                .spacing(2.0)
                .value_spacing(1.0),
        );
        each(
            ExpanderRow::new("A", true, |_| 0)
                .title_spacing(2.0)
                .padding(3.0),
        );
        each(SliderRow::new("A", 1.0, 0.0..=2.0, |_| 0).spacing(2.0));
        each(RangeSliderRow::new("A", (0.0, 1.0), 0.0..=2.0, |_| 0).value_spacing(2.0));
        each(
            ToggleGroupRow::new(
                "A",
                vec![crate::components::ToggleItem::new(1u8, "x")],
                None,
                |_| 0u8,
            )
            .padding(3.0),
        );
        let s = SwitchRow::<u8>::new("A", true, |_| 0).spacing(7.0);
        assert_eq!(s.row.spacing, Some(7.0));
        let c = ComboRow::new("A", &o, None, |_: String| 0u8).value_spacing(5.0);
        assert_eq!(c.value_spacing, Some(5.0));
    }

    #[test]
    fn a_spin_row_spaces_and_sizes_its_controls() {
        let s = SpinRow::<u8>::new("A", 1.0, 0.0..=2.0, 1.0, |_| 0)
            .control_spacing(6.0)
            .value_width(60.0);
        assert_eq!((s.control_spacing, s.value_width), (6.0, 60.0));
        each(s);
    }

    #[test]
    fn a_button_row_takes_padding() {
        let b = ButtonRow::<u8>::new("A").padding([1.0, 2.0]);
        assert_eq!(b.padding, Some(iced::Padding::from([1.0, 2.0])));
        each(b);
    }
}

/// Presentation-only overrides for an existing action row; content and actions remain owned.
#[derive(Clone, Default)]
pub struct ActionRowProps {
    pub label_props: Option<crate::components::TextProps>,
    pub subtitle_props: Option<crate::components::TextProps>,
    pub button_props: Option<crate::components::ButtonProps>,
    pub spacing: Option<f32>,
    pub title_spacing: Option<f32>,
    pub line_spacing: Option<f32>,
    pub padding: Option<iced::Padding>,
    pub collapse: Option<CollapseBehavior>,
    pub wrap_suffix: Option<bool>,
}
impl ActionRowProps {
    pub fn label_props(mut self, value: impl Into<crate::components::TextProps>) -> Self {
        self.label_props = Some(value.into());
        self
    }

    pub fn subtitle_props(mut self, value: impl Into<crate::components::TextProps>) -> Self {
        self.subtitle_props = Some(value.into());
        self
    }

    pub fn button_props(mut self, value: impl Into<crate::components::ButtonProps>) -> Self {
        self.button_props = Some(value.into());
        self
    }

    pub fn spacing(mut self, value: impl Into<f32>) -> Self {
        self.spacing = Some(value.into());
        self
    }

    pub fn title_spacing(mut self, value: impl Into<f32>) -> Self {
        self.title_spacing = Some(value.into());
        self
    }

    pub fn line_spacing(mut self, value: impl Into<f32>) -> Self {
        self.line_spacing = Some(value.into());
        self
    }

    pub fn padding(mut self, value: impl Into<iced::Padding>) -> Self {
        self.padding = Some(value.into());
        self
    }

    pub fn collapse(mut self, value: impl Into<CollapseBehavior>) -> Self {
        self.collapse = Some(value.into());
        self
    }

    pub fn wrap_suffix(mut self, value: impl Into<bool>) -> Self {
        self.wrap_suffix = Some(value.into());
        self
    }

    pub fn apply<'a, M>(&self, mut row: ActionRow<'a, M>) -> ActionRow<'a, M> {
        if let Some(props) = &self.label_props {
            row.label_props.merge(props);
        }
        if let Some(props) = &self.subtitle_props {
            row.subtitle_props.merge(props);
        }
        if let Some(props) = &self.button_props {
            row.button_props.merge(props);
        }
        if let Some(v) = self.spacing {
            row = row.spacing(v);
        }
        if let Some(v) = self.title_spacing {
            row = row.title_spacing(v);
        }
        if let Some(v) = self.line_spacing {
            row = row.line_spacing(v);
        }
        if let Some(v) = self.padding {
            row = row.padding(v);
        }
        if let Some(v) = self.collapse {
            row = row.collapse(v);
        }
        if let Some(v) = self.wrap_suffix {
            row = row.wrap_suffix(v);
        }
        row
    }
}
/// Presentation overrides for the concrete combo row, excluding selection and events.
#[derive(Clone, Default)]
pub struct ComboRowProps {
    pub dropdown_props: Option<super::dropdown::DropdownItemProps>,
    pub row_props: Option<ActionRowProps>,
    pub label_props: Option<crate::components::TextProps>,
    pub value_spacing: Option<f32>,
}
impl ComboRowProps {
    pub fn dropdown_props(mut self, props: super::dropdown::DropdownItemProps) -> Self {
        self.dropdown_props = Some(props);
        self
    }

    pub fn row_props(mut self, value: impl Into<ActionRowProps>) -> Self {
        self.row_props = Some(value.into());
        self
    }

    pub fn label_props(mut self, value: impl Into<crate::components::TextProps>) -> Self {
        self.label_props = Some(value.into());
        self
    }

    pub fn value_spacing(mut self, value: impl Into<f32>) -> Self {
        self.value_spacing = Some(value.into());
        self
    }

    pub fn apply<'a, T: ToString + PartialEq + Clone, M>(
        &self,
        mut row: ComboRow<'a, T, M>,
    ) -> ComboRow<'a, T, M> {
        if let Some(props) = &self.row_props {
            row.row = props.apply(row.row);
        }
        if let Some(props) = &self.label_props {
            row.row.label_props.merge(props);
        }
        if let Some(v) = self.value_spacing {
            row = row.value_spacing(v);
        }
        if let Some(props) = &self.dropdown_props {
            row.dropdown_props.merge(props);
        }
        row
    }
}
/// Presentation overrides for a concrete toggle-group row.
#[derive(Clone, Default)]
pub struct ToggleGroupRowProps {
    pub metrics: Option<Metrics>,
    pub group_props: Option<crate::components::button::ToggleGroupProps>,
    pub row_props: Option<ActionRowProps>,
    pub homogeneous: Option<bool>,
}
impl ToggleGroupRowProps {
    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
    pub fn group_props(mut self, props: crate::components::button::ToggleGroupProps) -> Self {
        self.group_props = Some(props);
        self
    }

    pub fn row_props(mut self, value: impl Into<ActionRowProps>) -> Self {
        self.row_props = Some(value.into());
        self
    }

    pub fn homogeneous(mut self, value: impl Into<bool>) -> Self {
        self.homogeneous = Some(value.into());
        self
    }

    pub fn apply<'a, T: Clone + PartialEq, M>(
        &self,
        mut row: ToggleGroupRow<'a, T, M>,
    ) -> ToggleGroupRow<'a, T, M> {
        if let Some(props) = &self.row_props {
            row.row = props.apply(row.row);
        }
        if let Some(v) = self.homogeneous {
            row = row.homogeneous(v);
        }
        if let Some(props) = &self.group_props {
            row.group = props.apply(row.group);
        }
        if let Some(metrics) = self.metrics {
            row = row.metrics(metrics);
        }
        row
    }
}

#[cfg(test)]
mod nested_props_tests {
    use super::*;
    use crate::components::{ButtonProps, TextProps};

    #[test]
    fn optional_combo_overrides_merge_without_changing_selection_or_disabled_policy() {
        let options = [1, 2];
        let row = ComboRow::new("Choice", &options, Some(2), |value| value)
            .disabled(true)
            .disabled_when(|value| *value == 1)
            .label_props(TextProps::default().width(137).height(29))
            .dropdown_props(
                super::super::dropdown::DropdownItemProps::default()
                    .label_props(TextProps::default().width(151).height(31))
                    .button_props(ButtonProps::default().clip(true)),
            );
        let row = ComboRowProps::default().apply(row);
        let row = ComboRowProps::default()
            .row_props(ActionRowProps::default().label_props(TextProps::default().height(37)))
            .label_props(TextProps::default())
            .dropdown_props(
                super::super::dropdown::DropdownItemProps::default()
                    .label_props(TextProps::default().height(41))
                    .button_props(ButtonProps::default().clip(false)),
            )
            .apply(row);
        assert_eq!(row.row.label_props.width, Some(137.into()));
        assert_eq!(row.row.label_props.height, Some(37.into()));
        assert_eq!(row.dropdown_props.label.width, Some(151.into()));
        assert_eq!(row.dropdown_props.label.height, Some(41.into()));
        assert_eq!(row.dropdown_props.button.clip, Some(false));
        assert_eq!(row.selected, Some(2));
        assert_eq!((row.on_select)(1), 1);
        assert!(row.insensitive);
        assert!(row.disabled.as_ref().unwrap()(&1));
        assert!(!row.disabled.as_ref().unwrap()(&2));
    }
    use iced::Size;
    use iced::advanced::{layout, renderer::Headless, widget::Tree};

    fn geometry(widget: impl Into<Element<'static, ()>>) -> (Size, usize) {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut widget = widget.into();
        let mut tree = Tree::new(&widget);
        let node = widget.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(600.0, 1000.0)),
        );
        fn count(tree: &Tree) -> usize {
            1 + tree.children.iter().map(count).sum::<usize>()
        }
        (node.size(), count(&tree))
    }

    #[test]
    fn default_nested_props_preserve_geometry_and_tree() {
        let make = || {
            ActionRow::new("日本語 🦀")
                .subtitle("secondary")
                .on_activate(())
        };
        assert_eq!(
            geometry(make()),
            geometry(ActionRowProps::default().apply(make()))
        );
        let configured = make().label_props(TextProps::default().height(83));
        assert_eq!(
            ActionRowProps::default()
                .apply(configured)
                .label_props
                .height,
            Some(83.into())
        );
    }

    #[test]
    fn label_and_button_overrides_reach_real_children_after_defaults() {
        let make = || ActionRow::new("Title").on_activate(());
        let baseline = geometry(make());
        let label = geometry(make().label_props(TextProps::default().height(120)));
        assert!(label.0.height >= 120.0);
        assert!(label.0.height > baseline.0.height);
        assert_eq!(label.1, baseline.1, "no replacement child or extra wrapper");
        let button = geometry(make().button_props(ButtonProps::default().width(237).height(199)));
        assert_eq!(button.0, Size::new(237.0, 199.0));
        assert_eq!(button.1, baseline.1);
    }

    #[test]
    fn reusable_row_props_preserve_data_and_actions() {
        let row = ActionRowProps::default()
            .spacing(9.0)
            .apply(ActionRow::new("kept").on_activate(7u8));
        assert_eq!(row.title, "kept");
        assert_eq!(row.on_press, Some(7));
        assert_eq!(row.spacing, Some(9.0));
        let options = ["a", "b"];
        let row = ComboRowProps::default()
            .value_spacing(17.0)
            .apply(ComboRow::new("choice", &options, Some("b"), |_| 8u8));
        assert_eq!(row.selected, Some("b"));
        assert_eq!((row.on_select)("a"), 8);
        assert_eq!(row.value_spacing, Some(17.0));
    }
}
