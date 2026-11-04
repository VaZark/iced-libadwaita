//! `HistogramRangeSlider`: a histogram over a range slider that edit **one** range. Pressing a
//! bar picks that bin's range, dragging a thumb moves the ends, and the bars inside the range are
//! accented.
//!
//! It is a [`ComposedComponent`](crate::metas::ComposedComponent): two parts (a minimal
//! [`BarChart::histogram`](crate::widgets::BarChart::histogram) and a [`range_slider`](crate::widgets::range_slider)),
//! one shared range, one `on_change`, and one rule (`lo <= hi`). The bounds of the slider are the
//! histogram's own, so there is nothing to keep in step. The gallery's Composed component page
//! shows how it is defined.
//!
//! ```ignore
//! HistogramRangeSlider::new(&self.bpm, self.range, Msg::Range).step(1.0)
//! ```

use crate::Element;
use crate::metas::ComposedComponent;
use crate::primitives::histogram::Histogram;
use crate::widgets::{BarChart, range_slider};
use std::borrow::Cow;
use std::rc::Rc;

/// See the [module](self).
pub struct HistogramRangeSlider<'a, M> {
    histogram: Cow<'a, Histogram>,
    range: (f32, f32),
    on_change: Box<dyn Fn((f32, f32)) -> M + 'a>,
    step: Option<f32>,
    height: f32,
    full: bool,
}

impl<'a, M: 'a> HistogramRangeSlider<'a, M> {
    /// `histogram` is borrowed (`&hist`) or owned (`Histogram::from_counts(..)` built in the view,
    /// for one that depends on a filter). `range` is the chosen `(lo, hi)` within its `min..max`;
    /// `on_change` gets the next range from whichever part the user touched.
    pub fn new(
        histogram: impl Into<Cow<'a, Histogram>>,
        range: (f32, f32),
        on_change: impl Fn((f32, f32)) -> M + 'a,
    ) -> Self {
        Self {
            histogram: histogram.into(),
            range,
            on_change: Box::new(on_change),
            step: None,
            height: 48.0,
            full: false,
        }
    }
    /// How far a thumb moves at a time, and the least the thumbs may be apart. Without one the
    /// thumbs move freely.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step.max(f32::EPSILON));
        self
    }
    /// The height of the bars (default 48).
    pub fn height(mut self, px: f32) -> Self {
        self.height = px;
        self
    }
    /// With axes, gridlines, tooltip and hover on the bars, in the series color (default bare).
    pub fn full(mut self, full: bool) -> Self {
        self.full = full;
        self
    }
}

impl<'a, M: 'a> From<HistogramRangeSlider<'a, M>> for Element<'a, M> {
    fn from(c: HistogramRangeSlider<'a, M>) -> Self {
        let (full, step, height) = (c.full, c.step, c.height);
        let h: Rc<Histogram> = Rc::new(c.histogram.into_owned());
        let (min, max) = (h.min(), h.max());
        ComposedComponent::new(c.range, c.on_change)
            // Part 1: the bars. Those inside the range are accented; pressing one picks its range.
            .part({
                let h = h.clone();
                move |r| {
                    let r = *r;
                    let (sel, pick) = (h.clone(), h.clone());
                    let bars = BarChart::histogram(&h)
                        .height(if full { height.max(140.0) } else { height })
                        .selected(move |i| sel.selected(i, r))
                        .on_pick(move |i| pick.bin_range(i));
                    if full { bars } else { bars.minimal().gap(0.07) }.into()
                }
            })
            // Part 2: the thumbs, for the exact ends.
            .part(move |r| {
                let mut s = range_slider(min..=max, *r, |moved| moved);
                if let Some(step) = step {
                    s = s.step(step).min_gap(step);
                }
                s.into()
            })
            // The rule, once, whichever part sent the value: lo <= hi.
            .normalize(|(lo, hi)| (lo.min(hi), lo.max(hi)))
            .spacing(6.0)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_what_it_is_given() {
        let h = Histogram::from_counts(60.0, 180.0, [1, 3, 2]);
        let s: HistogramRangeSlider<'_, ()> = HistogramRangeSlider::new(&h, (80.0, 120.0), |_| ())
            .step(1.0)
            .height(32.0);
        assert_eq!(
            (s.range, s.step, s.height),
            ((80.0, 120.0), Some(1.0), 32.0)
        );
        assert!(!s.full);
    }
}
