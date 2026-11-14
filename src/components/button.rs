//! Buttons, `AdwButtonContent`, `AdwSplitButton`, `AdwToggleGroup`.

use crate::Element;
use crate::styles::button as sb;
use crate::{Icon, Metrics, Theme};
use iced::alignment::Vertical;
use iced::widget::{Button, button, container, row, space};
use iced::{Length, Padding};

/// `AdwButtonContent`: an optional icon next to a label.
pub struct ButtonContent<'a, M> {
    label: String,
    icon: Option<Icon>,
    spacing: Option<f32>,
    icon_end: bool,
    metrics: Metrics,
    _p: std::marker::PhantomData<&'a M>,
}

impl<'a, M> ButtonContent<'a, M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            spacing: None,
            icon_end: false,
            metrics: Metrics::desktop(),
            _p: std::marker::PhantomData,
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// The gap between the icon and the label (default `space_s`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// Puts the icon after the label instead of before it (default before).
    pub fn icon_end(mut self, end: bool) -> Self {
        self.icon_end = end;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<ButtonContent<'a, M>> for Element<'a, M> {
    fn from(c: ButtonContent<'a, M>) -> Self {
        let mut r = row![]
            .spacing(c.spacing.unwrap_or(c.metrics.space_s))
            .align_y(Vertical::Center);
        let label = crate::components::typography::label(c.label);
        match (c.icon, c.icon_end) {
            (Some(i), false) => r = r.push(i.view(c.metrics.icon_size)).push(label),
            (Some(i), true) => r = r.push(label).push(i.view(c.metrics.icon_size)),
            (None, _) => r = r.push(label),
        }
        r.into()
    }
}

/// Width of the [`min_height`] strut. iced's flex layout drops a child of width exactly zero
/// (it would not count towards the row's height), so the strut is one thousandth of a pixel.
const STRUT_WIDTH: f32 = 0.001;

/// `content` in a row at least `min` tall, vertically centered.
pub(crate) fn min_height<'a, M: 'a, R: iced::advanced::Renderer + 'a>(
    content: impl Into<iced::Element<'a, M, Theme, R>>,
    min: f32,
) -> iced::widget::Row<'a, M, Theme, R> {
    row![
        space()
            .width(Length::Fixed(STRUT_WIDTH))
            .height(Length::Fixed(min)),
        content.into(),
    ]
    .align_y(Vertical::Center)
}

pub(crate) fn sized<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    m: &Metrics,
) -> Button<'a, M, Theme> {
    sized_variant(content, m, "")
}

/// [`sized`] for the variant `name`: when the theme gives that variant a bevel
/// (`[button.<name>] border-style=raised`), the content sits in a [`Bevel`] that fills the
/// button and takes the padding; otherwise the button is built exactly as before.
pub(crate) fn sized_variant<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    m: &Metrics,
    name: &str,
) -> Button<'a, M, Theme> {
    if !name.is_empty() {
        let section = format!("button.{name}");
        if let Some((style, width)) = crate::components::bevel_of(&section) {
            let inner = min_height(content, m.button_min_height - m.button_padding.y());
            return button(
                crate::components::Bevel::new(inner, section, style, width)
                    .padding(m.button_padding)
                    .pressable(true),
            )
            .padding(Padding::ZERO);
        }
    }
    // A minimum height, not a fixed one: a strut of the minimum inner height beside the
    // content, which is centered against it. A one-line label gets the standard height; taller
    // content (an icon over a label) makes the button grow instead of spilling out of it, since
    // iced does not clip. To cap the height, opt in with the button's own
    // `.height(Length::Fixed(..))`.
    button(min_height(
        content,
        m.button_min_height - m.button_padding.y(),
    ))
    .padding(m.button_padding)
}

/// Standard button (`.default`).
pub fn raised_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized_variant(content, &Metrics::desktop(), "default").style(sb::variant("default"))
}

