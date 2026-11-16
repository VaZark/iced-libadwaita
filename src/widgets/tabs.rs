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
use crate::components::{ButtonProps, ContainerProps, TextProps};
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
    label_props: TextProps,
    tab_button_props: ButtonProps,
    close_button_props: ButtonProps,
    tab_container_props: ContainerProps,
    indicator_props: ContainerProps,
    separator_props: ContainerProps,
    panel_props: Option<ContainerProps>,
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
    /// Owned presentation overrides for the concrete child.
    pub fn label_props(mut self, props: TextProps) -> Self {
        self.label_props = props;
        self
    }
    /// Owned presentation overrides for the concrete child.
    pub fn tab_button_props(mut self, props: ButtonProps) -> Self {
        self.tab_button_props = props;
        self
    }
    /// Owned presentation overrides for the concrete child.
    pub fn close_button_props(mut self, props: ButtonProps) -> Self {
        self.close_button_props = props;
        self
    }
    /// Presentation overrides for the concrete tab_container container.
    pub fn tab_container_props(mut self, props: ContainerProps) -> Self {
        self.tab_container_props = props;
        self
    }
    /// Presentation overrides for the concrete indicator container.
    pub fn indicator_props(mut self, props: ContainerProps) -> Self {
        self.indicator_props = props;
        self
    }
    /// Presentation overrides for the concrete separator container.
    pub fn separator_props(mut self, props: ContainerProps) -> Self {
        self.separator_props = props;
        self
    }
    /// Presentation overrides for the concrete panel container.
    pub fn panel_props(mut self, props: ContainerProps) -> Self {
        self.panel_props = Some(props);
        self
    }
    pub fn new(active: Option<K>, on_change: impl Fn(K) -> M + 'a) -> Self {
        Self {
            tab_container_props: ContainerProps::default(),
            indicator_props: ContainerProps::default(),
            separator_props: ContainerProps::default(),
            panel_props: None,
            label_props: TextProps::default(),
            tab_button_props: ButtonProps::default(),
            close_button_props: ButtonProps::default(),
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
            self.close_button_props
                .apply(
                    button(Icon::WindowClose.view(self.tab_metric(
                        name,
                        "close-icon-size",
                        self.metrics.icon_size,
                    )))
                    .padding(self.metrics.space_xs)
                    .style(sb::circular_flat)
                    .on_press(f(item.key.clone())),
                )
                .into(),
        )
    }
    /// The message a press on tab `i` sends: none when it is disabled.
    fn press(&self, i: usize) -> Option<M> {
        self.list()
            .activate(i)
            .map(|i| (self.on_change)(self.items[i].key.clone()))
    }

    fn tab_style(
        &self,
        active: bool,
        variant: TabsVariant,
    ) -> crate::components::props::ButtonStyle {
        self.tab_button_props.style.clone().unwrap_or_else(|| {
            std::sync::Arc::new(move |theme, status| match variant {
                TabsVariant::Pills if active => sb::suggested_pill(theme, status),
                _ => sb::flat(theme, status),
            })
        })
    }

    /// One tab's button, in the look of the variant.
    fn tab_button(
        &self,
        i: usize,
        active: bool,
        name: &str,
        variant: TabsVariant,
        status: Option<std::rc::Rc<std::cell::Cell<button::Status>>>,
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
        content = content.push(self.label_props.apply(text(item.label.clone())));
        let b = button(content)
            .padding(
                self.padding
                    .unwrap_or(iced::Padding::from([m.space_xs, m.space_m])),
            )
            .on_press_maybe(self.press(i));
        let style = self.tab_style(active, variant);
        let title: Element<'a, M> = self
            .tab_button_props
            .apply(b)
            .style(move |theme, status| style(theme, status))
            .into();
        let title = match status {
            Some(status) => iced::Element::new(TabButtonTarget {
                content: title,
                enabled: !item.disabled,
                status,
            }),
            None => title,
        };
        match self.close_button(i, name) {
            Some(close) => row![title, close].align_y(Vertical::Center).into(),
            None => title,
        }
    }

    fn tab_list(&self, name: &str, variant: TabsVariant) -> Element<'a, M> {
        let m = &self.metrics;
        let active = self.active_index();
        if variant == TabsVariant::Floating {
            let statuses: Vec<_> = self
                .items
                .iter()
                .map(|_| std::rc::Rc::new(std::cell::Cell::new(button::Status::Disabled)))
                .collect();
            let tabs = (0..self.items.len())
                .map(|i| {
                    self.tab_button(
                        i,
                        active == Some(i),
                        name,
                        variant,
                        Some(statuses[i].clone()),
                    )
                })
                .collect();
            let styles: Vec<_> = (0..self.items.len())
                .map(|i| self.tab_style(active == Some(i), variant))
                .collect();

            let closable: Vec<_> = (0..self.items.len())
                .map(|i| self.close_button(i, name).is_some())
                .collect();
            return FloatingIndicator::new(tabs, active.unwrap_or(usize::MAX))
                .spacing(self.spacing.unwrap_or(m.space_xs))
                .target_bounds(move |i, child| floating_title_bounds(closable[i], child))
                .target_shape(move |theme, i, _| styles[i](theme, statuses[i].get()).border.radius)
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
            let button = self.tab_button(i, on, name, variant, None);
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
                    column![button, self.indicator_props.apply(line)].into()
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
            tabs = tabs.push(self.tab_container_props.apply(container(tab).width(width)));
        }
        let tabs = tabs.width(if self.grow {
            Length::Fill
        } else {
            Length::Shrink
        });
        match variant {
            TabsVariant::Default | TabsVariant::Outline => column![
                tabs,
                self.separator_props.apply(
                    container(space().height(self.tab_metric(
                        name,
                        "separator-width",
                        m.border_width
                    )))
                    .width(Length::Fill)
                    .style(sc::card_separator)
                )
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
            line = line.push(self.label_props.apply(if item.disabled {
                text.style(st::dim)
            } else {
                text
            }));
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
            .padding(self.tab_metric(name, "panel-padding", m.inset));
        iced::widget::stack![
            self.panel_props.clone().unwrap_or_default().apply(
                container(page)
                    .width(Length::Fill)
                    .style(|t: &Theme| iced::widget::container::Style {
                        background: Some(
                            t.token_color("frame.panel", "bg")
                                .unwrap_or(t.semantic.window.bg)
                                .into()
                        ),
                        ..Default::default()
                    })
                    .padding(iced::Padding {
                        top: page_top,
                        ..iced::Padding::ZERO
                    })
            ),
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
            Some(p) => {
                let p = match &self.panel_props {
                    Some(props) => props.apply(container(p)).into(),
                    None => p,
                };
                column![list, p]
                    .spacing(self.panel_spacing.unwrap_or(self.metrics.space_m))
                    .into()
            }
            None => list,
        }
    }
}

