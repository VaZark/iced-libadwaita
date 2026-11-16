//! The tooltip every control uses: a small label in the on-screen-display style, a few pixels
//! from what it describes. Use this instead of building an `iced` tooltip by hand, so they all
//! look alike.

use crate::Element;
use iced::widget::{text, tooltip as iced_tooltip};

pub use iced_tooltip::Position as TooltipPosition;

/// The space between the tooltip and what it describes.
const GAP: f32 = 6.0;

/// How long the pointer rests on a control before its tooltip shows (`[motion] tooltip-delay`).
pub(crate) fn delay() -> std::time::Duration {
    std::time::Duration::from_secs_f32(crate::config::current().tooltip_delay_ms.max(0.0) / 1000.0)
}

/// `content` with a tooltip saying `label` on hover, on the `position` side of it.
pub fn tooltip<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    label: impl Into<String>,
    position: TooltipPosition,
) -> Element<'a, M> {
    tooltip_with_props(content, label, position, TooltipProps::default())
}

/// Presentation for the tooltip label and the existing tooltip surface.
#[derive(Clone, Default)]
pub struct TooltipProps {
    pub label_props: super::props::TextProps,
    pub container_style: Option<super::props::ContainerStyle>,
    pub gap: Option<f32>,
    pub padding: Option<iced::Pixels>,
    /// How long the pointer rests before the tooltip shows; the `[motion] tooltip-delay` setting
    /// when unset.
    pub delay: Option<std::time::Duration>,
}
impl TooltipProps {
    pub(crate) fn merge(&mut self, overrides: &Self) {
        self.label_props.merge(&overrides.label_props);
        super::props::merge_fields!(self, overrides; container_style, gap, padding, delay);
    }

    pub fn label_props(mut self, props: super::props::TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn delay(mut self, delay: std::time::Duration) -> Self {
        self.delay = Some(delay);
        self
    }
    pub fn style(
        mut self,
        style: impl Fn(&crate::Theme) -> iced::widget::container::Style + Send + Sync + 'static,
    ) -> Self {
        self.container_style = Some(std::sync::Arc::new(style));
        self
    }
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }
    pub fn padding(mut self, padding: impl Into<iced::Pixels>) -> Self {
        self.padding = Some(padding.into());
        self
    }
}

pub fn tooltip_with_props<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    label: impl Into<String>,
    position: TooltipPosition,
    props: TooltipProps,
) -> Element<'a, M> {
    let mut tooltip = iced_tooltip(
        content,
        props.label_props.apply(text(label.into()).size(12)),
        position,
    )
    .gap(props.gap.unwrap_or(GAP))
    .delay(props.delay.unwrap_or_else(delay))
    .style(|t: &crate::Theme| match t.token_color("tooltip", "bg") {
        // A theme with its own tooltip: a fill, its text color and a 1px solid border.
        Some(bg) => iced::widget::container::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: Some(
                t.token_color("tooltip", "fg")
                    .unwrap_or(t.semantic.window.fg),
            ),
            border: iced::Border {
                color: t
                    .token_color("frame.tooltip", "border")
                    .unwrap_or(iced::Color::BLACK),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        },
        None => crate::styles::container::osd(t),
    });
    if let Some(padding) = props.padding {
        tooltip = tooltip.padding(padding);
    }
    if let Some(style) = props.container_style {
        tooltip = tooltip.style(move |theme| style(theme));
    }
    tooltip.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tooltip_waits_for_the_configured_delay_unless_its_props_say_otherwise() {
        assert_eq!(delay(), std::time::Duration::from_secs(1));
        let props = TooltipProps::default().delay(std::time::Duration::ZERO);
        assert_eq!(props.delay, Some(std::time::Duration::ZERO));
        let mut base = TooltipProps::default();
        base.merge(&props);
        assert_eq!(base.delay, Some(std::time::Duration::ZERO));
    }

    #[test]
    fn it_wraps_any_content() {
        let _: Element<'_, ()> = tooltip(text("x"), "42%", TooltipPosition::Left);
    }
}
