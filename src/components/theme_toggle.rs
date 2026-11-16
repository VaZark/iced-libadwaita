//! A controlled System → Light → Dark → System appearance button.
use crate::{AppearanceIcon, Element, Metrics};
use iced::{
    Length, Padding,
    widget::{button, container},
};

pub use crate::primitives::appearance::ThemeMode;

/// Presentation overrides for the concrete children of [`ThemeToggle`]. Defaults
/// preserve the toggle's existing layout, styles and controlled behavior.
#[derive(Clone, Default)]
pub struct ThemeToggleProps {
    pub button_props: super::props::ButtonProps,
    pub container_props: super::props::ContainerProps,
    pub tooltip_props: super::tooltip::TooltipProps,
}

impl ThemeToggleProps {
    pub fn button_props(mut self, props: super::props::ButtonProps) -> Self {
        self.button_props = props;
        self
    }
    pub fn container_props(mut self, props: super::props::ContainerProps) -> Self {
        self.container_props = props;
        self
    }
    pub fn tooltip_props(mut self, props: super::tooltip::TooltipProps) -> Self {
        self.tooltip_props = props;
        self
    }
    /// Merge explicitly supplied child fields, retaining all other presentation
    /// fields and the parent's mode, callbacks, disabled flag, padding and metrics.
    pub fn apply<'a, M>(&self, mut toggle: ThemeToggle<'a, M>) -> ThemeToggle<'a, M> {
        toggle.button_props.merge(&self.button_props);
        toggle.container_props.merge(&self.container_props);
        toggle.tooltip_props.merge(&self.tooltip_props);
        toggle
    }
}

impl ThemeMode {
    pub fn icon(self) -> AppearanceIcon {
        match self {
            Self::System => AppearanceIcon::System,
            Self::Light => AppearanceIcon::Light,
            Self::Dark => AppearanceIcon::Dark,
        }
    }
}

/// A single icon button displaying the current preference and cycling to the next.
/// Handle its message with `utils::set_scheme_override(mode.scheme_override())`.
pub struct ThemeToggle<'a, M> {
    mode: ThemeMode,
    on_toggle: Option<Box<dyn Fn(ThemeMode) -> M + 'a>>,
    disabled: bool,
    padding: Padding,
    metrics: Metrics,
    button_props: super::props::ButtonProps,
    container_props: super::props::ContainerProps,
    tooltip_props: super::tooltip::TooltipProps,
}
impl<'a, M> ThemeToggle<'a, M> {
    pub fn new(mode: ThemeMode) -> Self {
        Self {
            mode,
            on_toggle: None,
            disabled: false,
            padding: Padding::ZERO,
            metrics: Metrics::desktop(),
            button_props: Default::default(),
            container_props: Default::default(),
            tooltip_props: Default::default(),
        }
    }
    /// Presentation applied to the native button after its defaults. Does not
    /// replace the parent-owned action or enable a disabled toggle.
    pub fn button_props(mut self, props: super::props::ButtonProps) -> Self {
        self.button_props = props;
        self
    }
    /// Presentation of the existing icon-centering container inside the button.
    pub fn container_props(mut self, props: super::props::ContainerProps) -> Self {
        self.container_props = props;
        self
    }
    /// Presentation of the tooltip surface and label. Use
    /// [`TooltipProps::label_props`](super::tooltip::TooltipProps::label_props)
    /// for label styling; its text continues to describe the parent-owned mode.
    pub fn tooltip_props(mut self, props: super::tooltip::TooltipProps) -> Self {
        self.tooltip_props = props;
        self
    }
    pub fn on_toggle(mut self, on_toggle: impl Fn(ThemeMode) -> M + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }
    /// Padding around the icon inside the button (default none: the icon is centered in a
    /// square of `button_min_height`).
    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = p.into();
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }
    fn press(&self) -> Option<M> {
        if self.disabled {
            None
        } else {
            self.on_toggle.as_ref().map(|f| f(self.mode.next()))
        }
    }
}
fn appearance_icon_size(metrics: &Metrics, tokens: &crate::theme_config::Tokens) -> f32 {
    tokens
        .number("theme-toggle", "icon-size")
        .unwrap_or(metrics.icon_size)
}