/// `.flat` button.
pub fn flat_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized_variant(content, &Metrics::desktop(), "flat").style(sb::variant("flat"))
}

/// `.suggested-action` button.
pub fn suggested_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized_variant(content, &Metrics::desktop(), "suggested").style(sb::variant("suggested"))
}

/// `.destructive-action` button.
pub fn destructive_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized_variant(content, &Metrics::desktop(), "destructive").style(sb::variant("destructive"))
}

/// `.pill` button.
pub fn pill_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    let m = Metrics::desktop();
    button(content.into())
        .padding(m.pill_padding)
        .style(sb::variant("pill"))
}

/// `.pill.suggested-action` button.
pub fn suggested_pill_button<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
) -> Button<'a, M, Theme> {
    let m = Metrics::desktop();
    button(content.into())
        .padding(m.pill_padding)
        .style(sb::variant("suggested-pill"))
}

/// `GtkProgressBar` with Adwaita's 6px thickness.
pub fn progress_bar<'a>(
    range: std::ops::RangeInclusive<f32>,
    value: f32,
) -> iced::widget::ProgressBar<'a, Theme> {
    iced::widget::progress_bar(range, value).girth(Metrics::desktop().progress_height)
}

/// A button of any look: the **variant** `name` (`[button.<name>]` in a theme file, or declared by
/// the app), around `content`. A name the theme does not declare draws the neutral button.
pub fn variant_button<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    name: impl Into<String>,
) -> Button<'a, M, Theme> {
    let name = name.into();
    sized_variant(content, &Metrics::desktop(), &name).style(sb::variant(&name))
}

/// Selects a button's **variant**: `button(label).variant("brand")`. See
/// [`styles::button::variant`](crate::styles::button::variant).
pub trait ButtonVariant: Sized {
    /// Styles the button as the variant `name`.
    fn variant(self, name: impl Into<String>) -> Self;
}

impl<'a, M: 'a> ButtonVariant for Button<'a, M, Theme> {
    fn variant(self, name: impl Into<String>) -> Self {
        self.style(sb::variant(&name.into()))
    }
}

/// A button that is a circle: a square of [`size`](CircularButton::size) px whose content is
/// centered and clipped to that square, and whose corner radius is **half the size whatever the
/// variant or the theme says**. A border radius on a button of any shape cannot promise a circle
/// (a wider button gives a pill, and an oversized radius an ellipse), so this is a widget of its
/// own that fixes width and height to the same value.
///
/// The diameter is the theme's `[metrics] circular-button-size`, or the button height when unset.
/// Its [`variant`](CircularButton::variant) is `flat` unless set: any variant (`suggested`,
/// `default`, an app's own) styles its colors; its shape stays a circle.
pub struct CircularButton<'a, M> {
    content: Element<'a, M>,
    size: f32,
    variant: String,
    on_press: Option<M>,
}

impl<'a, M: 'a> CircularButton<'a, M> {
    /// A circular button around any content.
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            size: crate::theme_config::circular_button_size()
                .unwrap_or_else(|| Metrics::desktop().button_min_height),
            variant: "flat".into(),
            on_press: None,
        }
    }
    /// A circular button around an icon at the theme's icon size.
    pub fn icon(icon: Icon) -> Self {
        Self::new(icon.view(Metrics::desktop().icon_size))
    }
    /// The diameter in px.
    pub fn size(mut self, px: f32) -> Self {
        self.size = px.max(1.0);
        self
    }
    /// The variant that styles its colors (default `flat`).
    pub fn variant(mut self, name: impl Into<String>) -> Self {
        self.variant = name.into();
        self
    }
    pub fn on_press(mut self, msg: M) -> Self {
        self.on_press = Some(msg);
        self
    }
    /// Pressable only when there is a message; disabled otherwise.
    pub fn on_press_maybe(mut self, msg: Option<M>) -> Self {
        self.on_press = msg;
        self
    }
    /// The diameter in px.
    pub fn diameter(&self) -> f32 {
        self.size
    }
    /// The corner radius in px: half the diameter.
    pub fn radius(&self) -> f32 {
        self.size / 2.0
    }
}

