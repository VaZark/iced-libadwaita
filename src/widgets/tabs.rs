//! `Tabs`: a row of tabs and the panel of the active one, after Mantine's
//! [Tabs](https://mantine.dev/core/tabs/) and Radix UI's. The behavior (which tab a click
//! activates, disabled tabs) is the [`tabs`](crate::primitives::tabs) primitive, shared with the
//! segmented control behind [`toggle_group`](crate::components::toggle_group).
//!
//! A tab can be [`closable`](TabItem::closable): the close button is drawn in the style of the
//! variant and sends the tab's key to [`on_close`](Tabs::on_close) (the app removes it).
//!
//! Controlled: the app owns the active key and takes it from `on_change`. Each tab may carry
//! the panel that shows while it is active (built only then), or none, for a list that drives
//! something else (the gallery's Preview / Code tabs).
//!
//! ```ignore
//! Tabs::new(Some(self.tab), Msg::Tab)
//!     .variant(TabsVariant::Floating)
//!     .tab(TabItem::new(Tab::Gallery, "Gallery").icon(Icon::Home).panel(|| gallery()))
//!     .tab(TabItem::new(Tab::Messages, "Messages").panel(|| messages()))
//!     .tab(TabItem::new(Tab::Settings, "Settings").disabled(true))
//! ```
//!
//! Variants, as in Mantine: [`Default`](TabsVariant::Default) (an accent line under the active
//! tab, over a hairline), [`Outline`](TabsVariant::Outline) (the active tab boxed),
//! [`Pills`](TabsVariant::Pills) (the active tab filled) and
//! [`Floating`](TabsVariant::Floating) (a highlight that slides behind the active tab, a
//! [`FloatingIndicator`](crate::widgets::FloatingIndicator)). Arrow-key navigation is not included: iced
//! has no focus model to hang it on (the primitive has `step` for it).

use crate::Element;
use crate::components::typography::label;
use crate::primitives::tabs::TabList;
use crate::styles::button as sb;
use crate::styles::container as sc;
use crate::styles::text as st;
use crate::widgets::FloatingIndicator;
use crate::{Icon, Metrics, Theme};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space};

/// How the tabs look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabsVariant {
    /// An accent line under the active tab, over a hairline.
    #[default]
    Default,
    /// The active tab in a box that opens into its panel.
    Outline,
    /// The active tab filled with the accent color.
    Pills,
    /// A highlight that slides behind the active tab.
    Floating,
}

type Panel<'a, M> = Box<dyn Fn() -> Element<'a, M> + 'a>;

/// One tab: its key, label, optional icon and panel.
pub struct TabItem<'a, K, M> {
    key: K,
    label: String,
    icon: Option<Icon>,
    disabled: bool,
    closable: bool,
    panel: Option<Panel<'a, M>>,
}

impl<'a, K, M> TabItem<'a, K, M> {
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            icon: None,
            disabled: false,
            closable: false,
            panel: None,
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// A disabled tab is dimmed and cannot be activated.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// A close button on this tab, in the style of the tabs' variant. Its press sends the tab's
    /// key to [`Tabs::on_close`]; the app removes the tab.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
    /// What shows while this tab is active; built only then.
    pub fn panel(mut self, build: impl Fn() -> Element<'a, M> + 'a) -> Self {
        self.panel = Some(Box::new(build));
        self
    }
}

/// See the [module](self).
pub struct Tabs<'a, K, M> {
    items: Vec<TabItem<'a, K, M>>,
    active: Option<K>,
    on_change: Box<dyn Fn(K) -> M + 'a>,
    on_close: Option<Box<dyn Fn(K) -> M + 'a>>,
    variant: TabsVariant,
    named_variant: Option<String>,
    explicit_variant: bool,
    grow: bool,
    metrics: Metrics,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    icon_spacing: Option<f32>,
    panel_spacing: Option<f32>,
    tokens: Option<crate::theme_config::Tokens>,
}

fn tab_metric_for(
    tokens: &crate::theme_config::Tokens,
    name: &str,
    key: &str,
    inherited: f32,
) -> f32 {
    resolve_tab_metric(tokens.variant_number("tabs", name, key), inherited)
}

fn resolve_tab_metric(configured: Option<f32>, inherited: f32) -> f32 {
    configured.unwrap_or(inherited).max(0.0)
}