impl<'a, M: Clone + 'a> From<ThemeToggle<'a, M>> for Element<'a, M> {
    fn from(toggle: ThemeToggle<'a, M>) -> Self {
        let content = toggle.container_props.apply(
            container(toggle.mode.icon().view(appearance_icon_size(
                &toggle.metrics,
                &crate::theme_config::current().tokens(),
            )))
            .center(Length::Fixed(toggle.metrics.button_min_height)),
        );
        let control = toggle.button_props.apply(
            button(content)
                .padding(toggle.padding)
                .style(crate::styles::button::flat)
                .on_press_maybe(toggle.press()),
        );
        super::tooltip::tooltip_with_props(
            control,
            format!(
                "Appearance: {} · Switch to {}",
                toggle.mode.label(),
                toggle.mode.next().label()
            ),
            crate::components::TooltipPosition::Bottom,
            toggle.tooltip_props,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icon_size_is_config_owned_or_inherits_metrics_without_arithmetic() {
        let mut metrics = Metrics::default();
        metrics.icon_size = 23.0;
        let config = |text: &str| {
            crate::theme_config::ThemeConfig::builder("appearance-icon-test")
                .files(false)
                .toolkit_text("")
                .app_text(text)
                .build()
        };
        let base = config("");
        assert_eq!(appearance_icon_size(&metrics, &base.tokens()), 23.0);
        let explicit = config(r##"{
    "theme-toggle": {
        "icon-size": 31,
    },
}
"##);
        assert!(explicit.warnings().is_empty());
        assert_eq!(appearance_icon_size(&metrics, &explicit.tokens()), 31.0);
        for invalid in ["0px", "-1px", "129px", "NaNpx", "24"] {
            let cfg = config(&format!("[theme-toggle]\nicon-size={invalid}\n"));
            assert!(!cfg.warnings().is_empty(), "{invalid}");
            assert_eq!(appearance_icon_size(&metrics, &cfg.tokens()), 23.0);
        }
        assert_eq!(
            appearance_icon_size(&metrics, &crate::theme_config::libadwaita().tokens()),
            18.0
        );
    }

    #[test]
    fn defaults_to_system_and_cycles_all_three_preferences() {
        assert_eq!(ThemeMode::default(), ThemeMode::System);
        for (current, next) in [
            (ThemeMode::System, ThemeMode::Light),
            (ThemeMode::Light, ThemeMode::Dark),
            (ThemeMode::Dark, ThemeMode::System),
        ] {
            let toggle = ThemeToggle::new(current).on_toggle(|mode| mode);
            assert_eq!(toggle.press(), Some(next));
            assert_eq!(toggle.disabled(true).press(), None);
            assert_eq!(ThemeToggle::<ThemeMode>::new(current).press(), None);
            assert_eq!(ThemeMode::from(current.scheme_override()), current);
        }
    }
    #[test]
    fn appearance_artwork_fits_and_renders_at_header_sizes() {
        for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
            let tree =
                resvg::usvg::Tree::from_str(mode.icon().source(), &Default::default()).unwrap();
            let bounds = tree.root().abs_stroke_bounding_box();
            assert!(
                bounds.left() >= 0.0
                    && bounds.top() >= 0.0
                    && bounds.right() <= 16.0
                    && bounds.bottom() <= 16.0
            );
            for size in [16, 18, 24, 32] {
                let mut pixels = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
                resvg::render(
                    &tree,
                    resvg::tiny_skia::Transform::from_scale(size as f32 / 16.0, size as f32 / 16.0),
                    &mut pixels.as_mut(),
                );
                assert!(pixels.data().chunks_exact(4).any(|p| p[3] > 0));
            }
        }
    }
}

#[cfg(test)]
mod child_api_tests {
    use super::*;
    use crate::Theme;
    use crate::components::{
        ButtonProps, ContainerProps, TextProps, TooltipPosition, TooltipProps,
    };
    use iced::advanced::{
        Renderer as _, Shell, clipboard, layout, renderer::Headless, widget::Tree,
    };
    use iced::{Color, Event, Point, Rectangle, Size, Vector, mouse};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn metrics() -> Metrics {
        let mut metrics = Metrics::default();
        metrics.button_min_height = 32.0;
        metrics.icon_size = 16.0;
        metrics
    }

