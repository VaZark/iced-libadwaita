//! Path commands ([`Cmd`]) as a canvas [`Path`].

use crate::primitives::chart::shape::{Cmd, Pt};
use iced::Point;
use iced::widget::canvas::{self, Path};

fn point(p: Pt) -> Point {
    Point::new(p.x, p.y)
}

/// The canvas path for `cmds`.
pub fn to_path(cmds: &[Cmd]) -> Path {
    Path::new(|b| {
        for c in cmds {
            match *c {
                Cmd::Move(p) => b.move_to(point(p)),
                Cmd::Line(p) => b.line_to(point(p)),
                Cmd::Cubic(a, c, e) => b.bezier_curve_to(point(a), point(c), point(e)),
                Cmd::Arc {
                    center,
                    radius,
                    start,
                    end,
                } => b.arc(canvas::path::Arc {
                    center: point(center),
                    radius,
                    start_angle: iced::Radians(start),
                    end_angle: iced::Radians(end),
                }),
                Cmd::Close => b.close(),
            }
        }
    })
}