impl<'a, K: Clone + PartialEq + 'a, M: Clone + 'a> Tabs<'a, K, M> {
    pub fn new(active: Option<K>, on_change: impl Fn(K) -> M + 'a) -> Self {
        Self {
            items: Vec::new(),
            active,
            on_change: Box::new(on_change),
            on_close: None,
            variant: TabsVariant::default(),
            named_variant: None,
            explicit_variant: false,
            grow: false,
            metrics: Metrics::desktop(),
            spacing: None,
            padding: None,
            icon_spacing: None,
            panel_spacing: None,
            tokens: None,
        }
    }
    /// Uses a local token snapshot instead of the installed global theme config.
    /// Inherited metrics come from `.metrics(...)`, not from this snapshot.
    pub fn tokens(mut self, tokens: crate::theme_config::Tokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Uses this config's tokens locally; pass its resolved metrics with `.metrics(...)`.
    pub fn theme_config(self, cfg: &crate::theme_config::ThemeConfig) -> Self {
        self.tokens(cfg.tokens())
    }

    fn resolved_tokens(&self) -> crate::theme_config::Tokens {
        self.tokens
            .clone()
            .unwrap_or_else(|| crate::theme_config::current().tokens())
    }

    fn tab_metric(&self, name: &str, key: &str, inherited: f32) -> f32 {
        tab_metric_for(&self.resolved_tokens(), name, key, inherited)
    }

    fn selected_variant(&self, width: f32) -> String {
        let explicit = self
            .named_variant
            .as_deref()
            .or_else(|| self.explicit_variant.then(|| tab_name(self.variant)));
        selected_tab(&self.resolved_tokens(), width, explicit)
    }

    fn bevel_of(&self, section: &str) -> Option<(crate::primitives::bevel::BevelStyle, f32)> {
        match &self.tokens {
            Some(tokens) => Some((
                crate::primitives::bevel::BevelStyle::parse(tokens.text(section, "border-style")?)?,
                tokens
                    .number(section, "border-width")
                    .unwrap_or(self.metrics.border_width),
            )),
            None => crate::components::bevel_of(section),
        }
    }

    pub fn tab(mut self, item: TabItem<'a, K, M>) -> Self {
        self.items.push(item);
        self
    }
    /// A closable tab's close button was pressed.
    pub fn on_close(mut self, f: impl Fn(K) -> M + 'a) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }
    /// Select a presentation explicitly, overriding theme breakpoint rules and any named variant.
    pub fn variant(mut self, v: TabsVariant) -> Self {
        self.variant = v;
        self.explicit_variant = true;
        self.named_variant = None;
        self
    }
    /// Select a logical variant explicitly, overriding breakpoint rules.
    pub fn named_variant(mut self, name: impl Into<String>) -> Self {
        self.named_variant = Some(name.into());
        self.explicit_variant = false;
        self
    }

    /// The tabs share the width equally (not for `Floating`, which hugs its tabs).
    pub fn grow(mut self, grow: bool) -> Self {
        self.grow = grow;
        self
    }
    /// The gap between the tabs (default: the theme's `space_xs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The padding inside each tab's button (default: the theme's `space_xs` by `space_m`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The gap between a tab's icon and its label (default: the theme's `space_s`).
    pub fn icon_spacing(mut self, px: f32) -> Self {
        self.icon_spacing = Some(px);
        self
    }
    /// The gap between the tab list and the active panel (default: the theme's `space_m`).
    pub fn panel_spacing(mut self, px: f32) -> Self {
        self.panel_spacing = Some(px);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn list(&self) -> TabList {
        TabList::with_disabled(
            self.items.len(),
            self.items.iter().map(|t| t.disabled).collect(),
        )
    }
    fn active_index(&self) -> Option<usize> {
        let key = self.active.as_ref()?;
        self.items.iter().position(|t| t.key == *key)
    }
    /// The close button of tab `i`, if it is closable and the app handles closing.
    fn close_button(&self, i: usize, name: &str) -> Option<Element<'a, M>> {
        let item = &self.items[i];
        let f = self
            .on_close
            .as_ref()
            .filter(|_| item.closable && !item.disabled)?;
        Some(
            button(Icon::WindowClose.view(self.tab_metric(
                name,
                "close-icon-size",
                self.metrics.icon_size,
            )))
            .padding(self.metrics.space_xs)
            .style(sb::circular_flat)
            .on_press(f(item.key.clone()))
            .into(),
        )
    }
    /// The message a press on tab `i` sends: none when it is disabled.
    fn press(&self, i: usize) -> Option<M> {
        self.list()
            .activate(i)
            .map(|i| (self.on_change)(self.items[i].key.clone()))
    }

    /// One tab's button, in the look of the variant.
    fn tab_button(
        &self,
        i: usize,
        active: bool,
        name: &str,
        variant: TabsVariant,
    ) -> Element<'a, M> {
        let m = &self.metrics;
        let item = &self.items[i];
        let text = |s: String| {
            let t = label(s);
            if active && !item.disabled {
                t
            } else {
                t.style(st::dim)
            }
        };
        let mut content = row![]
            .spacing(self.icon_spacing.unwrap_or(m.space_s))
            .align_y(Vertical::Center);
        if let Some(icon) = item.icon {
            content = content.push(icon.view(m.icon_size));
        }
        content = content.push(text(item.label.clone()));
        let b = button(content)
            .padding(
                self.padding
                    .unwrap_or(iced::Padding::from([m.space_xs, m.space_m])),
            )
            .on_press_maybe(self.press(i));
        let title: Element<'a, M> = match variant {
            TabsVariant::Pills if active => b.style(sb::suggested_pill),
            _ => b.style(sb::flat),
        }
        .into();
        match self.close_button(i, name) {
            Some(close) => row![title, close].align_y(Vertical::Center).into(),
            None => title,
        }
    }

