//! `MediaCard`: an album, an artist, a playlist, an app: a picture, a title and a subtitle, drawn
//! as a **card** or, in compact mode, as a tight [`MediaRow`](super::MediaRow). One pattern in
//! two densities, so a grid of cards and a list of rows are the same data with one switch.
//!
//! * **Card**: the picture centered over the title and the subtitle, the whole card pressable.
//! * **Compact**: the picture small at the start and the title over the subtitle beside it, a
//!   flat row ([`flat_row`](crate::style::button::flat_row)).
//!
//! Put cards in a [`CardGrid`](super::CardGrid); `CardGrid::compact(true)` lays the compact form
//! out as a list.
//!
//! ```ignore
//! CardGrid::new(albums.iter().map(|a| {
//!     MediaCard::new(&a.name)
//!         .subtitle(format!("{} · {}", a.artist, a.year))
//!         .picture(cover(a.id, if compact { 36.0 } else { 88.0 }))
//!         .compact(compact)
//!         .on_press(Msg::Open(a.id))
//!         .into()
//! }))
//! .compact(compact)
//! ```

use super::{Element, MediaRow, card_button, typography as ty};
use crate::Metrics;
use crate::style::text as st;
use iced::Length;
use iced::widget::{column, container};

/// See the [module](self).
pub struct MediaCard<'a, M> {
    title: String,
    subtitle: Option<String>,
    picture: Option<Element<'a, M>>,
    compact: bool,
    on_press: Option<M>,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> MediaCard<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            picture: None,
            compact: false,
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
    /// A tight row instead of a card (default card).
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
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

impl<'a, M: Clone + 'a> From<MediaCard<'a, M>> for Element<'a, M> {
    fn from(c: MediaCard<'a, M>) -> Self {
        let m = c.metrics;
        if c.compact {
            let mut row = MediaRow::new(c.title).metrics(m);
            if let Some(s) = c.subtitle {
                row = row.subtitle(s);
            }
            if let Some(p) = c.picture {
                row = row.picture(p);
            }
            if let Some(msg) = c.on_press {
                row = row.on_press(msg);
            }
            return row.into();
        }
        let mut body = column![].spacing(m.space_xs).width(Length::Fill);
        if let Some(p) = c.picture {
            body = body.push(container(p).center_x(Length::Fill));
        }
        body = body.push(ty::heading(c.title).width(Length::Fill));
        if let Some(s) = c.subtitle {
            body = body.push(ty::caption(s).style(st::dim));
        }
        card_button(body)
            .width(Length::Fill)
            .on_press_maybe(c.on_press)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_its_parts() {
        let c: MediaCard<'_, u8> = MediaCard::new("Album")
            .subtitle("Artist")
            .picture(iced::widget::space())
            .compact(true)
            .on_press(3);
        assert!(c.compact && c.picture.is_some() && c.on_press == Some(3));
        assert_eq!(c.subtitle.as_deref(), Some("Artist"));
        assert_eq!(c.title, "Album");
    }
}
