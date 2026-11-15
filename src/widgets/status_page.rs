//! `AdwStatusPage`, `AdwClamp`, `AdwBin`, and `.card`.

#[cfg(test)]
mod surface_regression_tests {
    use super::*;
    use crate::layouts::boundary::{SurfaceShape, current_surface};
    use iced::advanced::{
        Renderer as _,
        layout::{self, Layout},
        renderer::{self, Headless},
        widget::{Tree, Widget},
    };
    use iced::{Color, Point, Rectangle, Size, mouse};
    use std::{
        cell::Cell,
        rc::Rc,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    struct Overflow(Rc<Cell<Option<SurfaceShape>>>);
    impl Widget<(), crate::Theme, iced::Renderer> for Overflow {
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fill, Length::Fill)
        }
        fn layout(
            &mut self,
            _: &mut Tree,
            _: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(limits.max())
        }
        fn draw(
            &self,
            _: &Tree,
            r: &mut iced::Renderer,
            _: &crate::Theme,
            _: &renderer::Style,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &Rectangle,
        ) {
            self.0.set(current_surface());
            r.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::with_size(Size::new(80.0, 52.0)),
                    snap: false,
                    ..Default::default()
                },
                Color::from_rgb(1.0, 0.0, 0.0),
            );
        }
    }
    fn pixel(image: &[u8], x: usize, y: usize) -> &[u8] {
        &image[(y * 80 + x) * 4..(y * 80 + x + 1) * 4]
    }
    #[test]
    fn card_final_style_is_resolved_once_shared_with_content_and_border_survives() {
        let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let bounds = Rectangle::new(Point::new(4.0, 4.0), Size::new(64.0, 40.0));
        let view = Rectangle::with_size(Size::new(80.0, 52.0));
        for radius in [
            iced::border::Radius::from(0.0),
            100.0.into(),
            iced::border::Radius {
                top_left: 22.0,
                top_right: 3.0,
                bottom_left: 0.0,
                bottom_right: 12.0,
            },
        ] {
            let calls = Arc::new(AtomicUsize::new(0));
            let count = calls.clone();
            let seen = Rc::new(Cell::new(None));
            let mut card: Element<'_, ()> = Card::new(Element::new(Overflow(seen.clone())))
                .padding(0)
                .container_props(ContainerProps::default().clip(false).style(move |_| {
                    count.fetch_add(1, Ordering::SeqCst);
                    container::Style {
                        background: Some(Color::from_rgb(0.0, 0.0, 1.0).into()),
                        border: iced::Border {
                            radius,
                            width: 2.0,
                            color: Color::from_rgb(0.0, 1.0, 0.0),
                        },
                        ..Default::default()
                    }
                }))
                .into();
            let mut tree = Tree::new(&card);
            let node = card
                .as_widget_mut()
                .layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(bounds.size(), bounds.size()),
                )
                .move_to(bounds.position());
            renderer.reset(view);
            card.as_widget().draw(
                &tree,
                &mut renderer,
                &crate::Theme::light().with_shadows(crate::Shadows::none()),
                &renderer::Style::default(),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &view,
            );
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(seen.get(), Some(SurfaceShape::new(bounds, radius)));
            let image = renderer.screenshot(Size::new(80, 52), 1.0, Color::BLACK);
            assert_eq!(pixel(&image, 35, 25), &[255, 0, 0, 255], "content survives");
            assert_eq!(
                pixel(&image, 35, 4),
                &[0, 255, 0, 255],
                "border is above content"
            );
            assert_eq!(
                pixel(&image, 1, 1),
                &[0, 0, 0, 255],
                "overflow clipped, not covered"
            );
            if radius.top_left > 0.0 {
                assert_eq!(
                    pixel(&image, 4, 4),
                    &[0, 0, 0, 255],
                    "rounded excluded corner"
                );
            }
        }
    }
}

use crate::Element;
use crate::components::{ContainerProps, TextProps};
use crate::styles::{container as sc, text as st};
use crate::{Metrics, Typography};
use iced::Length;
use iced::alignment::Horizontal;
use iced::widget::{column, container};