    fn tab_list(&self, name: &str, variant: TabsVariant) -> Element<'a, M> {
        let m = &self.metrics;
        let active = self.active_index();
        if variant == TabsVariant::Floating {
            let tabs = (0..self.items.len())
                .map(|i| self.tab_button(i, active == Some(i), name, variant))
                .collect();
            return FloatingIndicator::new(tabs, active.unwrap_or(0))
                .spacing(self.spacing.unwrap_or(m.space_xs))
                .into();
        }
        let width = if self.grow {
            Length::Fill
        } else {
            Length::Shrink
        };
        let mut tabs = row![].spacing(self.spacing.unwrap_or(m.space_xs));
        for i in 0..self.items.len() {
            let on = active == Some(i);
            let button = self.tab_button(i, on, name, variant);
            let tab: Element<'a, M> = match variant {
                TabsVariant::Default => {
                    let line = container(space().height(self.tab_metric(
                        name,
                        "indicator-height",
                        m.focus_ring_width,
                    )))
                    .width(Length::Fill)
                    .style(move |t: &Theme| iced::widget::container::Style {
                        background: on.then_some(iced::Background::Color(t.semantic.accent.bg)),
                        ..Default::default()
                    });
                    column![button, line].into()
                }
                TabsVariant::Outline => {
                    let border_width = self.tab_metric(name, "border-width", m.border_width);
                    container(button)
                        .style(move |t: &Theme| iced::widget::container::Style {
                            border: if on {
                                iced::Border {
                                    color: t.semantic.border,
                                    width: border_width,
                                    radius: iced::border::top(t.metrics.radius_m),
                                }
                            } else {
                                iced::Border::default()
                            },
                            ..Default::default()
                        })
                        .into()
                }
                _ => button,
            };
            tabs = tabs.push(container(tab).width(width));
        }
        let tabs = tabs.width(if self.grow {
            Length::Fill
        } else {
            Length::Shrink
        });
        match variant {
            TabsVariant::Default | TabsVariant::Outline => column![
                tabs,
                container(space().height(self.tab_metric(name, "separator-width", m.border_width)))
                    .width(Length::Fill)
                    .style(sc::card_separator)
            ]
            .into(),
            _ => tabs.into(),
        }
    }
}

