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
    pub fn slider_props(mut self, props: super::range_slider::RangeSliderProps) -> Self {
        self.slider = props.apply(self.slider);
        self
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

/// Presentation-only overrides supported by [`Scale`]. Values and events stay parent-owned.
#[derive(Clone, Default)]
pub struct ScaleProps {
    pub width: Option<iced::Length>,
    pub height: Option<iced::Length>,
    pub orientation: Option<Axis>,
    pub slider_props: Option<super::range_slider::RangeSliderProps>,
}
impl ScaleProps {
    pub fn width(mut self, value: impl Into<iced::Length>) -> Self {
        self.width = Some(value.into());
        self
    }
    pub fn height(mut self, value: impl Into<iced::Length>) -> Self {
        self.height = Some(value.into());
        self
    }
    pub fn orientation(mut self, value: impl Into<Axis>) -> Self {
        self.orientation = Some(value.into());
        self
    }
    pub fn slider_props(mut self, value: impl Into<super::range_slider::RangeSliderProps>) -> Self {
        self.slider_props = Some(value.into());
        self
    }
    pub fn apply<'a, T: Scalar + 'a, M: 'a>(&self, mut widget: Scale<'a, T, M>) -> Scale<'a, T, M> {
        if let Some(value) = self.width.clone() {
            widget = widget.width(value);
        }
        if let Some(value) = self.height.clone() {
            widget = widget.height(value);
        }
        if let Some(value) = self.orientation.clone() {
            widget = widget.orientation(value);
        }
        if let Some(value) = self.slider_props.clone() {
            widget = widget.slider_props(value);
        }
        widget
    }
}
