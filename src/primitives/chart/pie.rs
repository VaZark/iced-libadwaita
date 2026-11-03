//! Pie layout: values as angles, annular sectors, hit testing (d3-shape's `pie` and `arc`).
//!
//! Angles are radians measured **clockwise from twelve o'clock**, as d3 does.

use super::shape::{Cmd, Pt};
use std::f32::consts::{FRAC_PI_2, TAU};

/// One slice of a pie.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slice {
    /// Position in the input.
    pub index: usize,
    pub value: f32,
    pub start: f32,
    pub end: f32,
}

impl Slice {
    pub fn middle(&self) -> f32 {
        (self.start + self.end) / 2.0
    }
}

/// The slices for `values` between angles `start` and `end`, with `pad` radians between them.
/// A zero, negative or non-finite value gets a zero-width slice. All zero gives all zero-width.
pub fn pie(values: &[f32], start: f32, end: f32, pad: f32) -> Vec<Slice> {
    let clean = |v: f32| if v.is_finite() && v > 0.0 { v } else { 0.0 };
    let sum: f32 = values.iter().map(|v| clean(*v)).sum();
    let n = values.len();
    let span = end - start;
    let pad = if n > 1 {
        pad.min(span / n as f32).max(0.0)
    } else {
        0.0
    };
    let k = if sum > 0.0 {
        (span - pad * n as f32) / sum
    } else {
        0.0
    };
    let mut a = start;
    values
        .iter()
        .enumerate()
        .map(|(index, v)| {
            let value = clean(*v);
            let s = Slice {
                index,
                value,
                start: a,
                end: a + value * k,
            };
            a = s.end + pad;
            s
        })
        .collect()
}

/// The point at radius `r` and angle `a` (clockwise from the top) around `center`.
pub fn polar(center: Pt, r: f32, a: f32) -> Pt {
    Pt::new(center.x + r * a.sin(), center.y - r * a.cos())
}

/// The outline of the sector between `inner` and `outer` radii and angles `a0..a1`. An `inner`
/// of zero is a wedge. A full turn is a ring or a disc.
pub fn arc(center: Pt, inner: f32, outer: f32, a0: f32, a1: f32) -> Vec<Cmd> {
    let (s, e) = (a0 - FRAC_PI_2, a1 - FRAC_PI_2);
    let mut out = vec![Cmd::Move(polar(center, outer, a0))];
    out.push(Cmd::Arc {
        center,
        radius: outer,
        start: s,
        end: e,
    });
    if inner > 0.0 {
        // Back along the inner edge in short straight steps: a backwards arc is not portable
        // across drawing backends, and at these steps the difference is under a pixel.
        let steps = (((a1 - a0).abs() / 0.05).ceil() as usize).max(1);
        for k in 0..=steps {
            let a = a1 + (a0 - a1) * k as f32 / steps as f32;
            out.push(Cmd::Line(polar(center, inner, a)));
        }
    } else {
        out.push(Cmd::Line(center));
    }
    out.push(Cmd::Close);
    out
}

/// The slice under `p`, if any.
pub fn hit(center: Pt, inner: f32, outer: f32, slices: &[Slice], p: Pt) -> Option<usize> {
    let (dx, dy) = (p.x - center.x, p.y - center.y);
    let r = dx.hypot(dy);
    if r < inner || r > outer {
        return None;
    }
    // Clockwise from the top, in 0..TAU.
    let a = dx.atan2(-dy).rem_euclid(TAU);
    // The layout may start at any angle, so test the angle a turn either way too.
    slices.iter().position(|s| {
        s.value > 0.0
            && [a - TAU, a, a + TAU]
                .iter()
                .any(|a| (s.start..s.end).contains(a))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_share_the_circle() {
        let s = pie(&[1.0, 1.0, 2.0], 0.0, TAU, 0.0);
        assert!((s[0].end - TAU / 4.0).abs() < 1e-5);
        assert!((s[2].end - TAU).abs() < 1e-5);
        assert_eq!(s[1].start, s[0].end);
    }

    #[test]
    fn padding_is_taken_from_the_slices() {
        let s = pie(&[1.0, 1.0], 0.0, TAU, 0.1);
        assert!((s[1].start - s[0].end - 0.1).abs() < 1e-5);
        assert!(s[1].end <= TAU);
    }

    #[test]
    fn bad_values_are_empty_slices() {
        let s = pie(&[2.0, -1.0, f32::NAN, 2.0], 0.0, TAU, 0.0);
        assert_eq!(s[1].start, s[1].end);
        assert_eq!(s[2].value, 0.0);
        assert!(
            pie(&[0.0, 0.0], 0.0, TAU, 0.0)
                .iter()
                .all(|s| s.start == s.end)
        );
    }

    #[test]
    fn polar_starts_at_the_top() {
        let c = Pt::new(10.0, 10.0);
        let top = polar(c, 5.0, 0.0);
        assert!((top.x - 10.0).abs() < 1e-5 && (top.y - 5.0).abs() < 1e-5);
        let right = polar(c, 5.0, FRAC_PI_2);
        assert!((right.x - 15.0).abs() < 1e-5);
    }

    #[test]
    fn a_donut_has_a_hole() {
        let c = Pt::new(0.0, 0.0);
        let s = pie(&[1.0, 1.0], 0.0, TAU, 0.0);
        // Just right of the top is the first slice; just left of it, the second.
        assert_eq!(hit(c, 5.0, 10.0, &s, Pt::new(1.0, -8.0)), Some(0));
        assert_eq!(hit(c, 5.0, 10.0, &s, Pt::new(-1.0, -8.0)), Some(1));
        assert_eq!(hit(c, 5.0, 10.0, &s, Pt::new(0.5, -2.0)), None);
        assert_eq!(hit(c, 5.0, 10.0, &s, Pt::new(0.0, -12.0)), None);
    }

    #[test]
    fn arcs_close() {
        let a = arc(Pt::new(0.0, 0.0), 2.0, 4.0, 0.0, 1.0);
        assert_eq!(*a.last().unwrap(), Cmd::Close);
        // Move, outer arc, then the inner edge back, then Close.
        assert!(a.len() > 5);
        let Cmd::Line(end) = a[a.len() - 2] else {
            panic!()
        };
        let start = polar(Pt::new(0.0, 0.0), 2.0, 0.0);
        assert!((end.x - start.x).abs() < 1e-4 && (end.y - start.y).abs() < 1e-4);
    }
}