// Iced keeps Button::status private. Mirror its input status at the same redraw
// boundary so the indicator can resolve the identical style before the button draws.
struct TabButtonTarget<'a, M> {
    content: Element<'a, M>,
    enabled: bool,
    status: std::rc::Rc<std::cell::Cell<button::Status>>,
}

#[derive(Default)]
struct TabButtonState {
    pressed: bool,
}

impl TabButtonState {
    fn update(&mut self, event: &iced::Event, over: bool, enabled: bool) -> button::Status {
        use iced::{Event, mouse, touch};
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
                if enabled && over =>
            {
                self.pressed = true;
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => self.pressed = false,
            _ => {}
        }
        if !enabled {
            button::Status::Disabled
        } else if over && self.pressed {
            button::Status::Pressed
        } else if over {
            button::Status::Hovered
        } else {
            button::Status::Active
        }
    }
}

impl<'a, M: 'a> iced::advanced::Widget<M, Theme, iced::Renderer> for TabButtonTarget<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<TabButtonState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(TabButtonState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(&[&self.content]);
    }
    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }
    fn size_hint(&self) -> iced::Size<Length> {
        self.content.as_widget().size_hint()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        let captured = shell.is_event_captured();
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if !captured {
            let status = tree.state.downcast_mut::<TabButtonState>().update(
                event,
                cursor.is_over(layout.bounds()),
                self.enabled,
            );
            if matches!(
                event,
                iced::Event::Window(iced::window::Event::RedrawRequested(_))
            ) {
                self.status.set(status);
            }
        }
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

fn floating_title_bounds(closable: bool, child: iced::advanced::Layout<'_>) -> iced::Rectangle {
    if closable {
        child
            .children()
            .next()
            .expect("closable tab title")
            .bounds()
    } else {
        child.bounds()
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

    #[test]
    fn floating_indicator_and_title_resolve_identical_hover_press_and_disabled_styles() {
        use super::*;
        use iced::advanced::{layout, renderer, widget::Tree};
        use iced::{Event, Point, Rectangle, Size, mouse};
        let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(11.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for disabled in [false, true] {
            let calls = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
            let observed = calls.clone();
            let tabs = Tabs::new(Some(0), |_| ())
                .variant(TabsVariant::Floating)
                .tab_button_props(ButtonProps::default().style(move |theme, status| {
                    observed.lock().unwrap().push(status);
                    let radius = match status {
                        button::Status::Active | button::Status::Disabled => 0.0.into(),
                        button::Status::Hovered => 100.0.into(),
                        button::Status::Pressed => iced::border::Radius {
                            top_left: theme.metrics.radius_m,
                            top_right: 0.0,
                            bottom_right: 3.0,
                            bottom_left: 7.0,
                        },
                    };
                    button::Style {
                        border: iced::Border::default().rounded(radius),
                        ..button::Style::default()
                    }
                }))
                .tab(TabItem::new(0, "Title").disabled(disabled));
            let mut element = tabs.tab_list("floating", TabsVariant::Floating);
            let mut tree = Tree::new(&element);
            let viewport = Rectangle::with_size(Size::new(500.0, 100.0));
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport.size()),
            );
            let layout = layout::Layout::new(&node);
            let mut messages = vec![];
            for (event, cursor, expected) in [
                (
                    Event::Mouse(mouse::Event::CursorMoved {
                        position: Point::new(2.0, 2.0),
                    }),
                    mouse::Cursor::Available(Point::new(2.0, 2.0)),
                    button::Status::Hovered,
                ),
                (
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    mouse::Cursor::Available(Point::new(2.0, 2.0)),
                    button::Status::Pressed,
                ),
                (
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                    mouse::Cursor::Available(Point::new(2.0, 2.0)),
                    button::Status::Hovered,
                ),
                (
                    Event::Mouse(mouse::Event::CursorLeft),
                    mouse::Cursor::Unavailable,
                    button::Status::Active,
                ),
            ] {
                for event in [
                    event,
                    Event::Window(iced::window::Event::RedrawRequested(
                        std::time::Instant::now(),
                    )),
                ] {
                    element.as_widget_mut().update(
                        &mut tree,
                        &event,
                        layout,
                        cursor,
                        &renderer,
                        &mut iced::advanced::clipboard::Null,
                        &mut iced::advanced::Shell::new(&mut messages),
                        &viewport,
                    );
                }
                calls.lock().unwrap().clear();
                element.as_widget().draw(
                    &tree,
                    &mut renderer,
                    &Theme::light(),
                    &renderer::Style::default(),
                    layout,
                    cursor,
                    &viewport,
                );
                let expected = if disabled {
                    button::Status::Disabled
                } else {
                    expected
                };
                assert_eq!(*calls.lock().unwrap(), vec![expected, expected]);
            }
            assert_eq!(messages.len(), usize::from(!disabled));
        }
    }

    #[test]
    fn floating_shape_uses_the_same_current_theme_button_style() {
        use super::*;
        let props = ButtonProps::default().style(|theme, _| button::Style {
            border: iced::Border::default().rounded(iced::border::Radius {
                top_left: theme.metrics.radius_m,
                top_right: 0.0,
                bottom_right: theme.metrics.radius_m * 2.0,
                bottom_left: 1.0,
            }),
            ..button::Style::default()
        });
        let tabs = Tabs::new(Some(0), |_| ()).tab_button_props(props.clone());
        let shared = tabs.tab_style(true, TabsVariant::Floating);
        assert!(std::sync::Arc::ptr_eq(
            &shared,
            props.style.as_ref().unwrap()
        ));
        for radius in [0.0, 5.0, 30.0] {
            let mut theme = Theme::light();
            theme.metrics.radius_m = radius;
            let actual = shared(&theme, button::Status::Active).border.radius;
            assert_eq!(actual.top_left, radius);
            assert_eq!(actual.top_right, 0.0);
            assert_eq!(actual.bottom_right, radius * 2.0);
        }
        let default = Tabs::new(Some(0), |_| ()).tab_style(true, TabsVariant::Floating);
        let theme = Theme::light();
        assert_eq!(
            default(&theme, button::Status::Active).border.radius,
            sb::flat(&theme, button::Status::Active).border.radius
        );
    }

    #[test]
    fn floating_closable_target_is_title_not_row_or_close_button() {
        use super::*;
        use iced::Size;
        use iced::advanced::{layout, widget::Tree};
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(11.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let tabs = Tabs::new(Some(0), |_| ())
            .variant(TabsVariant::Floating)
            .tab_button_props(ButtonProps::default().height(iced::Length::Fixed(20.0)))
            .close_button_props(ButtonProps::default().height(iced::Length::Fixed(60.0)))
            .on_close(|_| ())
            .tab(TabItem::new(0, "Title").closable(true))
            .tab(TabItem::new(1, "Plain"));
        let mut element = tabs.tab_list("floating", TabsVariant::Floating);
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 100.0)),
        );
        let layout = layout::Layout::new(&node);
        let closable = layout.children().next().unwrap();
        let target = floating_title_bounds(true, closable);
        assert_eq!(target.height, 20.0);
        assert_eq!(closable.bounds().height, 60.0);
        assert_eq!(target.y, 20.0);
        assert!(target.width < closable.bounds().width);
        let plain = layout.children().nth(1).unwrap();
        assert_eq!(floating_title_bounds(false, plain), plain.bounds());
    }

    #[test]
    fn classic_tabs_share_panel_surface_and_resolve_panel_padding_from_tokens() {
        use crate::{Element, Theme};
        use iced::advanced::{layout, renderer, widget::Tree};
        use iced::{Color, Point, Rectangle, Size, mouse};
        let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(11.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut heights = Vec::new();
        for padding in [0, 9] {
            let cfg = crate::theme_config::ThemeConfig::builder("classic-tab-regression")
                .files(false)
                .toolkit_text(include_str!("../../themes/win95.ron"))
                .app_text(format!("{{ \"tabs\": {{ \"panel-padding\": {padding} }} }}"))
                .build();
            assert!(cfg.warnings().is_empty(), "{:?}", cfg.warnings());
            let theme = Theme::light().with_theme_config(&cfg);
            let tabs = super::Tabs::new(Some(0usize), |_| ())
                .theme_config(&cfg)
                .metrics(theme.metrics)
                .padding(4)
                .variant(super::TabsVariant::Default)
                .tab(super::TabItem::new(0usize, " ").panel(|| {
                    iced::widget::container(iced::widget::space().width(120).height(40)).into()
                }))
                .tab(super::TabItem::new(1usize, " ").panel(|| iced::widget::space().into()));
            let mut element: Element<'_, ()> = tabs.into();
            let mut tree = Tree::new(&element);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(160.0, 120.0)),
            );
            heights.push(node.size().height);
            element.as_widget().draw(
                &tree,
                &mut renderer,
                &theme,
                &renderer::Style::default(),
                layout::Layout::new(&node),
                mouse::Cursor::Unavailable,
                &Rectangle::new(Point::ORIGIN, Size::new(160.0, 120.0)),
            );
            let pixels = renderer.screenshot(Size::new(160, 120), 1.0, Color::WHITE);
            let pixel = |x: usize, y: usize| &pixels[(y * 160 + x) * 4..(y * 160 + x) * 4 + 4];
            // Blank labels let these samples isolate actual tab/panel fills, not glyphs.
            assert_eq!(pixel(6, 10), &[212, 208, 200, 255]);
            assert_eq!(pixel(6, 32), pixel(6, 10));
        }
        assert_eq!(heights[1] - heights[0], 18.0);
    }

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
        let tokens = tokens(r##"{
    "tabs": {
        "breakpoint-rules": "xs:pills,md:outline,xl:floating",
    },
}
"##);
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
            r##"{
    "tabs": {
        "indicator-height": 3,
    },
    "tabs.pills": {
        "indicator-height": 7,
        "close-icon-size": 0,
    },
}
"##,
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
            .app_text(r##"{
    "tabs": {
        "breakpoint-rules": "xs:pills,md:outline",
        "close-icon-size": 11,
    },
    "tabs.outline": {
        "close-icon-size": 23,
    },
}
"##)
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