/// `AdwClamp`: limits the width of `content`, centering it.
pub fn clamp<'a, M: 'a>(content: impl Into<Element<'a, M>>, maximum: f32) -> Element<'a, M> {
    clamp_with_props(content, maximum, &ClampProps::default())
}

/// `AdwBin`: a single-child container.
pub fn bin<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    bin_with_props(content, &ContainerProps::default())
}

/// `.card`: padded card surface.
pub fn card<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Card::new(content).into()
}

/// `.card`, the generic chainable box: **one** surface (one border, one shadow, one
/// clip) holding any number of children, separated by hairlines. Other components are
/// this box with a different default: [`boxed_list`](crate::widgets::boxed_list) is a `Card` of
/// rows with no padding, a preferences group is a titled `Card` of rows.
///
/// ```ignore
/// Card::new(carousel).padding(0).push(ComboRow::new(..)) // one frame, one shadow
/// ```
///
/// Padding applies to each child; use `.padding(0)` for children that pad themselves
/// (rows, carousels). Content is clipped to the final `container_props` surface,
/// including custom asymmetric corners; its border and elevation stay outside
/// the content clip. `container_props.clip` cannot disable this surface boundary.
/// Child overlays are portals: they escape the Card and own their own surface.
pub struct Card<'a, M> {
    container_props: ContainerProps,
    child_props: ContainerProps,
    separator_props: ContainerProps,
    children: Vec<Element<'a, M>>,
    padding: iced::Padding,
    width: Length,
    height: Length,
    separators: bool,
}

impl<'a, M: 'a> Card<'a, M> {
    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn child_props(mut self, props: ContainerProps) -> Self {
        self.child_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn separator_props(mut self, props: ContainerProps) -> Self {
        self.separator_props = props;
        self
    }

    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            container_props: ContainerProps::default(),
            child_props: ContainerProps::default(),
            separator_props: ContainerProps::default(),

            children: vec![content.into()],
            padding: Metrics::desktop().card_padding,
            width: Length::Fill,
            height: Length::Shrink,
            separators: true,
        }
    }
    /// Appends a child inside the same surface (after a hairline).
    pub fn push(mut self, child: impl Into<Element<'a, M>>) -> Self {
        self.children.push(child.into());
        self
    }
    /// Whether hairlines separate the children (default true).
    pub fn separators(mut self, on: bool) -> Self {
        self.separators = on;
        self
    }
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = p.into();
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
}

impl<'a, M: 'a> From<Card<'a, M>> for Element<'a, M> {
    fn from(c: Card<'a, M>) -> Self {
        let mut col = column![].width(Length::Fill);
        for (i, child) in c.children.into_iter().enumerate() {
            if i > 0 && c.separators {
                col = col.push(
                    c.separator_props.apply(
                        container(iced::widget::space().height(1))
                            .width(Length::Fill)
                            .style(sc::card_separator),
                    ),
                );
            }
            col = col.push(
                c.child_props
                    .apply(container(child).padding(c.padding).width(Length::Fill)),
            );
        }
        // Layout props and the final style resolver have one owner. Native
        // container clipping is rectangular; Boundary owns the surface invariant
        // even if presentation props request clip(false).
        let style = c.container_props.style.clone();
        let surface = crate::layouts::boundary::Boundary::surface(
            c.container_props
                .apply(container(col).width(c.width).height(c.height)),
            move |theme| {
                style
                    .as_ref()
                    .map_or_else(|| sc::card_surface(theme), |f| f(theme))
            },
        )
        .underlay(draw_card_elevation);
        crate::components::framed(surface, "card")
    }
}

