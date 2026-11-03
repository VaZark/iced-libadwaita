//! Shape generators: points in, path commands out (d3-shape's `line` and `area`).

/// A point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

impl Pt {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// One drawing command. Angles of [`Cmd::Arc`] are radians in the drawing plane (0 to the
/// right, increasing clockwise on a screen whose y grows downwards).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cmd {
    Move(Pt),
    Line(Pt),
    /// Two control points, then the end.
    Cubic(Pt, Pt, Pt),
    Arc {
        center: Pt,
        radius: f32,
        start: f32,
        end: f32,
    },
    Close,
}

/// How points are joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Curve {
    /// Straight segments.
    #[default]
    Linear,
    /// Horizontal, a vertical step half way, horizontal.
    Step,
    /// A smooth curve that never overshoots the data (d3's `curveMonotoneX`). Needs points
    /// ordered by x; otherwise it falls back to straight segments.
    Monotone,
}

/// The commands for a line through `points` (a `Move`, then segments). Empty in, empty out.
pub fn line(points: &[Pt], curve: Curve) -> Vec<Cmd> {
    let Some((first, rest)) = points.split_first() else {
        return Vec::new();
    };
    let mut out = vec![Cmd::Move(*first)];
    segments(points, rest, curve, &mut out);
    out
}

/// A closed area between `top` and `bottom` (the same x, in the same order). `bottom` is
/// walked backwards, so the outline is one closed loop.
pub fn area(top: &[Pt], bottom: &[Pt], curve: Curve) -> Vec<Cmd> {
    let mut out = line(top, curve);
    if out.is_empty() {
        return out;
    }
    let back: Vec<Pt> = bottom.iter().rev().copied().collect();
    if let Some(first) = back.first() {
        out.push(Cmd::Line(*first));
        segments(&back, &back[1..], curve, &mut out);
    }
    out.push(Cmd::Close);
    out
}

fn segments(all: &[Pt], rest: &[Pt], curve: Curve, out: &mut Vec<Cmd>) {
    match curve {
        Curve::Linear => out.extend(rest.iter().map(|p| Cmd::Line(*p))),
        Curve::Step => {
            let mut prev = all[0];
            for p in rest {
                let mid = (prev.x + p.x) / 2.0;
                out.push(Cmd::Line(Pt::new(mid, prev.y)));
                out.push(Cmd::Line(Pt::new(mid, p.y)));
                out.push(Cmd::Line(*p));
                prev = *p;
            }
        }
        Curve::Monotone => {
            let Some(t) = tangents(all) else {
                out.extend(rest.iter().map(|p| Cmd::Line(*p)));
                return;
            };
            for i in 0..all.len().saturating_sub(1) {
                let (a, b) = (all[i], all[i + 1]);
                let dx = (b.x - a.x) / 3.0;
                out.push(Cmd::Cubic(
                    Pt::new(a.x + dx, a.y + dx * t[i]),
                    Pt::new(b.x - dx, b.y - dx * t[i + 1]),
                    b,
                ));
            }
        }
    }
}

/// Slopes at each point that keep the curve monotone between points (Steffen's method).
/// `None` when x does not strictly move in one direction.
fn tangents(p: &[Pt]) -> Option<Vec<f32>> {
    let n = p.len();
    if n < 3 {
        return if n == 2 && p[1].x != p[0].x {
            let s = (p[1].y - p[0].y) / (p[1].x - p[0].x);
            Some(vec![s, s])
        } else {
            None
        };
    }
    let dir = (p[1].x - p[0].x).signum();
    if (0..n - 1).any(|i| (p[i + 1].x - p[i].x) * dir <= 0.0) {
        return None;
    }
    let h: Vec<f32> = (0..n - 1).map(|i| p[i + 1].x - p[i].x).collect();
    let s: Vec<f32> = (0..n - 1).map(|i| (p[i + 1].y - p[i].y) / h[i]).collect();
    let mut t = vec![0.0; n];
    for i in 1..n - 1 {
        let pp = (s[i - 1] * h[i] + s[i] * h[i - 1]) / (h[i - 1] + h[i]);
        t[i] = (s[i - 1].signum() + s[i].signum())
            * s[i - 1].abs().min(s[i].abs()).min(0.5 * pp.abs());
        if s[i - 1] == 0.0 || s[i] == 0.0 {
            t[i] = 0.0;
        }
    }
    t[0] = (3.0 * s[0] - t[1]) / 2.0;
    t[n - 1] = (3.0 * s[n - 2] - t[n - 2]) / 2.0;
    Some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(v: &[(f32, f32)]) -> Vec<Pt> {
        v.iter().map(|(x, y)| Pt::new(*x, *y)).collect()
    }

    #[test]
    fn a_line_moves_then_joins() {
        let l = line(&pts(&[(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)]), Curve::Linear);
        assert_eq!(l.len(), 3);
        assert!(matches!(l[0], Cmd::Move(_)));
        assert!(line(&[], Curve::Linear).is_empty());
    }

    #[test]
    fn a_step_turns_half_way() {
        let l = line(&pts(&[(0.0, 0.0), (2.0, 4.0)]), Curve::Step);
        assert_eq!(l[1], Cmd::Line(Pt::new(1.0, 0.0)));
        assert_eq!(l[2], Cmd::Line(Pt::new(1.0, 4.0)));
    }

    #[test]
    fn a_monotone_curve_does_not_overshoot() {
        let p = pts(&[(0.0, 0.0), (1.0, 10.0), (2.0, 10.0), (3.0, 20.0)]);
        for c in line(&p, Curve::Monotone) {
            if let Cmd::Cubic(a, b, _) = c {
                assert!((0.0..=20.0).contains(&a.y) && (0.0..=20.0).contains(&b.y));
            }
        }
        // The flat stretch stays flat.
        let l = line(&p, Curve::Monotone);
        let Cmd::Cubic(a, b, _) = l[2] else { panic!() };
        assert_eq!((a.y, b.y), (10.0, 10.0));
    }

    #[test]
    fn unordered_x_falls_back_to_lines() {
        let l = line(&pts(&[(0.0, 0.0), (2.0, 1.0), (1.0, 2.0)]), Curve::Monotone);
        assert!(l.iter().all(|c| !matches!(c, Cmd::Cubic(..))));
    }

    #[test]
    fn an_area_is_one_closed_loop() {
        let top = pts(&[(0.0, 1.0), (1.0, 2.0)]);
        let bottom = pts(&[(0.0, 0.0), (1.0, 0.0)]);
        let a = area(&top, &bottom, Curve::Linear);
        assert_eq!(*a.last().unwrap(), Cmd::Close);
        assert_eq!(a[2], Cmd::Line(Pt::new(1.0, 0.0)));
        assert_eq!(a[3], Cmd::Line(Pt::new(0.0, 0.0)));
    }
}
