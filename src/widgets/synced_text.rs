//! `SyncedText`: a scrolling list of timed lines with the current one highlighted (lyrics,
//! subtitles, a transcript). Pressing a line asks to seek to its time.
//!
//! The application owns the playback position. The component is controlled: give it the lines
//! and the position, and call [`follow_synced_text`] whenever the current line changes so the
//! list scrolls with it.
//!
//! ```ignore
//! SyncedText::new(LYRICS, &self.lines, Msg::Seek).position(self.pos)
//!
//! // in update, when the position moves to another line:
//! follow_synced_text(LYRICS, synced::current(&self.lines, self.pos), self.lines.len())
//! ```

use crate::Element;
use crate::components::typography as ty;
use crate::primitives::synced::{self, Line};
use crate::styles::text as st;
use iced::widget::{Id, column, container, scrollable};
use iced::{Alignment, Length, Task};

/// See the [module](self).
pub struct SyncedText<'a, M> {
    id: Id,
    lines: &'a [Line],
    current: Option<usize>,
    on_seek: Box<dyn Fn(f32) -> M + 'a>,
    placeholder: String,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    line_padding: Option<iced::Padding>,
    align: Alignment,
}

impl<'a, M> SyncedText<'a, M> {
    /// `id` names the scrollable (for [`follow_synced_text`]); `on_seek` gets the time of the
    /// pressed line.
    pub fn new(id: impl Into<Id>, lines: &'a [Line], on_seek: impl Fn(f32) -> M + 'a) -> Self {
        Self {
            id: id.into(),
            lines,
            current: None,
            on_seek: Box::new(on_seek),
            placeholder: "Nothing to show.".into(),
            spacing: None,
            padding: None,
            line_padding: None,
            align: Alignment::Start,
        }
    }
    /// The highlighted line.
    pub fn current(mut self, current: Option<usize>) -> Self {
        self.current = current;
        self
    }
    /// The playback position in seconds: highlights the line active at it.
    pub fn position(self, pos: f32) -> Self {
        let current = synced::current(self.lines, pos);
        self.current(current)
    }
    /// The gap between lines (default: the theme's `space_xxs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The padding around the lines (default: the theme's `empty_padding_y` by `space_m`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The padding inside each line's button (default: the theme's `space_xs` by `gap`).
    pub fn line_padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.line_padding = Some(p.into());
        self
    }
    /// Where the lines sit across the column (default: [`Alignment::Start`]).
    pub fn align(mut self, a: Alignment) -> Self {
        self.align = a;
        self
    }
    /// What an empty list says.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
}

/// The task that scrolls the [`SyncedText`] named `id` to line `current` of `total`. Run it when
/// the current line changes.
pub fn follow_synced_text<T: Send + 'static>(
    id: impl Into<Id>,
    current: Option<usize>,
    total: usize,
) -> Task<T> {
    iced::widget::operation::snap_to(
        id,
        iced::widget::scrollable::RelativeOffset {
            x: 0.0,
            y: synced::scroll_to(current, total),
        },
    )
}

impl<'a, M: Clone + 'a> From<SyncedText<'a, M>> for Element<'a, M> {
    fn from(s: SyncedText<'a, M>) -> Self {
        if s.lines.is_empty() {
            return container(ty::body(s.placeholder).style(st::dim))
                .center(Length::Fill)
                .into();
        }
        let current = s.current;
        let on_seek = s.on_seek;
        let m = crate::Metrics::desktop();
        let line_padding = s
            .line_padding
            .unwrap_or(iced::Padding::from([m.space_xs, m.gap]));
        let rows = s.lines.iter().enumerate().map(|(i, l)| {
            let t = if Some(i) == current {
                ty::title_3(l.text.as_str())
            } else {
                ty::body(l.text.as_str()).size(18).style(st::dim)
            };
            crate::components::flat_button(container(t).padding(line_padding))
                .on_press(on_seek(l.at))
                .into()
        });
        scrollable(
            column(rows)
                .spacing(s.spacing.unwrap_or(m.space_xxs))
                .padding(
                    s.padding
                        .unwrap_or(iced::Padding::from([m.empty_padding_y, m.space_m])),
                )
                .align_x(s.align)
                .width(Length::Fill),
        )
        .id(s.id)
        .height(Length::Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gaps_and_padding_are_parameters() {
        let lines = [Line::new(0.0, "a"), Line::new(4.0, "b")];
        let s = SyncedText::new("lyrics", &lines, |_| ())
            .spacing(1.0)
            .padding(2.0)
            .line_padding([3.0, 4.0])
            .align(Alignment::Center);
        assert_eq!(s.spacing, Some(1.0));
        let _: Element<'_, ()> = s.into();
    }

    #[test]
    fn the_position_picks_the_current_line() {
        let lines = [Line::new(0.0, "a"), Line::new(4.0, "b")];
        let s = SyncedText::new("lyrics", &lines, |_| ()).position(5.0);
        assert_eq!(s.current, Some(1));
    }
}
