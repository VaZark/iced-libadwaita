//! A single-thumb slider sharing the range control's oriented interaction and rendering.
use super::RangeSlider;
use crate::Element;
use crate::primitives::{range::Scalar, scrub::Axis};
use iced::Length;
use std::ops::RangeInclusive;

pub struct Scale<'a, T, M> {
    slider: RangeSlider<'a, M, T>,
}
impl<'a, T: Scalar + 'a, M: 'a> Scale<'a, T, M> {
    pub fn new(range: RangeInclusive<T>, value: T, on_change: impl Fn(T) -> M + 'a) -> Self {
        let low = if range.start() <= range.end() {
            *range.start()
        } else {
            *range.end()
        };
        Self {
            slider: RangeSlider::new(range, (low, value), move |(_, value)| on_change(value))
                .single(),
        }
    }
    pub fn orientation(mut self, axis: Axis) -> Self {
        self.slider = self.slider.orientation(axis);
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.slider = self.slider.width(width);
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.slider = self.slider.height(height);
        self
    }
    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.slider = self.slider.step(step);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.slider = self.slider.disabled(disabled);
        self
    }
    pub fn on_release(mut self, message: M) -> Self
    where
        M: Clone,
    {
        self.slider = self.slider.on_release(message);
        self
    }
}
impl<'a, T: Scalar + 'a, M: 'a> From<Scale<'a, T, M>> for Element<'a, M> {
    fn from(scale: Scale<'a, T, M>) -> Self {
        scale.slider.into()
    }
}