impl<'a, M: Clone + 'a> From<CircularButton<'a, M>> for Element<'a, M> {
    fn from(b: CircularButton<'a, M>) -> Self {
        circular_shell(b.content, b.size)
            .style(sb::circular_variant(&b.variant, b.size))
            .on_press_maybe(b.on_press)
            .into()
    }
}

/// A button exactly `size` by `size` with its content centered and clipped to the square.
fn circular_shell<'a, M: 'a>(content: impl Into<Element<'a, M>>, size: f32) -> Button<'a, M, Theme> {
    button(container(content).center(Length::Fixed(size)).clip(true))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .padding(Padding::from(0.0))
}

/// A square `.circular` icon button (flat by default, like header bar buttons). A
/// [`CircularButton`] is the same with a variant and a size.
pub fn circular_button<'a, M: 'a>(icon: Icon) -> Button<'a, M, Theme> {
    let size = crate::theme_config::circular_button_size()
        .unwrap_or_else(|| Metrics::desktop().button_min_height);
    circular_shell(icon.view(Metrics::desktop().icon_size), size)
        .style(sb::circular_variant("flat", size))
}

/// A round `.suggested-action` button around any content: the play button of a player, a
/// primary action that is only an icon. [`circular_button`] is the flat one.
///
/// The diameter is the metrics' button height by default; [`size`](SuggestedCircularButton::size)
/// sets it (a player's play button is bigger than a toolbar's). It is a [`CircularButton`] with
/// the `suggested` variant.
pub struct SuggestedCircularButton<'a, M> {
    inner: CircularButton<'a, M>,
}

/// See [`SuggestedCircularButton`].
pub fn suggested_circular_button<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
) -> SuggestedCircularButton<'a, M> {
    SuggestedCircularButton {
        inner: CircularButton::new(content).variant("suggested"),
    }
}

impl<'a, M: 'a> SuggestedCircularButton<'a, M> {
    /// The diameter in px.
    pub fn size(mut self, px: f32) -> Self {
        self.inner = self.inner.size(px);
        self
    }
    pub fn on_press(mut self, msg: M) -> Self {
        self.inner = self.inner.on_press(msg);
        self
    }
    /// Pressable only when there is a message; disabled otherwise.
    pub fn on_press_maybe(mut self, msg: Option<M>) -> Self {
        self.inner = self.inner.on_press_maybe(msg);
        self
    }
}

impl<'a, M: Clone + 'a> From<SuggestedCircularButton<'a, M>> for Element<'a, M> {
    fn from(b: SuggestedCircularButton<'a, M>) -> Self {
        b.inner.into()
    }
}

#[cfg(test)]
mod circular_tests {
    use super::*;

    #[test]
    fn the_diameter_is_settable_and_a_message_makes_it_pressable() {
        let b: SuggestedCircularButton<'_, u8> = suggested_circular_button(iced::widget::space())
            .size(44.0)
            .on_press(1);
        assert_eq!((b.inner.size, b.inner.on_press), (44.0, Some(1)));
        let d: SuggestedCircularButton<'_, u8> =
            suggested_circular_button(iced::widget::space()).on_press_maybe(None);
        assert!(d.inner.on_press.is_none());
    }
}

/// A flat icon button with a rounded-rect hover (`.flat` + icon-only).
pub fn icon_button<'a, M: 'a>(icon: Icon) -> Button<'a, M, Theme> {
    let m = Metrics::desktop();
    button(container(icon.view(m.icon_size)).center(Length::Fixed(m.button_min_height)))
        .padding(Padding::from(0.0))
        .style(sb::flat)
}

