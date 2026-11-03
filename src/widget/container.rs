//! `Container`: centers its content and limits its width to a named size, after Mantine's
//! [Container](https://mantine.dev/core/container/). The base of a content page: every page
//! reads at the same width whatever the window, and the same sizes name the same widths
//! everywhere.
//!
//! | Size | Max width |
//! | --- | --- |
//! | [`ContainerSize::Xs`] | 540 |
//! | [`ContainerSize::Sm`] | 720 |
//! | [`ContainerSize::Md`] (default) | 960 |
//! | [`ContainerSize::Lg`] | 1140 |
//! | [`ContainerSize::Xl`] | 1320 |
//!
//! * The width includes the padding (border-box), so a `Md` container with the default padding
//!   holds content 960 minus both sides.
//! * [`fluid`](Container::fluid) drops the limit: the container fills its parent.
//! * The padding is horizontal by default, [`Metrics::page_padding`] each side (Mantine's
//!   container pads the sides only). [`padding`](Container::padding) takes any
//!   [`Padding`].
//! * A container does not scroll. Put it in a [`scrolled_window`](super::scrolled_window) for
//!   a page ([`Page`](super::Page) does).
//!
//! ```ignore
//! scrolled_window(Container::new(content).size(ContainerSize::Sm))
//! ```
//!
//! [`AdwClamp`](super::clamp) is the same idea in pixels; this is the named-size form.

use super::Element;
use crate::Metrics;
use iced::widget::container;
use iced::{Length, Padding};

/// A named maximum width.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ContainerSize {
    /// 540 px.
    Xs,
    /// 720 px.
    Sm,
    /// 960 px.
    #[default]
    Md,
    /// 1140 px.
    Lg,
    /// 1320 px.
    Xl,
    /// Any width in px.
    Px(f32),
}

impl ContainerSize {
    /// The maximum width in px.
    pub fn width(self) -> f32 {
        match self {
            Self::Xs => 540.0,
            Self::Sm => 720.0,
            Self::Md => 960.0,
            Self::Lg => 1140.0,
            Self::Xl => 1320.0,
            Self::Px(px) => px.max(0.0),
        }
    }
}

/// See the [module](self).
pub struct Container<'a, M> {
    content: Element<'a, M>,
    size: ContainerSize,
    fluid: bool,
    padding: Option<Padding>,
    metrics: Metrics,
}

impl<'a, M: 'a> Container<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            size: ContainerSize::Md,
            fluid: false,
            padding: None,
            metrics: Metrics::desktop(),
        }
    }
    /// The maximum width.
    pub fn size(mut self, size: ContainerSize) -> Self {
        self.size = size;
        self
    }
    /// No maximum width: fills its parent.
    pub fn fluid(mut self, fluid: bool) -> Self {
        self.fluid = fluid;
        self
    }
    /// The space between the edge and the content, in place of the horizontal page padding.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<Container<'a, M>> for Element<'a, M> {
    fn from(c: Container<'a, M>) -> Self {
        let page = c.metrics.page_padding;
        let padding = c.padding.unwrap_or(Padding {
            top: 0.0,
            bottom: 0.0,
            ..page
        });
        let inner = container(c.content).padding(padding).width(Length::Fill);
        let inner = if c.fluid {
            inner
        } else {
            inner.max_width(c.size.width())
        };
        container(inner).center_x(Length::Fill).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_mantines() {
        let w: Vec<f32> = [
            ContainerSize::Xs,
            ContainerSize::Sm,
            ContainerSize::Md,
            ContainerSize::Lg,
            ContainerSize::Xl,
        ]
        .iter()
        .map(|s| s.width())
        .collect();
        assert_eq!(w, vec![540.0, 720.0, 960.0, 1140.0, 1320.0]);
        assert_eq!(ContainerSize::default(), ContainerSize::Md);
        assert_eq!(ContainerSize::Px(-5.0).width(), 0.0);
    }
}
