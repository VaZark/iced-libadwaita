//! Page surfaces: the pieces a page is built from, with **the gutter in one place**.
//!
//! Every page keeps the same distance from the window edge ([`Metrics::gutter`]) because nothing
//! else in the app writes a gutter: a page is made of these. A surface that pads itself in the
//! app is a surface that drifts: one page 12 px in, the next 16, a third that forgot. Here the
//! number is a token (`[metrics] gutter` in the [theme config](crate::theme_config), 16 px, 12
//! compact), so a different look changes it once.
//!
//! | Surface | Use | Padding |
//! | --- | --- | --- |
//! | [`scrolling`] | a page that scrolls | the gutter on every side, inside the scrollable so the bar stays at the window edge |
//! | [`full`] | a page that does not scroll as a whole (a sheet's body) | the gutter on every side |
//! | [`band`] | a section, a heading, a list header | the side gutter |
//! | [`toolbar`] | a play row, a search entry | the side gutter and a little air |
//! | [`row`] | a row of a flat list | the side gutter, so its hover surface starts on the gutter |
//! | [`bar_padding`] | the padding of a bar (a player bar) | the side gutter and a little air |
//!
//! They are all a [`Container`](super::Container) (fluid) with the right padding, so a surface is
//! a container too. Enforce the rule with `cargo xtask lint-gutters`.
//!
//! ```ignore
//! surface::scrolling(column![
//!     surface::band(SectionHeading::new("Recent").link("View all", Msg::All)),
//!     FlowGrid::new(cards),
//! ])
//! ```

use super::{Container, Element, scrolled_window};
use crate::Metrics;
use iced::widget::container;
use iced::{Length, Padding};

/// The gutter in px.
pub fn gutter() -> f32 {
    Metrics::desktop().gutter
}

fn sides(m: &Metrics) -> Padding {
    Padding {
        top: 0.0,
        bottom: 0.0,
        left: m.gutter,
        right: m.gutter,
    }
}

fn base<'a, M: 'a>(content: impl Into<Element<'a, M>>, padding: Padding) -> Element<'a, M> {
    Container::new(content).fluid(true).padding(padding).into()
}

/// A page that scrolls. The surface is inside the scrollable, so the scroll bar stays at the
/// window edge while the content keeps the gutter on every side.
pub fn scrolling<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let g = gutter();
    scrolled_window(base(content, Padding::from(g)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// A page that does not scroll as a whole (a jam, a sheet's body): the gutter on every side.
pub fn full<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(base(content, Padding::from(gutter())))
        .height(Length::Fill)
        .into()
}

/// A band inside a page (a section, a heading, a list header): the side gutter only.
pub fn band<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    base(content, sides(&Metrics::desktop()))
}

/// A toolbar band (the play row, a search entry): the side gutter and a little air.
pub fn toolbar<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    base(content, bar_padding())
}

/// A row of a flat list: inset so its hover surface starts on the gutter.
pub fn row<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(base(content, sides(&Metrics::desktop())))
        .height(Length::Fill)
        .into()
}

/// The padding of a bar (a player bar): the gutter at the sides, a little air above and below.
pub fn bar_padding() -> Padding {
    let m = Metrics::desktop();
    Padding {
        top: m.band_air,
        bottom: m.band_air,
        ..sides(&m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gutter_is_a_token_and_a_bar_adds_air() {
        let m = Metrics::desktop();
        assert_eq!(gutter(), m.gutter);
        let p = bar_padding();
        assert_eq!((p.left, p.right), (m.gutter, m.gutter));
        assert_eq!((p.top, p.bottom), (m.band_air, m.band_air));
        assert!(Metrics::compact().gutter < Metrics::default().gutter);
    }
}