// Elevation uses the resolved surface, never a second lookup of the theme radius.
// All offsets/spread/blur come directly from the configured shadow layers.
fn draw_card_elevation(
    renderer: &mut iced::Renderer,
    theme: &crate::Theme,
    shape: crate::layouts::boundary::SurfaceShape,
) {
    use iced::advanced::Renderer as _;
    if !theme.shadows.enabled {
        return;
    }
    for layer in &theme.shadows.card.layers {
        let spread = layer.spread * theme.shadows.scale;
        let bounds = iced::Rectangle {
            x: shape.bounds.x - spread,
            y: shape.bounds.y - spread,
            width: (shape.bounds.width + 2.0 * spread).max(0.0),
            height: (shape.bounds.height + 2.0 * spread).max(0.0),
        };
        let radius = iced::border::Radius {
            top_left: (shape.radius.top_left + spread).max(0.0),
            top_right: (shape.radius.top_right + spread).max(0.0),
            bottom_left: (shape.radius.bottom_left + spread).max(0.0),
            bottom_right: (shape.radius.bottom_right + spread).max(0.0),
        };
        let outer = crate::layouts::boundary::SurfaceShape::new(bounds, radius);
        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds: outer.bounds,
                border: iced::Border::default().rounded(outer.radius),
                shadow: iced::Shadow {
                    color: iced::Color {
                        a: (layer.color.a * theme.shadows.intensity).clamp(0.0, 1.0),
                        ..layer.color
                    },
                    offset: iced::Vector::new(
                        layer.x * theme.shadows.scale,
                        layer.y * theme.shadows.scale,
                    ),
                    blur_radius: layer.blur * theme.shadows.scale,
                },
                snap: false,
            },
            iced::Color::TRANSPARENT,
        );
    }
}

/// A scrolled, clamped page: the body of a window pane. Content is limited to
/// `maximum` wide (default 720) with the standard page padding, in an Adwaita
/// scrolled window. (`AdwPreferencesPage` is the same thing specialised for groups.)
pub struct Page<'a, M> {
    content_props: Option<ContainerProps>,
    content: Element<'a, M>,
    maximum: f32,
    metrics: Metrics,
    padding: Option<iced::Padding>,
}

pub fn page<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Page<'a, M> {
    Page::new(content)
}