/// One entry of a [`ToggleGroup`]: a label, an icon, both, or any element.
pub struct ToggleItem<'a, T, M> {
    pub value: T,
    pub label: Option<String>,
    pub icon: Option<Icon>,
    custom: Option<Element<'a, M>>,
    disabled: bool,
}

impl<'a, T, M> ToggleItem<'a, T, M> {
    /// A text segment.
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: Some(label.into()),
            icon: None,
            custom: None,
            disabled: false,
        }
    }
    /// An icon-only segment.
    pub fn icon_only(value: T, icon: Icon) -> Self {
        Self {
            value,
            label: None,
            icon: Some(icon),
            custom: None,
            disabled: false,
        }
    }
    /// A segment with any content (an avatar, a badge, a row of widgets…).
    pub fn custom(value: T, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            value,
            label: None,
            icon: None,
            custom: Some(content.into()),
            disabled: false,
        }
    }
    /// A segment that cannot be chosen (the choice is not available now): dimmed, and it does
    /// not react to the pointer.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Adds an icon before the label. When the group is too narrow for icon and
    /// label together, the segment falls back to the icon alone.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// `AdwToggleGroup` / `AdwViewSwitcher`: exclusive, linked segments.
///
/// It respects the space it is given: with [`width`](ToggleGroup::width)`(Fill)`
/// (or when a stacked row asks for 100%) the segments stretch evenly with no
/// dead space; when too wide it shrinks fairly and never overshoots its limits.
pub struct ToggleGroup<'a, T, M> {
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: Box<dyn Fn(T) -> M + 'a>,
    width: Length,
    homogeneous: bool,
    compact: Option<bool>,
    metrics: Metrics,
}

/// Builds a [`ToggleGroup`].
pub fn toggle_group<'a, T: Clone + PartialEq, M>(
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> ToggleGroup<'a, T, M> {
    ToggleGroup {
        items,
        active,
        on_select: Box::new(on_select),
        width: Length::Shrink,
        homogeneous: false,
        compact: None,
        metrics: Metrics::desktop(),
    }
}

