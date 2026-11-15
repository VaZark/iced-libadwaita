//! `AdwToolbarView` and `GtkActionBar`.

use crate::Element;
use crate::components::ContainerProps;
use crate::styles::container as sc;
use crate::{Metrics, Theme};
use iced::alignment::Vertical;
use iced::gradient::Linear;
use iced::widget::{column, container, row, space, stack};
use iced::{Background, Color, Length, Radians};

/// `AdwToolbarView:top-bar-style` / `bottom-bar-style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarStyle {
    /// No shadow (the Adwaita default; libadwaita draws a scroll undershoot instead).
    #[default]
    Flat,
    /// Persistent shadow: `0 1px` and `0 2px 4px` of `--headerbar-shade-color` at 50%.
    Raised,
    /// A 1px line in `--headerbar-darker-shade-color`.
    RaisedBorder,
}

fn toolbar_metric_for(
    tokens: &crate::theme_config::Tokens,
    name: &str,
    key: &str,
    inherited: f32,
) -> f32 {
    tokens
        .variant_number("toolbar", name, key)
        .unwrap_or(inherited)
        .max(0.0)
}

fn shadow_opacity(configured: Option<f32>) -> f32 {
    configured.unwrap_or(0.5).clamp(0.0, 1.0)
}

/// The shadow cast by a raised bar onto the content, drawn as a fading strip
/// (iced draws one shadow per quad and later siblings would paint over it).
fn shade_strip<'a, M: 'a>(
    top: bool,
    name: &str,
    tokens: &crate::theme_config::Tokens,
) -> Element<'a, M> {
    let height = toolbar_metric_for(tokens, name, "shadow-height", Metrics::desktop().space_s);
    let opacity = shadow_opacity(tokens.variant_number("toolbar", name, "shadow-opacity"));
    container(space())
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(move |t: &Theme| {
            let s = &t.semantic;
            let k = if t.shadows.enabled {
                t.shadows.intensity
            } else {
                0.0
            };
            let c = Color {
                a: (s.headerbar_shade.a * opacity * k).min(1.0),
                ..s.headerbar_shade
            };
            let clear = Color { a: 0.0, ..c };
            // iced angles: 0 = left→right; π/2 = top→bottom.
            let (from, to) = if top { (c, clear) } else { (clear, c) };
            let g = Linear::new(Radians(std::f32::consts::FRAC_PI_2))
                .add_stop(0.0, from)
                .add_stop(1.0, to);
            iced::widget::container::Style {
                background: Some(Background::Gradient(g.into())),
                ..Default::default()
            }
        })
        .into()
}

fn border_line<'a, M: 'a>(name: &str, tokens: &crate::theme_config::Tokens) -> Element<'a, M> {
    let width = toolbar_metric_for(
        tokens,
        name,
        "separator-width",
        Metrics::desktop().border_width,
    );
    container(space().height(width))
        .width(Length::Fill)
        .style(|t: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(t.semantic.headerbar_darker_shade)),
            ..Default::default()
        })
        .into()
}

/// `AdwToolbarView`: top bars, content, bottom bars.
pub struct ToolbarView<'a, M> {
    top: Vec<Element<'a, M>>,
    bottom: Vec<Element<'a, M>>,
    content: Element<'a, M>,
    top_style: Option<BarStyle>,
    bottom_style: Option<BarStyle>,
    named_variant: Option<String>,
    tokens: Option<crate::theme_config::Tokens>,
    spacing: f32,
    content_props: ContainerProps,
}

pub fn toolbar_view<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> ToolbarView<'a, M> {
    ToolbarView::new(content)
}

