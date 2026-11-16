//! Local config-owned navigation densities beside the shell's compatibility controls.
use crate::{Element, Message};
use iad::components::typography as ty;
use iad::theme_config::{Schema, ThemeConfig};
use iad::{Metrics, layouts, widgets as w};
use iced::Length;
use iced::widget::{column, container};
use iced_adaptive_design as iad;
use std::sync::LazyLock;

const CONFIG_TEXT: &str = include_str!("navigation_demo.conf");
const DESTINATIONS: [(&str, &str); 2] = [("Inbox", "Opened Inbox"), ("Archive", "Opened Archive")];

fn schema() -> Schema {
    Schema::toolkit()
        .variant("navigation", "reading", &[])
        .variant("navigation", "scan", &[])
        .variant("button", "gallery-navigation", &[])
}

fn config() -> &'static ThemeConfig {
    static CONFIG: LazyLock<ThemeConfig> = LazyLock::new(|| {
        let schema = schema();
        let report = schema.validate(CONFIG_TEXT);
        assert!(
            report.errors.is_empty(),
            "navigation demo: {:?}",
            report.errors
        );
        assert!(
            report.unknown.is_empty(),
            "navigation demo: {:?}",
            report.unknown
        );
        let config = ThemeConfig::builder("org.iad.Gallery.NavigationDemo")
            .files(false)
            .schema(schema)
            .toolkit_text(CONFIG_TEXT)
            .app_text("")
            .build();
        assert!(
            config.warnings().is_empty(),
            "navigation demo: {:?}",
            config.warnings()
        );
        config
    });
    &CONFIG
}

fn target_style(
    config: &ThemeConfig,
    theme: &iad::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let mut target = iad::styles::button::flat(theme, status);
    if let Some(radius) = config
        .tokens()
        .variant("button", "gallery-navigation")
        .length("radius")
    {
        target.border.radius = radius.into();
    }
    target
}

fn preview(config: &ThemeConfig, metrics: Metrics, explicit: Option<&str>) -> Element<'static> {
    let config = config.clone();
    let explicit = explicit.map(str::to_owned);
    layouts::responsive(move |size| {
        let variant = explicit.clone().unwrap_or_else(|| {
            w::SidebarPadding::variant_for_width_with_tokens(
                &config.tokens(),
                size.width,
                "reading",
            )
        });
        let geometry =
            w::SidebarPadding::variant_geometry_with_tokens(&metrics, &config.tokens(), &variant);
        // SidebarList fills its pane. Size this small pane from the resolved row geometry,
        // rather than assigning it an unrelated demo height or the whole scrolling page.
        let rows = DESTINATIONS.len() as f32;
        let height =
            rows * geometry.row_height + (rows - 1.0) * geometry.gap + 2.0 * geometry.inset;
        let mut list = w::SidebarList::new(DESTINATIONS.map(|(name, result)| {
            let config = config.clone();
            w::SidebarItem::new(name, Message::ShowToast(result))
                .selected(name == DESTINATIONS[0].0)
                .row_props(
                    w::SidebarRowProps::default().button_props(
                        iad::components::ButtonProps::default()
                            .style(move |theme, status| target_style(&config, theme, status)),
                    ),
                )
        }))
        .metrics(metrics)
        .theme_config(&config)
        .available_width(size.width);
        if let Some(name) = &explicit {
            list = list.variant(name);
        }
        container(list).height(height).into()
    })
    .height(Length::Shrink)
    .into()
}

pub fn view(metrics: Metrics) -> Element<'static> {
    column![
        ty::caption("Explicit .variant(\"reading\"): a custom name, not an enum preset. Inbox is selected; hover either row to compare effects on the config-owned target shape."),
        preview(config(), metrics, Some("reading")),
        ty::caption("Local xs:reading,md:scan rules: resize the gallery. Thresholds and densities come from navigation_demo.conf; scan inherits base padding, gap and inset."),
        preview(config(), metrics, None),
    ]
    .spacing(metrics.space_m)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{layout, renderer::Headless, widget::Tree};
    use iced::{Rectangle, Size};

    fn rectangles(node: &layout::Node) -> Vec<Rectangle> {
        fn collect(layout: layout::Layout<'_>, out: &mut Vec<Rectangle>) {
            out.push(layout.bounds());
            for child in layout.children() {
                collect(child, out);
            }
        }
        let mut out = Vec::new();
        collect(layout::Layout::new(node), &mut out);
        out
    }

    #[test]
    fn selected_and_hoverable_demo_rows_use_the_config_owned_target_shape() {
        let config = config();
        let radius = config
            .number("button.gallery-navigation", "radius")
            .unwrap();
        let theme = iad::Theme::light();
        for status in [
            iced::widget::button::Status::Active,
            iced::widget::button::Status::Hovered,
            iced::widget::button::Status::Pressed,
            iced::widget::button::Status::Disabled,
        ] {
            let target = target_style(config, &theme, status);
            assert_eq!(target.border.radius, iced::border::Radius::from(radius));
            let inherited = iad::styles::button::flat(&theme, status);
            assert_eq!(target.background, inherited.background);
            assert_eq!(target.text_color, inherited.text_color);
        }
        let _ = view(Metrics::default());
    }

    #[test]
    fn validated_custom_densities_construct_and_resize_using_local_rules() {
        let config = config();
        assert!(schema().validate(CONFIG_TEXT).is_valid());
        assert!(config.unknown().is_empty());
        let tokens = config.tokens();
        let metrics = Metrics::default();
        let threshold = config.number("breakpoints", "md").unwrap();
        let inset = config.number("navigation", "inset").unwrap();
        let gap = config.number("navigation", "gap").unwrap();
        let narrow = threshold - inset;
        let wide = threshold + inset;
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(iad::Typography::default().body),
            Some("tiny-skia"),
        ))
        .unwrap();
        for explicit in [None, Some("reading")] {
            let mut element = preview(config, metrics, explicit);
            let mut tree = Tree::new(&element);
            // Reuse the same responsive element/tree on both sides of the local boundary.
            for width in [narrow, threshold, wide, narrow] {
                let expected =
                    explicit.unwrap_or(if width < threshold { "reading" } else { "scan" });
                assert_eq!(
                    w::SidebarPadding::variant_for_width_with_tokens(&tokens, width, "reading"),
                    if width < threshold { "reading" } else { "scan" }
                );
                let row_height = config
                    .number(&format!("navigation.{expected}"), "row-height")
                    .unwrap();
                let height = DESTINATIONS.len() as f32 * row_height + gap + 2.0 * inset;
                let node = element.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(width, metrics.clamp_maximum)),
                );
                assert_eq!(node.size(), Size::new(width, height));
                let rects = rectangles(&node);
                for row in 0..DESTINATIONS.len() {
                    let expected_rect = Rectangle::new(
                        iced::Point::new(inset, inset + row as f32 * (row_height + gap)),
                        Size::new(width - 2.0 * inset, row_height),
                    );
                    assert!(
                        rects.contains(&expected_rect),
                        "missing {expected_rect:?} in {rects:?}"
                    );
                }
            }
        }
        let scan = w::SidebarPadding::variant_geometry_with_tokens(&metrics, &tokens, "scan");
        assert_eq!(
            scan.row_padding,
            config.number("navigation", "row-padding").unwrap()
        );
        let _ = view(metrics);
    }
}
