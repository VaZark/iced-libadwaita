//! `HistogramRange`: a histogram that doubles as a range picker. Bars inside the selected range
//! are accented and the others dimmed. Pressing a bar selects that bin's range.
//!
//! The distribution shows where values cluster, which two bare thumbs cannot. Put a
//! [`RangeSlider`](super::RangeSlider) under it for the exact thumbs; the pair is the example
//! for [`ComposedComponent`](super::ComposedComponent).

use super::Element;
use crate::Theme;
use crate::primitives::histogram::Histogram;
use iced::widget::{column, container, mouse_area, row, space};
use iced::{Alignment, Background, Border, Length};

/// See the [module](self).
pub struct HistogramRange<'a, M> {
    histogram: &'a Histogram,
    selection: (f32, f32),
    height: f32,
    on_pick: Box<dyn Fn((f32, f32)) -> M + 'a>,
}

impl<'a, M> HistogramRange<'a, M> {
    /// `selection` is the chosen `(lo, hi)`; `on_pick` gets a bin's range when its bar is pressed.
    pub fn new(
        histogram: &'a Histogram,
        selection: (f32, f32),
        on_pick: impl Fn((f32, f32)) -> M + 'a,
    ) -> Self {
        Self {
            histogram,
            selection,
            height: 64.0,
            on_pick: Box::new(on_pick),
        }
    }
    /// The height of the tallest bar.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}

impl<'a, M: Clone + 'a> From<HistogramRange<'a, M>> for Element<'a, M> {
    fn from(h: HistogramRange<'a, M>) -> Self {
        let (hist, height) = (h.histogram, h.height);
        let bars = (0..hist.len()).map(|i| {
            let inside = hist.selected(i, h.selection);
            let bar_h = if hist.counts()[i] == 0 {
                2.0
            } else {
                (hist.fraction(i) * height).max(4.0)
            };
            let bar = container(space::horizontal())
                .width(Length::Fill)
                .height(Length::Fixed(bar_h))
                .style(move |t: &Theme| container::Style {
                    background: Some(Background::Color(if inside {
                        t.palette.accent_bg_color
                    } else {
                        iced::Color {
                            a: 0.22,
                            ..t.semantic.window.fg
                        }
                    })),
                    border: Border::default().rounded(2),
                    ..Default::default()
                });
            mouse_area(
                column![space::vertical().height(Length::Fill), bar]
                    .height(Length::Fixed(height))
                    .align_x(Alignment::Center),
            )
            .on_press((h.on_pick)(hist.bin_range(i)))
            .into()
        });
        row(bars).spacing(2).width(Length::Fill).into()
    }
}