    fn nodes(node: &layout::Node) -> Vec<Rectangle> {
        fn collect(layout: layout::Layout<'_>, out: &mut Vec<Rectangle>) {
            out.push(layout.bounds());
            for child in layout.children() {
                collect(child, out);
            }
        }
        let mut out = vec![];
        collect(layout::Layout::new(node), &mut out);
        out
    }

    #[test]
    fn default_native_and_aggregate_props_preserve_configured_layout_and_aliases() {
        let renderer = renderer();
        let metrics = metrics();
        let baseline: Element<'_, ThemeMode> = crate::components::tooltip(
            button(
                container(ThemeMode::System.icon().view(appearance_icon_size(
                    &metrics,
                    &crate::theme_config::current().tokens(),
                )))
                .center(Length::Fixed(metrics.button_min_height)),
            )
            .padding(3)
            .style(crate::styles::button::flat)
            .on_press(ThemeMode::Light),
            "Appearance: System · Switch to Light",
            TooltipPosition::Bottom,
        );
        let widget: crate::widgets::ThemeToggle<'_, ThemeMode> =
            crate::components::ThemeToggle::new(ThemeMode::System)
                .metrics(metrics)
                .padding(3)
                .on_toggle(|mode| mode)
                .button_props(ButtonProps::default())
                .container_props(ContainerProps::default())
                .tooltip_props(TooltipProps::default());
        let widget: crate::components::ThemeToggle<'_, ThemeMode> =
            ThemeToggleProps::default().apply(widget);
        let current: Element<'_, ThemeMode> = widget.into();
        let mut layouts = vec![];
        for mut element in [baseline, current] {
            let mut tree = Tree::new(element.as_widget());
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(400.0, 240.0)),
            );
            assert_eq!(node.size(), Size::new(38.0, 38.0));
            layouts.push(nodes(&node));
        }
        assert_eq!(layouts[0], layouts[1]);
    }

    #[test]
    fn aggregate_merges_partial_nested_fields_without_changing_parent_state() {
        let tooltip = TooltipProps::default()
            .gap(9.0)
            .padding(7)
            .style(|_| container::Style::default())
            .label_props(
                TextProps::default()
                    .width(137)
                    .size(19)
                    .style(|_| iced::widget::text::Style::default()),
            );
        let button = ButtonProps::default()
            .width(91)
            .padding(8)
            .clip(true)
            .style(|_, _| button::Style::default());
        let container = ContainerProps::default()
            .height(43)
            .padding(6)
            .clip(true)
            .style(|_| container::Style::default());
        let toggle = ThemeToggle::new(ThemeMode::Light)
            .on_toggle(|mode| mode)
            .disabled(true)
            .padding(3)
            .metrics(metrics())
            .button_props(button.clone())
            .container_props(container.clone())
            .tooltip_props(tooltip.clone());
        let toggle = ThemeToggleProps::default().apply(toggle);
        let toggle = ThemeToggleProps::default()
            .button_props(ButtonProps::default().clip(false).padding(0))
            .container_props(ContainerProps::default().clip(false).padding(0))
            .tooltip_props(
                TooltipProps::default()
                    .gap(0.0)
                    .label_props(TextProps::default().size(23)),
            )
            .apply(toggle);
        assert_eq!(toggle.button_props.width, button.width);
        assert_eq!(toggle.button_props.padding, Some(Padding::ZERO));
        assert_eq!(toggle.button_props.clip, Some(false));
        assert!(Arc::ptr_eq(
            toggle.button_props.style.as_ref().unwrap(),
            button.style.as_ref().unwrap()
        ));
        assert_eq!(toggle.container_props.height, container.height);
        assert_eq!(toggle.container_props.padding, Some(Padding::ZERO));
        assert_eq!(toggle.container_props.clip, Some(false));
        assert!(Arc::ptr_eq(
            toggle.container_props.style.as_ref().unwrap(),
            container.style.as_ref().unwrap()
        ));
        assert_eq!(toggle.tooltip_props.gap, Some(0.0));
        assert_eq!(toggle.tooltip_props.padding, tooltip.padding);
        assert_eq!(
            toggle.tooltip_props.label_props.width,
            tooltip.label_props.width
        );
        assert_eq!(
            toggle.tooltip_props.label_props.size,
            Some(iced::Pixels(23.0))
        );
        assert!(Arc::ptr_eq(
            toggle.tooltip_props.container_style.as_ref().unwrap(),
            tooltip.container_style.as_ref().unwrap()
        ));
        assert!(Arc::ptr_eq(
            toggle.tooltip_props.label_props.style.as_ref().unwrap(),
            tooltip.label_props.style.as_ref().unwrap()
        ));
        assert_eq!(toggle.mode, ThemeMode::Light);
        assert_eq!(toggle.padding, Padding::from(3));
        assert_eq!(toggle.metrics.button_min_height, 32.0);
        assert!(toggle.disabled);
        assert_eq!(toggle.press(), None);
        assert_eq!(toggle.disabled(false).press(), Some(ThemeMode::Dark));
    }

    #[test]
    fn resolved_icon_geometry_reaches_the_native_child() {
        let renderer = renderer();
        let mut metrics = metrics();
        metrics.icon_size = 23.0;
        let expected = appearance_icon_size(&metrics, &crate::theme_config::current().tokens());
        let mut element: Element<'_, ()> =
            ThemeToggle::new(ThemeMode::System).metrics(metrics).into();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 240.0)),
        );
        assert!(
            nodes(&node)
                .iter()
                .any(|bounds| bounds.size() == Size::new(expected, expected))
        );
    }

    #[test]
    fn container_geometry_and_button_padding_apply_without_extra_wrappers() {
        let renderer = renderer();
        let mut element: Element<'_, ()> = ThemeToggle::new(ThemeMode::System)
            .metrics(metrics())
            .container_props(ContainerProps::default().width(70).height(30))
            .button_props(ButtonProps::default().padding([4, 6]))
            .into();
        let mut tree = Tree::new(element.as_widget());
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 240.0)),
        );
        assert_eq!(node.size(), Size::new(82.0, 38.0));
        assert!(
            nodes(&node)
                .iter()
                .any(|r| r.size() == Size::new(70.0, 30.0))
        );
    }

    #[test]
    fn native_children_control_actual_geometry_without_owning_events_or_disabled() {
        let mut renderer = renderer();
        let viewport = Rectangle::with_size(Size::new(400.0, 240.0));
        for disabled in [false, true] {
            for with_callback in [false, true] {
                let draws = Arc::new(AtomicUsize::new(0));
                let button_draws = draws.clone();
                let container_draws = draws.clone();
                let props = ThemeToggleProps::default()
                    .button_props(
                        ButtonProps::default()
                            .width(100)
                            .height(54)
                            .padding([4, 6])
                            .clip(true)
                            .style(move |_, _| {
                                button_draws.fetch_add(1, Ordering::SeqCst);
                                button::Style::default()
                            }),
                    )
                    .container_props(
                        ContainerProps::default()
                            .width(70)
                            .height(30)
                            .clip(true)
                            .style(move |_| {
                                container_draws.fetch_add(1, Ordering::SeqCst);
                                container::Style::default()
                            }),
                    );
                let mut toggle = ThemeToggle::new(ThemeMode::Light)
                    .metrics(metrics())
                    .padding(2)
                    .disabled(disabled);
                if with_callback {
                    toggle = toggle.on_toggle(|mode| mode);
                }
                let mut element: Element<'_, ThemeMode> = props.apply(toggle).into();
                let mut tree = Tree::new(element.as_widget());
                let node = element.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, viewport.size()),
                );
                assert_eq!(node.size(), Size::new(100.0, 54.0));
                // A fixed native button imposes its padded minimum on the child.
                assert!(
                    nodes(&node)
                        .iter()
                        .any(|r| r.size() == Size::new(88.0, 46.0)),
                    "native layout: {:?}",
                    nodes(&node)
                );
                let layout = layout::Layout::new(&node);
                let cursor = mouse::Cursor::Available(Point::new(50.0, 27.0));
                assert_eq!(
                    element
                        .as_widget()
                        .mouse_interaction(&tree, layout, cursor, &viewport, &renderer)
                        == mouse::Interaction::Pointer,
                    with_callback && !disabled
                );
                let mut messages = vec![];
                // The parent remains controlled: repeated presses request the same next mode.
                for _ in 0..2 {
                    for event in [
                        mouse::Event::ButtonPressed(mouse::Button::Left),
                        mouse::Event::ButtonReleased(mouse::Button::Left),
                    ] {
                        element.as_widget_mut().update(
                            &mut tree,
                            &Event::Mouse(event),
                            layout,
                            cursor,
                            &renderer,
                            &mut clipboard::Null,
                            &mut Shell::new(&mut messages),
                            &viewport,
                        );
                    }
                }
                assert_eq!(
                    messages,
                    if with_callback && !disabled {
                        vec![ThemeMode::Dark; 2]
                    } else {
                        vec![]
                    }
                );
                renderer.reset(viewport);
                element.as_widget().draw(
                    &tree,
                    &mut renderer,
                    &Theme::light(),
                    &iced::advanced::renderer::Style {
                        text_color: Color::BLACK,
                    },
                    layout,
                    cursor,
                    &viewport,
                );
                assert_eq!(draws.load(Ordering::SeqCst), 2);
            }
        }
    }

    #[test]
    fn tooltip_surface_and_label_props_reach_the_actual_overlay() {
        let mut renderer = renderer();
        let viewport = Rectangle::with_size(Size::new(500.0, 300.0));
        let label_draws = Arc::new(AtomicUsize::new(0));
        let surface_draws = Arc::new(AtomicUsize::new(0));
        let label_seen = label_draws.clone();
        let surface_seen = surface_draws.clone();
        let tooltip = TooltipProps::default()
            .delay(std::time::Duration::ZERO)
            .gap(11.0)
            .padding(7)
            .label_props(
                TextProps::default()
                    .width(180)
                    .height(34)
                    .size(23)
                    .style(move |_| {
                        label_seen.fetch_add(1, Ordering::SeqCst);
                        iced::widget::text::Style::default()
                    }),
            )
            .style(move |_| {
                surface_seen.fetch_add(1, Ordering::SeqCst);
                container::Style::default()
            });
        let toggle = ThemeToggle::<()>::new(ThemeMode::Dark)
            .metrics(metrics())
            .tooltip_props(tooltip);
        let mut element: Element<'_, ()> = toggle.into();
        let mut tree = Tree::new(element.as_widget());
        let node = element
            .as_widget_mut()
            .layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport.size()),
            )
            .move_to(Point::new(200.0, 40.0));
        assert_eq!(node.size(), Size::new(32.0, 32.0));
        let layout = layout::Layout::new(&node);
        let cursor = mouse::Cursor::Available(Point::new(216.0, 56.0));
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(216.0, 56.0),
            }),
            layout,
            cursor,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut vec![]),
            &viewport,
        );
        let mut overlay = element
            .as_widget_mut()
            .overlay(&mut tree, layout, &renderer, &viewport, Vector::ZERO)
            .expect("appearance tooltip opens even when the parent has no action");
        let overlay_node = overlay.as_overlay_mut().layout(&renderer, viewport.size());
        let rects = nodes(&overlay_node);
        assert!(
            rects.iter().any(|r| r.size() == Size::new(180.0, 34.0)),
            "tooltip label: {rects:?}"
        );
        assert!(
            rects.iter().any(|r| r.size() == Size::new(194.0, 48.0)
                && (r.y - (layout.bounds().y + layout.bounds().height + 11.0)).abs() < 0.01),
            "tooltip gap and padding: {rects:?}"
        );
        renderer.reset(viewport);
        overlay.as_overlay().draw(
            &mut renderer,
            &Theme::light(),
            &iced::advanced::renderer::Style {
                text_color: Color::BLACK,
            },
            layout::Layout::new(&overlay_node),
            cursor,
        );
        assert_eq!(label_draws.load(Ordering::SeqCst), 1);
        assert_eq!(surface_draws.load(Ordering::SeqCst), 1);
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_icon_padding_is_a_parameter() {
        let t = ThemeToggle::<()>::new(ThemeMode::System).padding(4.0);
        assert_eq!(t.padding, Padding::from(4.0));
        let _: Element<'_, ()> = t.into();
    }
}
