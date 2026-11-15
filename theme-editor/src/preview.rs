//! The live preview: a small sample UI drawn with the theme being edited, inside the editor,
//! while the editor's own chrome keeps its fixed look.
//!
//! The preview's `Theme` (colors, radii, shadows) comes from the values as they are now, through
//! `iced::widget::Themer`. Its layout metrics (paddings, heights, gaps) are passed to the widgets
//! that take them (`.metrics(m)`). Widgets that read the global config for their text size or
//! internal gaps keep the editor's own values: the gallery and the toy windows show those.

use crate::Msg;
use iad::components as cmp;
use iad::theme_config::ThemeConfig;
use iad::widgets as w;
use iad::{Icon, Metrics, Role, Scheme};
use iced::widget::{column, container, row, space, text};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

/// What the preview is built from: the theme and the layout metrics the values imply.
#[derive(Clone)]
pub(crate) struct Preview {
    pub(crate) theme: iad::Theme,
    pub(crate) metrics: Metrics,
    /// The text of the file this was built from.
    pub(crate) source: String,
    pub(crate) dark: bool,
    tree: Vec<iad::primitives::tree::TreeNode>,
    tree_state: iad::primitives::tree::TreeState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NavigationGeometry {
    pub(crate) row_height: f32,
    pub(crate) row_padding: f32,
    pub(crate) gap: f32,
    pub(crate) inset: f32,
}

impl Preview {
    /// The preview of `text` (a theme file) in the light or the dark scheme.
    pub(crate) fn build(text: &str, dark: bool) -> Self {
        let cfg = ThemeConfig::builder("org.iad.ThemeEditor.Preview")
            .files(false)
            .toolkit_text(text)
            .app_text("")
            .build();
        use iad::primitives::tree::{TreeNode, TreeState};
        let tree = vec![TreeNode::branch(
            "project",
            "Project",
            vec![
                TreeNode::branch("src", "Sources", vec![TreeNode::new("main", "main.rs")]),
                TreeNode::new("theme", "theme.conf"),
            ],
        )];
        let mut tree_state = TreeState::default();
        tree_state.expand_all(&tree);
        let scheme = if dark { Scheme::Dark } else { Scheme::Light };
        let theme = iad::Theme::new(scheme).with_theme_config(&cfg);
        let metrics = theme.metrics;
        Self {
            theme,
            tree,
            tree_state,
            metrics,
            source: text.to_string(),
            dark,
        }
    }

    /// The sample UI, wrapped so it is drawn with the preview's theme.
    pub(crate) fn view(&self) -> iad::Element<'_, Msg> {
        self.view_at(iad::breakpoints::Breakpoints::default().md)
    }

    pub(crate) fn scale(&self) -> iad::breakpoints::Breakpoints {
        let mut b = iad::breakpoints::Breakpoints::default();
        for (key, value) in [
            ("sm", &mut b.sm),
            ("md", &mut b.md),
            ("lg", &mut b.lg),
            ("xl", &mut b.xl),
            ("xxl", &mut b.xxl),
        ] {
            if let Some(n) = self.theme.token_number("breakpoints", key) {
                *value = n;
            }
        }
        if b.is_ordered() {
            b
        } else {
            Default::default()
        }
    }

    pub(crate) fn selected(&self, widget: &str, width: f32) -> String {
        iad::breakpoints::variant_for_tokens(
            &self.theme.tokens,
            widget,
            &self.scale(),
            width,
            "default",
        )
    }