impl<'a, M: 'a> ToolbarView<'a, M> {
    /// Owned presentation overrides for the concrete child.
    pub fn content_props(mut self, props: ContainerProps) -> Self {
        self.content_props = props;
        self
    }
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content_props: ContainerProps::default(),
            top: Vec::new(),
            bottom: Vec::new(),
            content: content.into(),
            top_style: None,
            bottom_style: None,
            named_variant: None,
            tokens: None,
            spacing: 0.0,
        }
    }
    /// Uses local tokens for breakpoint rules and decoration geometry instead of global tokens.
    pub fn tokens(mut self, tokens: crate::theme_config::Tokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Uses this config's tokens locally without installing it globally.
    pub fn theme_config(self, cfg: &crate::theme_config::ThemeConfig) -> Self {
        self.tokens(cfg.tokens())
    }

    /// Select a logical variant for both sides; explicit per-side styles take priority.
    pub fn named_variant(mut self, name: impl Into<String>) -> Self {
        self.named_variant = Some(name.into());
        self
    }

    /// `add_top_bar`.
    pub fn top_bar(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.top.push(e.into());
        self
    }
    /// `add_bottom_bar`.
    pub fn bottom_bar(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.bottom.push(e.into());
        self
    }
    /// `top-bar-style`.
    pub fn top_bar_style(mut self, s: BarStyle) -> Self {
        self.top_style = Some(s);
        self
    }
    /// The gap between the bars and the content (default: 0, bars sit flush).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px;
        self
    }
    /// `bottom-bar-style`.
    pub fn bottom_bar_style(mut self, s: BarStyle) -> Self {
        self.bottom_style = Some(s);
        self
    }
}

impl<'a, M: 'a> From<ToolbarView<'a, M>> for Element<'a, M> {
    fn from(t: ToolbarView<'a, M>) -> Self {
        let mut col = column![].spacing(t.spacing);
        let has_top = !t.top.is_empty();
        let has_bottom = !t.bottom.is_empty();
        if has_top {
            col = col.push(column![
                column(t.top).spacing(t.spacing),
                toolbar_decoration(
                    t.top_style,
                    t.named_variant.clone(),
                    true,
                    true,
                    t.spacing,
                    t.tokens.clone()
                ),
            ]);
        }

        let mut layers = vec![
            // The content between the bars: popovers opened inside stay inside it.
            t.content_props
                .apply(
                    container(crate::layouts::content_area(t.content))
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .into(),
        ];
        if has_top {
            layers.push(toolbar_decoration(
                t.top_style,
                t.named_variant.clone(),
                true,
                false,
                0.0,
                t.tokens.clone(),
            ));
        }
        if has_bottom {
            layers.push(toolbar_decoration(
                t.bottom_style,
                t.named_variant.clone(),
                false,
                false,
                0.0,
                t.tokens.clone(),
            ));
        }
        col = col.push(stack(layers).width(Length::Fill).height(Length::Fill));

        if has_bottom {
            col = col.push(column![
                toolbar_decoration(
                    t.bottom_style,
                    t.named_variant.clone(),
                    false,
                    true,
                    t.spacing,
                    t.tokens.clone(),
                ),
                column(t.bottom).spacing(t.spacing),
            ]);
        }
        col.width(Length::Fill).height(Length::Fill).into()
    }
}

fn bar_name(style: BarStyle) -> &'static str {
    match style {
        BarStyle::Flat => "flat",
        BarStyle::Raised => "raised",
        BarStyle::RaisedBorder => "raised-border",
    }
}

fn bar_presentation(name: &str) -> BarStyle {
    match name {
        "raised" => BarStyle::Raised,
        "raised-border" => BarStyle::RaisedBorder,
        _ => BarStyle::Flat,
    }
}

fn selected_toolbar(
    tokens: &crate::theme_config::Tokens,
    width: f32,
    style: Option<BarStyle>,
    named: Option<&str>,
) -> String {
    style
        .map(bar_name)
        .or(named)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            crate::breakpoints::variant_for_tokens(
                tokens,
                "toolbar",
                &crate::breakpoints::Breakpoints::current(),
                width,
                "flat",
            )
        })
}

