//! Test-only local-token fixtures for theme schema and widget-layout regressions.
//! Gallery and toy windows are the editors interactive previews.

use crate::Msg;
use iad::theme_config::ThemeConfig;
use iad::widgets as w;
use iad::{Metrics, Scheme};
use iced::Length;
use iced::widget::{column, container, text};
use iced_adaptive_design as iad;

/// What the preview is built from: the theme and the layout metrics the values imply.
#[derive(Clone)]
pub(crate) struct Preview {
    pub(crate) theme: iad::Theme,
    pub(crate) metrics: Metrics,
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
        column![
            self.tree_sample(width),
            self.tabs_sample(width),
            self.toolbar_sample(width)
        ]
        .into()
    }
}