    pub(crate) fn view_at(&self, width: f32) -> iad::Element<'_, Msg> {
        let sample = container(self.sample(width))
            .width(Length::Fixed(width))
            .height(Length::Fill);
        iced::widget::Themer::new(Some(self.theme.clone()), sample).into()
    }

    pub(crate) fn tree_sample(&self, width: f32) -> iad::Element<'_, Msg> {
        w::Tree::new(&self.tree, &self.tree_state)
            .tokens(self.theme.tokens.as_ref().clone())
            .metrics(self.metrics)
            .variant(self.selected("tree", width))
            .lines(true)
            .checkboxes(true)
            .on_event(|_| Msg::Noop)
            .into()
    }

    pub(crate) fn navigation_geometry(&self, width: f32) -> NavigationGeometry {
        let g = w::SidebarPadding::variant_geometry_with_tokens(
            &self.metrics,
            &self.theme.tokens,
            &self.selected("navigation", width),
        );
        NavigationGeometry {
            row_height: g.row_height,
            row_padding: g.row_padding,
            gap: g.gap,
            inset: g.inset,
        }
    }

    pub(crate) fn tabs_sample(&self, width: f32) -> iad::Element<'_, Msg> {
        w::Tabs::new(Some(0), |_| Msg::Noop)
            .tokens(self.theme.tokens.as_ref().clone())
            .tab(w::TabItem::new(0, "Overview"))
            .tab(w::TabItem::new(1, "Settings"))
            .named_variant(self.selected("tabs", width))
            .metrics(self.metrics)
            .into()
    }

    pub(crate) fn toolbar_sample(&self, width: f32) -> iad::Element<'_, Msg> {
        w::toolbar_view(text("Toolbar content"))
            .tokens(self.theme.tokens.as_ref().clone())
            .top_bar(text("Toolbar"))
            .named_variant(self.selected("toolbar", width))
            .into()
    }

    fn sample(&self, width: f32) -> iad::Element<'_, Msg> {
        let m = self.metrics;
        let ty = iad::Typography::desktop();
        let noop = || Msg::Noop;

        let header = w::header_bar()
            .title("Preview")
            .subtitle("This is how the values look")
            .pack_start(cmp::icon_button(Icon::Sidebar).on_press(noop()))
            .pack_end(cmp::icon_button(Icon::Search).on_press(noop()))
            .metrics(m);

        let buttons = row![
            cmp::raised_button(cmp::typography::label("Default")).on_press(noop()),
            cmp::flat_button(cmp::typography::label("Flat")).on_press(noop()),
            cmp::suggested_button(cmp::typography::label("Suggested")).on_press(noop()),
            cmp::destructive_button(cmp::typography::label("Destructive")).on_press(noop()),
            cmp::pill_button(cmp::typography::label("Pill")).on_press(noop()),
            cmp::circular_button(Icon::Play).on_press(noop()),
            cmp::raised_button(cmp::typography::label("Disabled")),
        ]
        .spacing(m.space_s)
        .align_y(Alignment::Center)
        .wrap();

        let rows = w::preferences_group()
            .title("Rows")
            .description("A group of rows in a card")
            .push(
                w::ActionRow::new("Action row")
                    .subtitle("With a subtitle and a suffix")
                    .suffix(cmp::switch(true).on_toggle(|_| Msg::Noop))
                    .metrics(m),
            )
            .push(w::SwitchRow::new("Switch row", false, |_| Msg::Noop).metrics(m))
            .push(
                w::ActionRow::new("Entry")
                    .suffix(
                        cmp::entry("Placeholder", "Some text")
                            .on_input(|_| Msg::Noop)
                            .width(Length::Fixed(160.0))
                            .metrics(m),
                    )
                    .metrics(m),
            )
            .metrics(m);

        let controls = row![
            cmp::check_button("Check", true).on_toggle(|_| Msg::Noop),
            cmp::check_button("Unchecked", false).on_toggle(|_| Msg::Noop),
            cmp::progress_bar(0.0..=1.0, 0.6).length(Length::Fixed(120.0)),
        ]
        .spacing(m.space_m)
        .align_y(Alignment::Center)
        .wrap();

        let tags = row![
            cmp::tag("Accent", Role::Accent),
            cmp::tag("Success", Role::Success),
            cmp::tag("Warning", Role::Warning),
            cmp::tag("Error", Role::Error),
            cmp::tag("Destructive", Role::Destructive),
        ]
        .spacing(m.space_s)
        .wrap();

        let surfaces = {
            let s = &self.theme.semantic;
            column![
                row![
                    cmp::color_swatch("window", s.window),
                    cmp::color_swatch("view", s.view),
                    cmp::color_swatch("headerbar", s.headerbar),
                ]
                .spacing(m.space_xs),
                row![
                    cmp::color_swatch("card", s.card),
                    cmp::color_swatch("sidebar", s.sidebar),
                    cmp::color_swatch("popover", s.popover),
                ]
                .spacing(m.space_xs),
                row![
                    cmp::status_swatch("accent", s.accent),
                    cmp::status_swatch("primary", s.primary),
                    cmp::status_swatch("success", s.success),
                ]
                .spacing(m.space_xs),
            ]
            .spacing(m.space_xs)
        };

        let card = w::card(
            column![
                text("Text styles").size(ty.heading),
                ty.title_3("Title"),
                ty.body("Body text sits on a card, on the window, on a view."),
                ty.caption("A caption, dimmer and smaller."),
                controls,
                tags,
            ]
            .spacing(m.space_s),
        );

        let number = |section: &str, key: &str, fallback: f32| {
            self.theme.token_number(section, key).unwrap_or(fallback)
        };
        let tree_variant = self.selected("tree", width);
        let row_variant = self.selected("tree-row", width);
        let tree = self.tree_sample(width);
        let nested = w::TreeRow::new("Nested settings")
            .tokens(self.theme.tokens.as_ref().clone())
            .metrics(m)
            .variant(row_variant.clone())
            .available_width(width)
            .expanded(true)
            .on_toggle(|_| Msg::Noop)
            .push(
                w::TreeRow::new("Child row")
                    .tokens(self.theme.tokens.as_ref().clone())
                    .metrics(m)
                    .variant(row_variant.clone())
                    .available_width(width)
                    .on_activate(Msg::Noop),
            );
        let hierarchy = w::preferences_group()
            .title("Tree & nested rows")
            .description("Expanded levels show indentation and row geometry as you edit.")
            .push(tree)
            .push(nested)
            .metrics(m);

        let navigation = {
            let geometry = self.navigation_geometry(width);
            let defaults = iad::config::Config::default();
            let min = number(
                "navigation",
                "sidebar-min-width",
                defaults.sidebar_min_width,
            );
            let max = number(
                "navigation",
                "sidebar-max-width",
                defaults.sidebar_max_width,
            )
            .max(min);
            let fraction = number("navigation", "sidebar-fraction", defaults.sidebar_fraction);
            let sidebar_width = (width * fraction).clamp(min, max).min(width);
            // SidebarList's row renderer still reads global density geometry. Build the
            // same flat destinations with explicitly resolved local dimensions instead.
            let destination = |label| {
                iced::widget::button(
                    container(text(label)).center_y(Length::Fixed(geometry.row_height)),
                )
                .padding([0.0, geometry.row_padding])
                .width(Length::Fill)
                .style(iad::styles::button::flat)
                .on_press(Msg::Noop)
            };
            let sidebar = container(
                column![destination("Overview"), destination("Settings")].spacing(geometry.gap),
            )
            .padding(geometry.inset);
            w::NavigationSplitView::new(sidebar, container(text("Content")).padding(m.card_padding))
                .sidebar_width(sidebar_width)
                .collapsed(false)
        };
        let navigation = column![text("Navigation · sidebar width"), navigation].spacing(m.space_s);

        let tabs_variant = self.selected("tabs", width);
        let toolbar_variant = self.selected("toolbar", width);
        let tabs = self.tabs_sample(width);
        let toolbar = self.toolbar_sample(width);
        let selected = text(format!(
            "Widget variants: tree={tree_variant}, tree-row={row_variant}, navigation={}, tabs={tabs_variant}, toolbar={toolbar_variant}",
            self.selected("navigation", width)
        ));
        let page = column![
            selected, tabs, toolbar, hierarchy, navigation, buttons, rows, card, surfaces
        ]
        .spacing(m.group_spacing)
        .padding(m.page_padding);

        let body = container(iad::layouts::scrolled_window(page))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|t: &iad::Theme| iced::widget::container::Style {
                background: Some(t.semantic.window.bg.into()),
                text_color: Some(t.semantic.window.fg),
                ..Default::default()
            });
        column![header, body, space().height(0)].into()
    }
}