// Only decorations rebuild at a breakpoint: bars, popovers and content retain their widgets.
fn toolbar_decoration<'a, M: 'a>(
    style: Option<BarStyle>,
    named: Option<String>,
    top: bool,
    separator: bool,
    spacing: f32,
    tokens: Option<crate::theme_config::Tokens>,
) -> Element<'a, M> {
    crate::layouts::responsive(move |size| {
        let tokens = tokens
            .clone()
            .unwrap_or_else(|| crate::theme_config::current().tokens());
        let name = selected_toolbar(&tokens, size.width, style, named.as_deref());
        match (bar_presentation(&name), separator) {
            (BarStyle::RaisedBorder, true) => container(border_line(&name, &tokens))
                .padding(if top {
                    iced::Padding::ZERO.top(spacing)
                } else {
                    iced::Padding::ZERO.bottom(spacing)
                })
                .into(),
            (BarStyle::Raised, false) => {
                let strip = container(shade_strip(top, &name, &tokens));
                if top {
                    strip.align_top(Length::Fill).into()
                } else {
                    strip.align_bottom(Length::Fill).into()
                }
            }
            _ => space().height(0).into(),
        }
    })
    .height(if separator {
        Length::Shrink
    } else {
        Length::Fill
    })
    .into()
}

/// `GtkActionBar`.
pub struct ActionBar<'a, M> {
    start: Vec<Element<'a, M>>,
    center: Option<Element<'a, M>>,
    end: Vec<Element<'a, M>>,
    metrics: Metrics,
    spacing: Option<f32>,
    section_spacing: Option<f32>,
    padding: Option<iced::Padding>,
    section_props: ContainerProps,
    container_props: ContainerProps,
}

pub fn action_bar<'a, M>() -> ActionBar<'a, M> {
    ActionBar::new()
}

impl<'a, M> ActionBar<'a, M> {
    pub fn section_props(mut self, props: ContainerProps) -> Self {
        self.section_props = props;
        self
    }
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }
    pub fn new() -> Self {
        Self {
            section_props: ContainerProps::default(),
            container_props: ContainerProps::default(),
            start: Vec::new(),
            center: None,
            end: Vec::new(),
            metrics: Metrics::desktop(),
            spacing: None,
            section_spacing: None,
            padding: None,
        }
    }
    pub fn pack_start(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.start.push(e.into());
        self
    }
    pub fn pack_end(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.end.insert(0, e.into());
        self
    }
    pub fn center(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.center = Some(e.into());
        self
    }
    /// The gap between the items of the start and end sides (default: the theme's `space_s`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The gap between the start, center and end sections (default: the theme's `space_m`).
    pub fn section_spacing(mut self, px: f32) -> Self {
        self.section_spacing = Some(px);
        self
    }
    /// Padding around the bar (default: the theme's `space_s`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
}

impl<'a, M: 'a> Default for ActionBar<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> From<ActionBar<'a, M>> for Element<'a, M> {
    fn from(a: ActionBar<'a, M>) -> Self {
        let m = a.metrics;
        let side = |v: Vec<Element<'a, M>>, h| {
            a.section_props.apply(
                container(
                    row(v)
                        .spacing(a.spacing.unwrap_or(m.space_s))
                        .align_y(Vertical::Center),
                )
                .width(Length::FillPortion(1))
                .align_x(h),
            )
        };
        a.container_props
            .apply(
                container(
                    row![
                        side(a.start, iced::alignment::Horizontal::Left),
                        a.center.unwrap_or_else(|| space().into()),
                        side(a.end, iced::alignment::Horizontal::Right),
                    ]
                    .align_y(Vertical::Center)
                    .spacing(a.section_spacing.unwrap_or(m.space_m)),
                )
                .padding(a.padding.unwrap_or(iced::Padding::from(m.space_s)))
                .width(Length::Fill)
                .style(sc::headerbar),
            )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(text: &str) -> crate::theme_config::Tokens {
        let cfg = crate::theme_config::ThemeConfig::builder("toolbar-variant-tests")
            .files(false)
            .app_text(text)
            .build();
        assert!(cfg.warnings().is_empty(), "{:?}", cfg.warnings());
        cfg.tokens()
    }