impl<'a, T, M> ToggleGroup<'a, T, M> {
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// All segments the same width (`AdwToggleGroup:homogeneous`).
    pub fn homogeneous(mut self, h: bool) -> Self {
        self.homogeneous = h;
        self
    }
    /// Follows the window's mode: in mobile mode (`breakpoints::is_mobile(width)`, the `[windows]
    /// mobile-max-width` breakpoint) every segment that has an icon *and* a label shows the icon
    /// only; above it, icon and text. A segment with no icon has nothing shorter to show and
    /// falls back to its text. Same rule as [`MenuPresentation::resolve`](crate::widgets::MenuPresentation).
    pub fn window_width(self, width: f32) -> Self {
        self.compact(Some(crate::breakpoints::is_mobile(width)))
    }
    /// Icons or full content for the segments that have both, all together. Prefer
    /// [`window_width`](Self::window_width); this takes the decision directly. Without either
    /// (`None`) the group falls back to the space: icons as soon as any segment would be squeezed.
    pub fn compact(mut self, compact: Option<bool>) -> Self {
        self.compact = compact;
        self
    }
    /// `Length::Fill` makes the group fill the available width, each segment
    /// taking an equal share (`flex: 1`) with its content centered.
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, M: 'a> From<ToggleGroup<'a, T, M>> for Element<'a, M> {
    fn from(g: ToggleGroup<'a, T, M>) -> Self {
        let m = g.metrics;
        let active = g
            .active
            .as_ref()
            .and_then(|a| g.items.iter().position(|i| &i.value == a));
        let disabled: Vec<bool> = g.items.iter().map(|i| i.disabled).collect();
        let segments = g
            .items
            .into_iter()
            .map(|item| {
                let icon = |ic: Icon| -> Element<'a, M> { ic.view(m.icon_size).into() };
                let (full, compact): (Element<'a, M>, Option<Element<'a, M>>) =
                    match (item.custom, item.icon, item.label) {
                        (Some(c), _, _) => (c, None),
                        (None, Some(ic), Some(l)) => (
                            row![icon(ic), crate::components::typography::label(l)]
                                .spacing(m.space_s)
                                .align_y(Vertical::Center)
                                .into(),
                            Some(icon(ic)),
                        ),
                        (None, Some(ic), None) => (icon(ic), None),
                        (None, None, Some(l)) => {
                            (crate::components::typography::label(l).into(), None)
                        }
                        (None, None, None) => (iced::widget::space().into(), None),
                    };
                (item.value, full, compact)
            })
            .collect();
        crate::widgets::segmented::Segmented::new(segments, active, g.on_select)
            .width(g.width)
            .homogeneous(g.homogeneous)
            .compact(g.compact)
            .disabled(disabled)
            .metrics(m)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::layout::Limits;
    use iced::advanced::widget::Tree;
    use iced::{Element, Size};

    fn size_of(content: Size) -> Size {
        let boxed: Element<'_, (), Theme, ()> = min_height(
            Element::<'_, (), Theme, ()>::from(
                space()
                    .width(Length::Fixed(content.width))
                    .height(Length::Fixed(content.height)),
            ),
            24.0,
        )
        .into();
        let mut el = boxed;
        let mut tree = Tree::new(&el);
        el.as_widget_mut()
            .layout(
                &mut tree,
                &(),
                &Limits::new(Size::ZERO, Size::new(500.0, 500.0)),
            )
            .size()
    }

    #[test]
    fn content_gets_a_minimum_height_and_may_exceed_it() {
        // A one-line label (shorter than the minimum) is lifted to it...
        assert_eq!(size_of(Size::new(40.0, 19.0)).height, 24.0);
        // ...and taller content grows the box instead of overflowing it.
        assert_eq!(size_of(Size::new(40.0, 32.0)).height, 32.0);
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn button_content_gap_and_icon_side_are_parameters() {
        let c = ButtonContent::<()>::new("Go").icon(Icon::Check).spacing(2.0).icon_end(true);
        assert_eq!((c.spacing, c.icon_end), (Some(2.0), true));
        let _: Element<'_, ()> = c.into();
    }
}

#[cfg(test)]
mod circular_button_tests {
    use super::*;
    use iced::widget::button::Status;

    #[test]
    fn a_circular_button_is_a_square_with_half_its_size_as_radius_whatever_the_theme_says() {
        let b = CircularButton::<()>::new(iced::widget::space()).size(40.0);
        assert_eq!((b.diameter(), b.radius()), (40.0, 20.0));
        // A theme with a small radius and a variant that sets another one cannot change it.
        let cfg = crate::theme_config::ThemeConfig::builder("t")
            .files(false)
            .toolkit_text("[metrics]\nradius-m=2\nradius-pill=2\n[button.suggested]\nradius=5px\n")
            .app_text("")
            .build();
        let theme = Theme::light().with_theme_config(&cfg);
        for name in ["flat", "suggested", "default", "unknown"] {
            let style = sb::circular_variant(name, 40.0)(&theme, Status::Active);
            assert_eq!(style.border.radius, 20.0.into(), "{name}");
        }
        let _: Element<'_, ()> = b.variant("suggested").on_press(()).into();
    }

    #[test]
    fn the_diameter_follows_the_theme_unless_set() {
        let b = CircularButton::<()>::new(iced::widget::space());
        assert_eq!(b.diameter(), Metrics::desktop().button_min_height);
        assert_eq!(CircularButton::<()>::new(iced::widget::space()).size(0.0).diameter(), 1.0);
    }

    #[test]
    fn the_variant_extension_styles_a_plain_button() {
        let b: Button<'_, (), Theme> = flat_button(iced::widget::space()).variant("suggested");
        let _: Element<'_, ()> = b.into();
        let _: Element<'_, ()> = variant_button(iced::widget::space(), "flat").into();
    }
}