impl<'a, M: 'a> Page<'a, M> {
    pub fn content_props(mut self, props: ContainerProps) -> Self {
        self.content_props = Some(props);
        self
    }

    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content_props: None,
            content: content.into(),
            maximum: 720.0,
            metrics: Metrics::desktop(),
            padding: None,
        }
    }
    /// The padding around the content (default: the theme's page padding).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// `AdwClamp:maximum-size`.
    pub fn maximum_size(mut self, px: f32) -> Self {
        self.maximum = px;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<Page<'a, M>> for Element<'a, M> {
    fn from(p: Page<'a, M>) -> Self {
        let content = match p.content_props {
            Some(props) => props.apply(container(p.content)).into(),
            None => p.content,
        };
        crate::layouts::scrolled_window(
            crate::layouts::Container::new(content)
                .size(crate::layouts::ContainerSize::Px(p.maximum))
                .padding(p.padding.unwrap_or(p.metrics.page_padding))
                .metrics(p.metrics),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

/// `AdwStatusPage`.
pub struct StatusPage<'a, M> {
    title_props: TextProps,
    description_props: TextProps,
    body_props: ContainerProps,
    container_props: ContainerProps,
    icon: Option<Element<'a, M>>,
    title: Option<String>,
    description: Option<String>,
    child: Option<Element<'a, M>>,
    compact: bool,
    metrics: Metrics,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    maximum: f32,
}

pub fn status_page<'a, M>() -> StatusPage<'a, M> {
    StatusPage::new()
}

impl<'a, M> StatusPage<'a, M> {
    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn description_props(mut self, props: TextProps) -> Self {
        self.description_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn body_props(mut self, props: ContainerProps) -> Self {
        self.body_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }

    pub fn new() -> Self {
        Self {
            title_props: TextProps::default(),
            description_props: TextProps::default(),
            body_props: ContainerProps::default(),
            container_props: ContainerProps::default(),

            icon: None,
            title: None,
            description: None,
            child: None,
            compact: false,
            metrics: Metrics::desktop(),
            spacing: None,
            padding: None,
            maximum: 400.0,
        }
    }
    /// `icon-name` (any element, typically an [`Icon`](crate::Icon) view).
    pub fn icon(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.icon = Some(e.into());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, t: impl Into<String>) -> Self {
        self.description = Some(t.into());
        self
    }
    pub fn child(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.child = Some(e.into());
        self
    }
    /// `.compact`: smaller title, tighter spacing.
    pub fn compact(mut self, c: bool) -> Self {
        self.compact = c;
        self
    }
    /// The gap between the icon, the title, the description and the child (default: the theme's
    /// `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The padding around the content (default: the theme's `space_xl`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The widest the content gets, in px (default 400).
    pub fn maximum_size(mut self, px: f32) -> Self {
        self.maximum = px;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> Default for StatusPage<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> From<StatusPage<'a, M>> for Element<'a, M> {
    fn from(s: StatusPage<'a, M>) -> Self {
        let m = s.metrics;
        let t = Typography::desktop();
        let mut col = column![]
            .spacing(s.spacing.unwrap_or(m.space_m))
            .align_x(Horizontal::Center)
            .width(Length::Fill);
        if let Some(i) = s.icon {
            col = col.push(i);
        }
        if let Some(title) = s.title {
            col = col.push(s.title_props.apply(if s.compact {
                t.title_2(title)
            } else {
                t.title_1(title)
            }));
        }
        if let Some(d) = s.description {
            col = col.push(
                s.description_props
                    .apply(t.body(d).style(st::dim).align_x(Horizontal::Center)),
            );
        }
        if let Some(c) = s.child {
            col = col.push(c);
        }
        s.container_props
            .apply(
                container(clamp(
                    s.body_props.apply(
                        container(col)
                            .padding(s.padding.unwrap_or(iced::Padding::from(m.space_xl))),
                    ),
                    s.maximum,
                ))
                .center(Length::Fill),
            )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gaps_padding_and_width_are_parameters() {
        let s = StatusPage::<()>::new()
            .title("Empty")
            .description("Nothing here")
            .spacing(3.0)
            .padding([4.0, 5.0])
            .maximum_size(300.0);
        assert_eq!((s.spacing, s.maximum), (Some(3.0), 300.0));
        let _: Element<'_, ()> = s.into();
        let _: Element<'_, ()> = Page::<()>::new(iced::widget::space()).padding(2.0).into();
    }
}

/// Reusable, no-op-by-default presentation for [`Card`].
#[derive(Clone, Default)]
pub struct CardProps {
    pub container_props: Option<ContainerProps>,
    pub child_props: Option<ContainerProps>,
    pub separator_props: Option<ContainerProps>,
}
impl CardProps {
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }
    pub fn child_props(mut self, props: ContainerProps) -> Self {
        self.child_props = Some(props);
        self
    }
    pub fn separator_props(mut self, props: ContainerProps) -> Self {
        self.separator_props = Some(props);
        self
    }
    pub fn apply<'a, M: 'a>(&self, widget: Card<'a, M>) -> Card<'a, M> {
        let mut widget = widget;
        if let Some(props) = &self.container_props {
            widget.container_props.merge(props);
        }
        if let Some(props) = &self.child_props {
            widget.child_props.merge(props);
        }
        if let Some(props) = &self.separator_props {
            widget.separator_props.merge(props);
        }
        widget
    }
}

/// Reusable, no-op-by-default presentation for [`StatusPage`].
#[derive(Clone, Default)]
pub struct StatusPageProps {
    pub title_props: Option<TextProps>,
    pub description_props: Option<TextProps>,
    pub body_props: Option<ContainerProps>,
    pub container_props: Option<ContainerProps>,
}
impl StatusPageProps {
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = Some(props);
        self
    }
    pub fn description_props(mut self, props: TextProps) -> Self {
        self.description_props = Some(props);
        self
    }
    pub fn body_props(mut self, props: ContainerProps) -> Self {
        self.body_props = Some(props);
        self
    }
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }
    pub fn apply<'a, M>(&self, widget: StatusPage<'a, M>) -> StatusPage<'a, M> {
        let mut widget = widget;
        if let Some(props) = &self.title_props {
            widget.title_props.merge(props);
        }
        if let Some(props) = &self.description_props {
            widget.description_props.merge(props);
        }
        if let Some(props) = &self.body_props {
            widget.body_props.merge(props);
        }
        if let Some(props) = &self.container_props {
            widget.container_props.merge(props);
        }
        widget
    }
}

/// Presentation for the two concrete containers in [`clamp`].
#[derive(Clone, Default)]
pub struct ClampProps {
    pub content_props: ContainerProps,
    pub container_props: ContainerProps,
}
impl ClampProps {
    pub fn content_props(mut self, props: ContainerProps) -> Self {
        self.content_props = props;
        self
    }
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }
}
pub fn clamp_with_props<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    maximum: f32,
    props: &ClampProps,
) -> Element<'a, M> {
    props
        .container_props
        .apply(
            container(
                props
                    .content_props
                    .apply(container(content).max_width(maximum).width(Length::Fill)),
            )
            .center_x(Length::Fill),
        )
        .into()
}
pub fn bin_with_props<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    props: &ContainerProps,
) -> Element<'a, M> {
    props.apply(container(content)).into()
}
pub fn card_with_props<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    props: &CardProps,
) -> Element<'a, M> {
    props.apply(Card::new(content)).into()
}

