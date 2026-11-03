//! `MediaRow`: one row of a music, podcast, video or file list. A flat, rounded, whole-row
//! button holding a picture, a title with a subtitle, optional extra columns, and a trailing
//! part (a time, a menu). Together with [`VirtualList`](super::VirtualList) and a
//! [`SortHeader`](super::SortHeader) it is the list component.
//!
//! * **Regular**: picture, then the title over its subtitle.
//! * **Compact** ([`compact`](MediaRow::compact)): no picture, and the title and subtitle share
//!   the line, each in its own column, for dense lists.
//! * [`column`](MediaRow::column) adds a cell of a given width (an album, a BPM, a time);
//!   [`suffix`](MediaRow::suffix) adds the trailing part (a menu button).
//!
//! The whole row is the target, rounded, with a hover that reads on a plain page
//! ([`style::button::flat_row`](crate::style::button::flat_row)).
//!
//! ```ignore
//! MediaRow::new(&track.title)
//!     .subtitle(&track.artist)
//!     .picture(cover(track.id, 40.0))
//!     .column(Length::FillPortion(2), caption(&track.album))
//!     .column(Length::Fixed(44.0), caption(track.time()))
//!     .suffix(menu_button(Icon::ViewMore, menu))
//!     .on_press(Msg::Play(track.id))
//! ```

use super::{Element, typography as ty};
use crate::Metrics;
use crate::style::button as sb;
use crate::style::text as st;
use iced::widget::{button, column, container, row};
use iced::{Alignment, Length, Padding};

/// See the [module](self).
pub struct MediaRow<'a, M> {
    title: String,
    subtitle: Option<String>,
    picture: Option<Element<'a, M>>,
    columns: Vec<(Length, Element<'a, M>)>,
    suffix: Vec<Element<'a, M>>,
    compact: bool,
    height: Length,
    on_press: Option<M>,
    metrics: Metrics,
}

impl<'a, M: 'a> MediaRow<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            picture: None,
            columns: Vec::new(),
            suffix: Vec::new(),
            compact: false,
            height: Length::Shrink,
            on_press: None,
            metrics: Metrics::desktop(),
        }
    }
    /// The dimmed line under the title (the artist, the show).
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
    /// The picture at the start (a cover, an avatar). Hidden in compact mode.
    pub fn picture(mut self, picture: impl Into<Element<'a, M>>) -> Self {
        self.picture = Some(picture.into());
        self
    }
    /// An extra cell of `width` after the title: an album, a BPM, a time. Add as many as needed.
    pub fn column(mut self, width: impl Into<Length>, cell: impl Into<Element<'a, M>>) -> Self {
        self.columns.push((width.into(), cell.into()));
        self
    }
    /// A trailing part: a menu button, a toggle. Add as many as needed.
    pub fn suffix(mut self, part: impl Into<Element<'a, M>>) -> Self {
        self.suffix.push(part.into());
        self
    }
    /// No picture, and the subtitle on the title's line.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// The row's height. `Fill` centers the content in the height it is given (the rows of a
    /// [`VirtualList`](super::VirtualList)); the default fits the content.
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
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

impl<'a, M: Clone + 'a> From<MediaRow<'a, M>> for Element<'a, M> {
    fn from(r: MediaRow<'a, M>) -> Self {
        let m = r.metrics;
        let mut line = row![].spacing(m.space_m).align_y(Alignment::Center);
        if r.compact {
            line = line.push(ty::body(r.title).width(Length::FillPortion(3)));
            if let Some(s) = r.subtitle {
                line = line.push(ty::caption(s).style(st::dim).width(Length::FillPortion(2)));
            }
        } else {
            if let Some(p) = r.picture {
                line = line.push(p);
            }
            let mut text = column![ty::body(r.title)];
            if let Some(s) = r.subtitle {
                text = text.push(ty::caption(s).style(st::dim));
            }
            line = line.push(text.width(Length::FillPortion(3)));
        }
        for (width, cell) in r.columns {
            line = line.push(container(cell).width(width));
        }
        for part in r.suffix {
            line = line.push(part);
        }
        // A button does not center its content, so the row is centered in it explicitly.
        let inner = container(line.padding(Padding {
            left: m.space_s,
            right: m.space_m,
            top: if r.height == Length::Shrink { 4.0 } else { 0.0 },
            bottom: if r.height == Length::Shrink { 4.0 } else { 0.0 },
        }))
        .width(Length::Fill)
        .center_y(r.height);
        button(inner)
            .width(Length::Fill)
            .height(r.height)
            .padding(0)
            .style(sb::flat_row(8.0, 0.08, 0.14))
            .on_press_maybe(r.on_press)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_its_parts() {
        let r: MediaRow<'_, ()> = MediaRow::new("Song")
            .subtitle("Artist")
            .column(Length::Fixed(44.0), iced::widget::space())
            .suffix(iced::widget::space())
            .compact(true)
            .height(Length::Fill);
        assert_eq!(r.columns.len(), 1);
        assert_eq!(r.suffix.len(), 1);
        assert!(r.compact && r.height == Length::Fill && r.subtitle.as_deref() == Some("Artist"));
    }
}