    #[test]
    fn rules_select_toolbar_modes_but_explicit_side_styles_win() {
        let tokens = tokens("[toolbar]\nbreakpoint-rules=xs:flat,md:raised,xl:raised-border\n");
        for (width, expected) in [
            (0.0, "flat"),
            (800.0, "raised"),
            (100_000.0, "raised-border"),
        ] {
            assert_eq!(selected_toolbar(&tokens, width, None, None), expected);
            assert_eq!(
                selected_toolbar(&tokens, width, None, Some("raised")),
                "raised"
            );
            assert_eq!(
                selected_toolbar(&tokens, width, Some(BarStyle::Flat), Some("raised")),
                "flat"
            );
        }
        assert_eq!(
            selected_toolbar(&Default::default(), 800.0, None, None),
            "flat"
        );
        assert_eq!(bar_presentation("unknown"), BarStyle::Flat);
        for style in [BarStyle::Flat, BarStyle::Raised, BarStyle::RaisedBorder] {
            assert_eq!(bar_presentation(bar_name(style)), style);
        }
        let t = ToolbarView::<()>::new(space())
            .top_bar_style(BarStyle::Flat)
            .named_variant("raised");
        assert_eq!(t.top_style, Some(BarStyle::Flat));
        assert_eq!(t.bottom_style, None);
    }

    #[test]
    fn toolbar_variant_geometry_falls_back_to_base_then_metrics() {
        let tokens = tokens(
            "[toolbar]\nshadow-height=3px\n[toolbar.raised]\nshadow-height=7px\nshadow-opacity=0.2\nseparator-width=0px\n",
        );
        assert_eq!(
            toolbar_metric_for(&tokens, "raised", "shadow-height", 9.0),
            7.0
        );
        assert_eq!(
            toolbar_metric_for(&tokens, "flat", "shadow-height", 9.0),
            3.0
        );
        assert_eq!(
            toolbar_metric_for(&tokens, "raised", "separator-width", 9.0),
            0.0
        );
        assert_eq!(
            toolbar_metric_for(&Default::default(), "raised", "shadow-height", 9.0),
            9.0
        );
        assert_eq!(
            shadow_opacity(tokens.variant_number("toolbar", "raised", "shadow-opacity")),
            0.2
        );
    }

    #[test]
    fn toolbar_layout_keeps_fill_height_and_no_phantom_separator_gap() {
        use iced::advanced::renderer::Headless;
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for style in [BarStyle::Flat, BarStyle::Raised, BarStyle::RaisedBorder] {
            let mut element: Element<'_, ()> = ToolbarView::new(space())
                .top_bar(space().height(20))
                .top_bar_style(style)
                .spacing(4.0)
                .into();
            let mut tree = iced::advanced::widget::Tree::new(element.as_widget());
            for width in [400.0, 1000.0] {
                let limits = iced::advanced::layout::Limits::new(
                    iced::Size::ZERO,
                    iced::Size::new(width, 200.0),
                );
                let node = element
                    .as_widget_mut()
                    .layout(&mut tree, &renderer, &limits);
                assert_eq!(node.size(), iced::Size::new(width, 200.0));
                let expected = if style == BarStyle::RaisedBorder {
                    24.0 + toolbar_metric_for(
                        &crate::theme_config::current().tokens(),
                        "raised-border",
                        "separator-width",
                        Metrics::desktop().border_width,
                    )
                } else {
                    20.0
                };
                assert_eq!(node.children()[0].size().height, expected);
            }
        }
    }