#[derive(Clone, Default)]
pub struct PageProps {
    pub content_props: Option<ContainerProps>,
}
impl PageProps {
    pub fn content_props(mut self, props: ContainerProps) -> Self {
        self.content_props = Some(props);
        self
    }
    pub fn apply<'a, M: 'a>(&self, widget: Page<'a, M>) -> Page<'a, M> {
        let mut widget = widget;
        crate::components::props::merge_optional!(widget, self; content_props);
        widget
    }
}

#[cfg(test)]
mod typed_presentation_tests {
    #[test]
    fn container_geometry_and_function_adapters_apply() {
        let props = ContainerProps::default().width(123).height(45);
        let status: Element<'_, u8> = StatusPageProps::default()
            .container_props(props.clone())
            .apply(StatusPage::new().title("Empty"))
            .into();
        let bin: Element<'_, u8> = bin_with_props(iced::widget::space(), &props);
        let clamp: Element<'_, u8> = clamp_with_props(
            iced::widget::space(),
            400.0,
            &ClampProps::default().container_props(props),
        );
        for element in [status, bin, clamp] {
            assert_eq!(
                element.as_widget().size(),
                iced::Size::new(Length::Fixed(123.0), Length::Fixed(45.0))
            );
        }
    }
    #[test]
    fn owned_style_callback_receives_the_current_theme() {
        let props =
            StatusPageProps::default().container_props(ContainerProps::default().style(|theme| {
                iced::widget::container::Style {
                    text_color: Some(theme.semantic.window.fg),
                    ..Default::default()
                }
            }));
        let copy = props.clone();
        assert!(std::sync::Arc::ptr_eq(
            props
                .container_props
                .as_ref()
                .unwrap()
                .style
                .as_ref()
                .unwrap(),
            copy.container_props
                .as_ref()
                .unwrap()
                .style
                .as_ref()
                .unwrap()
        ));
        let callback = copy
            .container_props
            .as_ref()
            .unwrap()
            .style
            .as_ref()
            .unwrap();
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            assert_eq!(callback(&theme).text_color, Some(theme.semantic.window.fg));
        }
        let _: Element<'_, u8> = props.apply(StatusPage::new().title("Empty")).into();
    }

    use super::*;
    use iced::advanced::widget::Tree;
    fn same_tree(a: &Tree, b: &Tree) {
        assert!(a.tag == b.tag);
        assert_eq!(a.children.len(), b.children.len());
        for (a, b) in a.children.iter().zip(&b.children) {
            same_tree(a, b);
        }
    }
    #[test]
    fn reusable_defaults_preserve_geometry_and_tree() {
        let build = || StatusPage::<u8>::new().title("Title").description("Body");
        let w = build();
        let baseline: Element<'_, u8> = w.into();
        let w = StatusPageProps::default().apply(build());
        let configured: Element<'_, u8> = w.into();
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}

crate::components::props::impl_merge!(CardProps; fields: []; optional: [container_props, child_props, separator_props]; nested: []);