impl<'a, K: Clone + PartialEq + 'a, M: Clone + 'a> Tabs<'a, K, M> {
    /// The classic tab control, drawn when the theme gives `[frame.tab]` a bevel: tabs raised on
    /// their top, left and right (`border-edges=no-bottom`) over a raised page. The tabs touch,
    /// those that are not selected sit `[tabs] raise` px lower than the selected one, and the
    /// selected tab is that much taller at the top and runs that far over the page's frame, so
    /// it opens into the page. The page is `[frame.panel]`.
    fn classic(&self, name: &str) -> Element<'a, M> {
        use crate::components::Bevel;
        let m = self.metrics;
        let raise = self.tab_metric(name, "raise", m.space_xs);
        let body = self.tab_metric(name, "height", m.button_min_height);
        let (tab_style, tab_width) = self
            .bevel_of("frame.tab")
            .unwrap_or((crate::primitives::bevel::BevelStyle::Raised, m.border_width));
        let active = self.active_index();
        let page_top = body + raise;
        let strip_height = body + 2.0 * raise;
        let mut strip = row![]
            .spacing(self.spacing.unwrap_or(0.0))
            .align_y(Vertical::Bottom);
        for i in 0..self.items.len() {
            let on = active == Some(i);
            let item = &self.items[i];
            let mut line = row![]
                .spacing(self.icon_spacing.unwrap_or(m.space_s))
                .align_y(Vertical::Center);
            if let Some(icon) = item.icon {
                line = line.push(icon.view(m.icon_size));
            }
            let text = label(item.label.clone());
            line = line.push(if item.disabled {
                text.style(st::dim)
            } else {
                text
            });
            let height = if on { strip_height } else { body };
            let face = container(
                Bevel::new(
                    container(line).center_y(Length::Fill),
                    "frame.tab",
                    tab_style,
                    tab_width,
                )
                .padding(
                    self.padding
                        .unwrap_or(iced::Padding::from([0.0, m.space_m])),
                )
                .focused(on)
                .focus_inset(self.tab_metric(name, "focus-inset", m.space_s)),
            )
            .height(Length::Fixed(height))
            .style(|t: &Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(
                    t.token_color("frame.tab", "bg")
                        .unwrap_or(t.semantic.window.bg),
                )),
                ..Default::default()
            });
            let cell: Element<'a, M> = container(face)
                .padding(iced::Padding {
                    bottom: if on { 0.0 } else { raise },
                    ..iced::Padding::ZERO
                })
                .into();
            strip = strip.push(match self.press(i) {
                Some(msg) => iced::widget::mouse_area(cell)
                    .on_press(msg)
                    .interaction(iced::mouse::Interaction::Pointer)
                    .into(),
                None => cell,
            });
        }
        let strip = container(strip).height(Length::Fixed(strip_height));
        let panel = active
            .and_then(|i| self.items[i].panel.as_ref())
            .map(|build| build());
        let Some(panel) = panel else {
            return strip.into();
        };
        let (page_style, page_width) = self
            .bevel_of("frame.panel")
            .unwrap_or((crate::primitives::bevel::BevelStyle::Raised, m.border_width));
        let page = Bevel::new(panel, "frame.panel", page_style, page_width)
            .padding(self.padding.map_or(m.inset, |p| p.top.max(m.inset)));
        iced::widget::stack![
            container(page).width(Length::Fill).padding(iced::Padding {
                top: page_top,
                ..iced::Padding::ZERO
            }),
            strip,
        ]
        .into()
    }
}

impl<'a, K: Clone + PartialEq + 'a, M: Clone + 'a> Tabs<'a, K, M> {
    fn render(&self, name: &str) -> Element<'a, M> {
        if tab_presentation(name) == TabsVariant::Default && self.bevel_of("frame.tab").is_some() {
            return self.classic(name);
        }
        let list = self.tab_list(name, tab_presentation(name));
        let panel = self
            .active_index()
            .and_then(|i| self.items[i].panel.as_ref())
            .map(|build| build());
        match panel {
            Some(p) => column![list, p]
                .spacing(self.panel_spacing.unwrap_or(self.metrics.space_m))
                .into(),
            None => list,
        }
    }
}

fn tab_presentation(name: &str) -> TabsVariant {
    match name {
        "outline" => TabsVariant::Outline,
        "pills" => TabsVariant::Pills,
        "floating" => TabsVariant::Floating,
        _ => TabsVariant::Default,
    }
}

fn tab_name(variant: TabsVariant) -> &'static str {
    match variant {
        TabsVariant::Default => "default",
        TabsVariant::Outline => "outline",
        TabsVariant::Pills => "pills",
        TabsVariant::Floating => "floating",
    }
}

fn selected_tab(
    tokens: &crate::theme_config::Tokens,
    width: f32,
    explicit: Option<&str>,
) -> String {
    explicit.map(str::to_owned).unwrap_or_else(|| {
        crate::breakpoints::variant_for_tokens(
            tokens,
            "tabs",
            &crate::breakpoints::Breakpoints::current(),
            width,
            "default",
        )
    })
}

