//! Spectrum bars: a row of rounded bars that rise and fall with levels the app supplies, mirrored
//! around the middle (the look of a `cava` visualizer). It draws what it is given; the app
//! reads the levels (0 to 1, low pitch to high) and redraws as often as they change.

use crate::Element;
use crate::Theme;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Frame, Geometry, Path};
use iced::{Length, Point, Rectangle, Renderer, Size};

/// The least a bar shows (a bar at level 0 is a dot, so the row never disappears).
const MIN_BAR: f32 = 3.0;
const GAP: f32 = 3.0;

#[derive(Debug, Clone, Copy)]
pub struct SpectrumBars<'a> {
    levels: &'a [f32],
    height: f32,
}

/// Bars for `levels`, `height` tall; the width is whatever the row is given.
pub fn spectrum_bars(levels: &[f32], height: f32) -> SpectrumBars<'_> {
    SpectrumBars { levels, height }
}

impl<M> canvas::Program<M, Theme> for SpectrumBars<'_> {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let n = self.levels.len();
        if n == 0 {
            return vec![frame.into_geometry()];
        }
        let bar = ((bounds.width - GAP * (n - 1) as f32) / n as f32).max(1.0);
        let mid = bounds.height / 2.0;
        let color = theme.palette.accent_bg_color;
        for (i, level) in self.levels.iter().enumerate() {
            let h = (level.clamp(0.0, 1.0) * bounds.height).max(MIN_BAR);
            let x = i as f32 * (bar + GAP);
            let path = Path::rounded_rectangle(
                Point::new(x, mid - h / 2.0),
                Size::new(bar, h),
                (bar / 2.0).into(),
            );
            frame.fill(&path, color);
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<SpectrumBars<'a>> for Element<'a, M> {
    fn from(s: SpectrumBars<'a>) -> Self {
        let height = s.height;
        iced::widget::canvas(s)
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_for_any_levels() {
        let _: Element<'_, ()> = spectrum_bars(&[], 24.0).into();
        let _: Element<'_, ()> = spectrum_bars(&[0.0, 0.5, 2.0, -1.0], 24.0).into();
    }
}