    #[test]
    fn local_toolbar_rules_and_geometry_reach_layout_and_explicit_styles_win() {
        use iced::advanced::renderer::Headless;
        let cfg = crate::theme_config::ThemeConfig::builder("local-toolbar")
            .files(false)
            .app_text("[toolbar]\nbreakpoint-rules=xs:flat,md:raised-border\nseparator-width=5px\nshadow-height=9px\n[toolbar.raised-border]\nseparator-width=8px\n")
            .build();
        assert!(cfg.warnings().is_empty(), "{:?}", cfg.warnings());
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for use_config in [false, true] {
            for (named, explicit, expected_small, expected_large) in [
                (None, None, 20.0, 32.0),
                (Some("raised-border"), None, 32.0, 32.0),
                (Some("raised-border"), Some(BarStyle::Flat), 20.0, 20.0),
                (Some("flat"), Some(BarStyle::RaisedBorder), 32.0, 32.0),
            ] {
                let t = ToolbarView::<()>::new(space())
                    .top_bar(space().height(20))
                    .spacing(4.0);
                let mut t = if use_config {
                    t.theme_config(&cfg)
                } else {
                    t.tokens(cfg.tokens())
                };
                if let Some(name) = named {
                    t = t.named_variant(name);
                }
                if let Some(style) = explicit {
                    t = t.top_bar_style(style);
                }
                let mut element: Element<'_, ()> = t.into();
                let mut tree = iced::advanced::widget::Tree::new(element.as_widget());
                for (width, expected) in [(400.0, expected_small), (1000.0, expected_large)] {
                    let limits = iced::advanced::layout::Limits::new(
                        iced::Size::ZERO,
                        iced::Size::new(width, 200.0),
                    );
                    let node = element
                        .as_widget_mut()
                        .layout(&mut tree, &renderer, &limits);
                    assert_eq!(node.children()[0].size().height, expected);
                    assert_eq!(node.size().height, 200.0);
                }
            }
        }
        let tokens = cfg.tokens();
        let mut line: Element<'_, ()> = border_line("flat", &tokens);
        let mut tree = iced::advanced::widget::Tree::new(line.as_widget());
        let limits =
            iced::advanced::layout::Limits::new(iced::Size::ZERO, iced::Size::new(400.0, 200.0));
        assert_eq!(
            line.as_widget_mut()
                .layout(&mut tree, &renderer, &limits)
                .size()
                .height,
            5.0
        );
        let mut shadow: Element<'_, ()> = shade_strip(true, "raised", &tokens);
        let mut tree = iced::advanced::widget::Tree::new(shadow.as_widget());
        assert_eq!(
            shadow
                .as_widget_mut()
                .layout(&mut tree, &renderer, &limits)
                .size()
                .height,
            9.0
        );
        assert!(ToolbarView::<()>::new(space()).tokens.is_none());
    }

    #[test]
    fn shadow_opacity_is_optional_and_bounded() {
        assert_eq!(shadow_opacity(None), 0.5);
        assert_eq!(shadow_opacity(Some(0.2)), 0.2);
        assert_eq!(shadow_opacity(Some(-1.0)), 0.0);
        assert_eq!(shadow_opacity(Some(2.0)), 1.0);
    }

    #[test]
    fn raised_bars_build_with_inherited_geometry() {
        for style in [BarStyle::Raised, BarStyle::RaisedBorder] {
            let _: Element<'_, ()> = ToolbarView::new(space())
                .top_bar(space())
                .bottom_bar(space())
                .top_bar_style(style)
                .bottom_bar_style(style)
                .into();
        }
    }

    #[test]
    fn the_bars_take_their_own_spacing_and_padding() {
        let t = ToolbarView::<()>::new(space())
            .top_bar(space())
            .spacing(4.0);
        assert_eq!(t.spacing, 4.0);
        let _: Element<'_, ()> = t.into();
        let a = ActionBar::<()>::new()
            .pack_start(space())
            .spacing(1.0)
            .section_spacing(2.0)
            .padding([3.0, 4.0]);
        assert_eq!((a.spacing, a.section_spacing), (Some(1.0), Some(2.0)));
        let _: Element<'_, ()> = a.into();
    }
}

/// Owned presentation overrides for a nested `ToolbarView`; values and events stay parent-owned.
#[derive(Clone, Default)]
pub struct ToolbarProps {
    pub content_props: Option<ContainerProps>,
    pub spacing: Option<f32>,
}
impl ToolbarProps {
    pub fn content_props(mut self, value: impl Into<ContainerProps>) -> Self {
        self.content_props = Some(value.into());
        self
    }
    pub fn spacing(mut self, value: impl Into<f32>) -> Self {
        self.spacing = Some(value.into());
        self
    }
    pub fn apply<'a, M: 'a>(&self, mut widget: ToolbarView<'a, M>) -> ToolbarView<'a, M> {
        if let Some(value) = self.content_props.clone() {
            widget = widget.content_props(value);
        }
        if let Some(value) = self.spacing.clone() {
            widget = widget.spacing(value);
        }
        widget
    }
}
