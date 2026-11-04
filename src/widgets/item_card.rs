//! `ItemCard`: an album, an artist, a playlist, an app: a picture, a title and a subtitle, drawn
//! as a **card** or, in compact mode, as a tight [`ItemRow`](crate::widgets::ItemRow). One pattern in
//! two densities, so a grid of cards and a list of rows are the same data with one switch.
//!
//! * **Comfort** (default): the picture centered over the title and the subtitle, the whole card pressable.
//! * **Compact**: the picture small at the start and the title over the subtitle beside it, a
//!   flat row ([`flat_row`](crate::styles::button::flat_row)).
//!
//! The density is the item's own. How items are arranged is the container's: a
//! [`FlowGrid`](crate::layouts::FlowGrid) with several columns makes a grid, with one column a list, and a
//! [`TableGrid`](crate::layouts::TableGrid) gives each its span.
//!
//! ```ignore
//! FlowGrid::new(albums.iter().map(|a| {
//!     ItemCard::new(&a.name)
//!         .subtitle(format!("{} · {}", a.artist, a.year))
//!         .picture(cover(a.id, if density == Density::Compact { 36.0 } else { 88.0 }))
//!         .density(density)
//!         .on_press(Msg::Open(a.id))
//!         .into()
//! }))
//! .cols(if density == Density::Compact { 1 } else { 4 })
//! ```

use crate::Element;
use crate::Metrics;
use crate::components::typography as ty;
use crate::config::Density;
use crate::styles::text as st;
use crate::widgets::ItemRow;
use iced::Length;
use iced::widget::{column, container};

/// See the [module](self).
pub struct ItemCard<'a, M> {
    title: String,
    subtitle: Option<String>,
    picture: Option<Element<'a, M>>,
    density: Density,
    on_press: Option<M>,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> ItemCard<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            picture: None,
            density: Density::Comfort,
            on_press: None,
            metrics: Metrics::desktop(),
        }
    }
    /// The dimmed line under the title: the artist, the year, a count.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
    /// The picture: a cover, an avatar. Make it big for a card and small for the compact row.
    pub fn picture(mut self, picture: impl Into<Element<'a, M>>) -> Self {
        self.picture = Some(picture.into());
        self
    }
    /// [`Comfort`](Density::Comfort): a card. [`Compact`](Density::Compact): a tight row.
    pub fn density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }
    pub fn on_press(mut self, m: M) -> Self {
        self.on_press = Some(m);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<ItemCard<'a, M>> for Element<'a, M> {
    fn from(c: ItemCard<'a, M>) -> Self {
        let m = c.metrics;
        if c.density == Density::Compact {
            let mut row = ItemRow::new(c.title).metrics(m).inset();
            if let Some(s) = c.subtitle {
                row = row.subtitle(s);
            }
            if let Some(p) = c.picture {
                row = row.picture(p);
            }
            return iced::widget::button(row)
                .padding(0)
                .width(Length::Fill)
                .style(crate::styles::button::flat_row(8.0, 0.08, 0.14))
                .on_press_maybe(c.on_press)
                .into();
        }
        let mut body = column![].spacing(m.space_xs).width(Length::Fill);
        if let Some(p) = c.picture {
            body = body.push(container(p).center_x(Length::Fill));
        }
        // Text never leaves the card: what does not fit is clipped at its edge.
        body = body.push(clipped(ty::heading(c.title).width(Length::Fill)));
        if let Some(s) = c.subtitle {
            body = body.push(clipped(ty::caption(s).style(st::dim)));
        }
        // The card draws its own surface; the button only adds the events.
        let surface = container(body)
            .padding(m.card_padding)
            .width(Length::Fill)
            .style(crate::styles::container::card);
        iced::widget::button(surface)
            .padding(0)
            .style(crate::styles::button::events)
            .width(Length::Fill)
            .on_press_maybe(c.on_press)
            .into()
    }
}

/// `content` cut off at its own bounds, so a long title cannot spill over its neighbours.
fn clipped<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content).width(Length::Fill).clip(true).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_its_parts() {
        let c: ItemCard<'_, u8> = ItemCard::new("Album")
            .subtitle("Artist")
            .picture(iced::widget::space())
            .density(Density::Compact)
            .on_press(3);
        assert!(c.density == Density::Compact && c.picture.is_some() && c.on_press == Some(3));
        assert_eq!(c.subtitle.as_deref(), Some("Artist"));
        assert_eq!(c.title, "Album");
    }
}
