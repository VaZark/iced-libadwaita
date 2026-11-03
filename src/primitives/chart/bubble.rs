//! Bubble layout: points with a third value, the size, drawn as circles whose **area** follows
//! it (so a bubble twice the value looks twice as big, not four times).

use super::scale::Linear;
use super::shape::Pt;

/// One bubble's data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bubble {
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

impl Bubble {
    pub fn new(x: f32, y: f32, size: f32) -> Self {
        Self { x, y, size }
    }
}

/// A bubble placed on the canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// Which series, and which point of it.
    pub series: usize,
    pub index: usize,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
}

/// The radius for `size` in `domain`, between `r.0` and `r.1`, with area proportional to the
/// size: the smallest bubble is `r.0`, the largest `r.1`. A zero-width domain gives the middle.
pub fn radius(size: f32, domain: (f32, f32), r: (f32, f32)) -> f32 {
    let span = domain.1 - domain.0;
    if span <= 0.0 {
        return (r.0 + r.1) / 2.0;
    }
    let t = ((size - domain.0) / span).clamp(0.0, 1.0);
    // Interpolate the area, not the radius.
    (r.0 * r.0 + (r.1 * r.1 - r.0 * r.0) * t).sqrt()
}

/// The smallest and largest size of all the bubbles (`(0, 1)` when there are none).
pub fn size_extent(series: &[Vec<Bubble>]) -> (f32, f32) {
    let mut it = series
        .iter()
        .flatten()
        .map(|b| b.size)
        .filter(|s| s.is_finite());
    match it.next() {
        Some(first) => it.fold((first, first), |(lo, hi), s| (lo.min(s), hi.max(s))),
        None => (0.0, 1.0),
    }
}

/// The x and y extent of all the bubbles as `((x0, x1), (y0, y1))`, `None` when there are none.
pub fn extent(series: &[Vec<Bubble>]) -> Option<((f32, f32), (f32, f32))> {
    let mut it = series
        .iter()
        .flatten()
        .filter(|b| b.x.is_finite() && b.y.is_finite());
    let first = it.next()?;
    let init = ((first.x, first.x), (first.y, first.y));
    Some(it.fold(init, |((x0, x1), (y0, y1)), b| {
        ((x0.min(b.x), x1.max(b.x)), (y0.min(b.y), y1.max(b.y)))
    }))
}

/// Every bubble on the canvas through the `x` and `y` scales, radii between `radii.0` and
/// `radii.1`. Bubbles with a non-finite value are left out.
pub fn place(series: &[Vec<Bubble>], x: &Linear, y: &Linear, radii: (f32, f32)) -> Vec<Placed> {
    let domain = size_extent(series);
    let mut out = Vec::new();
    for (s, bubbles) in series.iter().enumerate() {
        for (i, b) in bubbles.iter().enumerate() {
            if !(b.x.is_finite() && b.y.is_finite() && b.size.is_finite()) {
                continue;
            }
            out.push(Placed {
                series: s,
                index: i,
                cx: x.map(b.x),
                cy: y.map(b.y),
                r: radius(b.size, domain, radii),
            });
        }
    }
    out
}

/// The bubble under `p`; where several overlap, the smallest, since it is drawn on top.
pub fn hit(placed: &[Placed], p: Pt) -> Option<usize> {
    placed
        .iter()
        .enumerate()
        .filter(|(_, b)| (p.x - b.cx).hypot(p.y - b.cy) <= b.r)
        .min_by(|(_, a), (_, b)| a.r.total_cmp(&b.r))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_follows_the_size() {
        let r = |s| radius(s, (0.0, 100.0), (0.0, 20.0));
        assert_eq!(r(0.0), 0.0);
        assert!((r(100.0) - 20.0).abs() < 1e-4);
        // A quarter of the size is a quarter of the area, so half the radius.
        assert!((r(25.0) - 10.0).abs() < 1e-4);
        assert_eq!(radius(5.0, (3.0, 3.0), (4.0, 8.0)), 6.0);
        assert_eq!(radius(500.0, (0.0, 100.0), (4.0, 8.0)), 8.0);
    }

    #[test]
    fn extents_cover_every_bubble() {
        let s = vec![
            vec![Bubble::new(1.0, 10.0, 5.0), Bubble::new(4.0, 2.0, 9.0)],
            vec![Bubble::new(-2.0, 7.0, 1.0)],
        ];
        assert_eq!(size_extent(&s), (1.0, 9.0));
        assert_eq!(extent(&s), Some(((-2.0, 4.0), (2.0, 10.0))));
        assert_eq!(extent(&[]), None);
        assert_eq!(size_extent(&[]), (0.0, 1.0));
    }

    #[test]
    fn bubbles_are_placed_through_the_scales() {
        let x = Linear::new((0.0, 10.0), (0.0, 100.0));
        let y = Linear::new((0.0, 10.0), (100.0, 0.0));
        let s = vec![vec![
            Bubble::new(5.0, 5.0, 1.0),
            Bubble::new(f32::NAN, 1.0, 1.0),
        ]];
        let p = place(&s, &x, &y, (4.0, 16.0));
        assert_eq!(p.len(), 1);
        assert_eq!((p[0].cx, p[0].cy), (50.0, 50.0));
    }

    #[test]
    fn the_smallest_bubble_under_the_pointer_wins() {
        let placed = vec![
            Placed {
                series: 0,
                index: 0,
                cx: 50.0,
                cy: 50.0,
                r: 30.0,
            },
            Placed {
                series: 0,
                index: 1,
                cx: 55.0,
                cy: 50.0,
                r: 8.0,
            },
        ];
        assert_eq!(hit(&placed, Pt::new(56.0, 50.0)), Some(1));
        assert_eq!(hit(&placed, Pt::new(30.0, 50.0)), Some(0));
        assert_eq!(hit(&placed, Pt::new(0.0, 0.0)), None);
    }
}
