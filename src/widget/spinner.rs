//! `AdwSpinner`, drawn on a canvas.
//!
//! iced has no implicit animation clock, so the app drives `rotation`
//! (radians) from a subscription, e.g. `iced::window::frames()`.

use super::Element;
use crate::Theme;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::{Length, Point, Radians, Rectangle, Renderer};

#[derive(Debug, Clone, Copy)]
pub struct Spinner {
    pub size: f32,
    pub rotation: f32,
}

pub fn spinner(size: f32, rotation: f32) -> Spinner {
    Spinner { size, rotation }
}

impl<M> canvas::Program<M, Theme> for Spinner {
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
        let c = frame.center();
        let r = self.size / 2.0 - 2.0;
        let w = (self.size / 10.0).max(2.0);
        let fg = theme.semantic.window.fg;
        frame.stroke(
            &Path::circle(c, r),
            Stroke::default()
                .with_width(w)
                .with_color(crate::color::alpha(fg, 0.15)),
        );
        let arc = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: Point::new(c.x, c.y),
                radius: r,
                start_angle: Radians(self.rotation),
                end_angle: Radians(self.rotation + 1.6),
            })
        });
        frame.stroke(
            &arc,
            Stroke::default()
                .with_width(w)
                .with_color(fg)
                .with_line_cap(canvas::LineCap::Round),
        );
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<Spinner> for Element<'a, M> {
    fn from(s: Spinner) -> Self {
        let size = s.size;
        iced::widget::canvas(s)
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into()
    }
}