impl<'a, K: Clone + PartialEq + 'a, M: Clone + 'a> From<Tabs<'a, K, M>> for Element<'a, M> {
    fn from(t: Tabs<'a, K, M>) -> Self {
        if let Some(name) = t.named_variant.as_deref() {
            return t.render(name);
        }
        if t.explicit_variant {
            return t.render(tab_name(t.variant));
        }
        crate::layouts::responsive(move |size| {
            let name = t.selected_variant(size.width);
            t.render(&name)
        })
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use iced::advanced::renderer::Headless;

    fn tokens(text: &str) -> crate::theme_config::Tokens {
        let cfg = crate::theme_config::ThemeConfig::builder("tabs-variant-tests")
            .files(false)
            .app_text(text)
            .build();
        assert!(cfg.warnings().is_empty(), "{:?}", cfg.warnings());
        cfg.tokens()
    }

    #[test]
    fn breakpoint_rules_select_presentations_and_explicit_variants_win() {
        let tokens = tokens("[tabs]\nbreakpoint-rules=xs:pills,md:outline,xl:floating\n");
        for (width, expected) in [(0.0, "pills"), (800.0, "outline"), (100_000.0, "floating")] {
            assert_eq!(selected_tab(&tokens, width, None), expected);
            assert_eq!(selected_tab(&tokens, width, Some("default")), "default");
        }
        for variant in [
            TabsVariant::Default,
            TabsVariant::Outline,
            TabsVariant::Pills,
            TabsVariant::Floating,
        ] {
            assert_eq!(tab_presentation(tab_name(variant)), variant);
        }
        assert_eq!(selected_tab(&Default::default(), 800.0, None), "default");
        assert_eq!(tab_presentation("unknown"), TabsVariant::Default);
        let t = tabs(Some(0))
            .named_variant("pills")
            .variant(TabsVariant::Default);
        assert!(t.explicit_variant);
        assert!(t.named_variant.is_none());
        let t = t.named_variant("outline");
        assert!(!t.explicit_variant);
        assert_eq!(t.named_variant.as_deref(), Some("outline"));
    }

    #[test]
    fn selected_variant_geometry_overrides_base_and_inherits_metrics() {
        let tokens = tokens(
            "[tabs]\nindicator-height=3px\n[tabs.pills]\nindicator-height=7px\nclose-icon-size=0px\n",
        );
        assert_eq!(
            tab_metric_for(&tokens, "pills", "indicator-height", 9.0),
            7.0
        );
        assert_eq!(
            tab_metric_for(&tokens, "outline", "indicator-height", 9.0),
            3.0
        );
        assert_eq!(
            tab_metric_for(&tokens, "pills", "close-icon-size", 9.0),
            0.0
        );
        assert_eq!(tab_metric_for(&tokens, "pills", "focus-inset", 9.0), 9.0);
    }

    #[test]
    fn local_tokens_drive_rules_geometry_and_preserve_explicit_builders() {
        let cfg = crate::theme_config::ThemeConfig::builder("local-tabs")
            .files(false)
            .app_text("[tabs]\nbreakpoint-rules=xs:pills,md:outline\nclose-icon-size=11px\n[tabs.outline]\nclose-icon-size=23px\n")
            .build();
        assert!(cfg.warnings().is_empty(), "{:?}", cfg.warnings());
        let mut m = Metrics::desktop();
        m.focus_ring_width = 13.0;
        for t in [
            tabs(Some(0)).tokens(cfg.tokens()),
            tabs(Some(0)).theme_config(&cfg),
        ] {
            let t = t.metrics(m).spacing(19.0).padding(17.0);
            assert_eq!(t.selected_variant(0.0), "pills");
            let selected = t.selected_variant(800.0);
            assert_eq!(selected, "outline");
            assert_eq!(t.tab_metric(&selected, "close-icon-size", 99.0), 23.0);
            assert_eq!(t.tab_metric("pills", "close-icon-size", 99.0), 11.0);
            assert_eq!(
                t.tab_metric(&selected, "indicator-height", t.metrics.focus_ring_width),
                13.0
            );
            assert_eq!(t.spacing, Some(19.0));
            assert_eq!(t.padding, Some(iced::Padding::from(17.0)));
            let t = t.named_variant("pills");
            assert_eq!(t.selected_variant(800.0), "pills");
            let t = t.variant(TabsVariant::Default);
            assert_eq!(t.selected_variant(800.0), "default");
            assert_eq!(t.tab_metric("default", "close-icon-size", 99.0), 11.0);
        }
        let t = tabs(Some(0)).theme_config(&cfg).tokens(Default::default());
        assert_eq!(t.selected_variant(800.0), "default");
        assert_eq!(t.tab_metric("default", "close-icon-size", 99.0), 99.0);
        assert_eq!(t.theme_config(&cfg).selected_variant(800.0), "outline");
        assert!(
            tabs(Some(0)).tokens.is_none(),
            "no override retains global fallback"
        );
    }

    #[test]
    fn optional_tab_geometry_inherits_and_accepts_zero() {
        assert_eq!(super::resolve_tab_metric(None, 7.0), 7.0);
        assert_eq!(super::resolve_tab_metric(Some(3.0), 7.0), 3.0);
        assert_eq!(super::resolve_tab_metric(Some(0.0), 7.0), 0.0);
        assert_eq!(super::resolve_tab_metric(Some(-2.0), 7.0), 0.0);
    }

    use super::*;

    #[test]
    fn the_gaps_and_padding_are_parameters() {
        for v in [
            TabsVariant::Default,
            TabsVariant::Floating,
            TabsVariant::Pills,
            TabsVariant::Outline,
        ] {
            let t = tabs(Some(0))
                .variant(v)
                .spacing(3.0)
                .padding([1.0, 2.0])
                .icon_spacing(1.0)
                .panel_spacing(9.0);
            assert_eq!((t.spacing, t.panel_spacing), (Some(3.0), Some(9.0)));
            let _: Element<'_, u8> = t.into();
        }
    }

    fn tabs(active: Option<u8>) -> Tabs<'static, u8, u8> {
        Tabs::new(active, |k| k)
            .tab(TabItem::new(0, "One"))
            .tab(TabItem::new(1, "Two").disabled(true))
            .tab(TabItem::new(2, "Three"))
    }

    #[test]
    fn a_tab_sends_its_key_and_a_disabled_one_nothing() {
        let t = tabs(Some(0));
        assert_eq!(t.press(0), Some(0));
        assert_eq!(t.press(1), None);
        assert_eq!(t.press(2), Some(2));
        assert_eq!(t.press(7), None, "no such tab");
    }

    #[test]
    fn the_active_tab_is_found_by_key() {
        assert_eq!(tabs(Some(2)).active_index(), Some(2));
        assert_eq!(tabs(None).active_index(), None);
        assert_eq!(tabs(Some(9)).active_index(), None);
    }

    #[test]
    fn only_the_active_tabs_panel_is_built() {
        use std::cell::Cell;
        let built = Cell::new(0);
        let t: Tabs<'_, u8, u8> = Tabs::new(Some(1), |k| k)
            .tab(TabItem::new(0, "A").panel(|| {
                built.set(built.get() + 10);
                iced::widget::space().into()
            }))
            .tab(TabItem::new(1, "B").panel(|| {
                built.set(built.get() + 1);
                iced::widget::space().into()
            }));
        let mut element: Element<'_, u8> = t.into();
        assert_eq!(built.get(), 0, "responsive content is built during layout");
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut tree = iced::advanced::widget::Tree::new(element.as_widget());
        let limits =
            iced::advanced::layout::Limits::new(iced::Size::ZERO, iced::Size::new(800.0, 600.0));
        let first = element
            .as_widget_mut()
            .layout(&mut tree, &renderer, &limits);
        assert!(first.size().height < 600.0, "tabs retain shrink height");
        assert_eq!(built.get(), 1);
        element
            .as_widget_mut()
            .layout(&mut tree, &renderer, &limits);
        assert_eq!(
            built.get(),
            1,
            "same-size relayout retains responsive state"
        );
    }

    #[test]
    fn a_close_button_needs_a_closable_enabled_tab_and_a_handler() {
        let tabs = |handler: bool| {
            let mut t: Tabs<'static, u8, u8> = Tabs::new(Some(0), |k| k)
                .tab(TabItem::new(0, "Pinned"))
                .tab(TabItem::new(1, "Closable").closable(true))
                .tab(TabItem::new(2, "Disabled").closable(true).disabled(true));
            if handler {
                t = t.on_close(|k| 100 + k);
            }
            t
        };
        let t = tabs(true);
        assert!(t.close_button(0, "default").is_none(), "not closable");
        assert!(t.close_button(1, "default").is_some());
        assert!(
            t.close_button(2, "default").is_none(),
            "a disabled tab cannot be closed"
        );
        assert!(
            tabs(false).close_button(1, "default").is_none(),
            "nobody handles closing"
        );
        let _: Element<'_, u8> = t.into();
    }
}
