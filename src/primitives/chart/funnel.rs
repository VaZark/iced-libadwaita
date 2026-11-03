//! Funnel layout: stages stacked top to bottom, each as wide as its value.

use super::shape::Pt;

/// One stage: a trapezoid that is `top` wide where it starts and `bottom` wide where it ends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stage {
    pub index: usize,
    pub value: f32,
    pub y0: f32,
    pub y1: f32,
    /// `(left, right)` at `y0`.
    pub top: (f32, f32),
    /// `(left, right)` at `y1`.
    pub bottom: (f32, f32),
    /// Of the first stage, `0..=1`.
    pub of_first: f32,
    /// Of the stage before (1 for the first).
    pub of_previous: f32,
}

impl Stage {
    /// The four corners, clockwise from the top left.
    pub fn corners(&self) -> [Pt; 4] {
        [
            Pt::new(self.top.0, self.y0),
            Pt::new(self.top.1, self.y0),
            Pt::new(self.bottom.1, self.y1),
            Pt::new(self.bottom.0, self.y1),
        ]
    }
    /// Whether `p` is inside the trapezoid.
    pub fn contains(&self, p: Pt) -> bool {
        if p.y < self.y0 || p.y > self.y1 {
            return false;
        }
        let t = if self.y1 > self.y0 {
            (p.y - self.y0) / (self.y1 - self.y0)
        } else {
            0.0
        };
        let l = self.top.0 + (self.bottom.0 - self.top.0) * t;
        let r = self.top.1 + (self.bottom.1 - self.top.1) * t;
        p.x >= l && p.x <= r
    }
}

/// How a stage's far edge is sized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Shape {
    /// Each stage narrows to the width of the next, so the funnel is one outline.
    #[default]
    Continuous,
    /// Each stage is a block as wide as its own value (a centered bar chart).
    Blocks,
}

/// The stages for `values` in a `width` × `height` area, `gap` apart. Widths are proportional
/// to the largest value, centered on `width / 2`.
pub fn funnel(values: &[f32], width: f32, height: f32, gap: f32, shape: Shape) -> Vec<Stage> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    let v = |i: usize| values[i].max(0.0);
    let max = (0..n).map(v).fold(0.0f32, f32::max);
    let half = |x: f32| {
        if max > 0.0 {
            x / max * width / 2.0
        } else {
            0.0
        }
    };
    let mid = width / 2.0;
    let h = ((height - gap * (n as f32 - 1.0)) / n as f32).max(0.0);
    (0..n)
        .map(|i| {
            let (top, bottom) = (
                half(v(i)),
                match shape {
                    Shape::Continuous if i + 1 < n => half(v(i + 1)),
                    _ => half(v(i)),
                },
            );
            let y0 = i as f32 * (h + gap);
            Stage {
                index: i,
                value: v(i),
                y0,
                y1: y0 + h,
                top: (mid - top, mid + top),
                bottom: (mid - bottom, mid + bottom),
                of_first: if v(0) > 0.0 { v(i) / v(0) } else { 0.0 },
                of_previous: if i > 0 && v(i - 1) > 0.0 {
                    v(i) / v(i - 1)
                } else {
                    1.0
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_narrow_with_the_values() {
        let s = funnel(&[100.0, 50.0, 25.0], 200.0, 90.0, 0.0, Shape::Continuous);
        assert_eq!(s[0].top, (0.0, 200.0));
        assert_eq!(s[0].bottom, (50.0, 150.0));
        assert_eq!(s[1].top, (50.0, 150.0));
        assert_eq!(s[2].bottom, s[2].top);
        assert_eq!(s[1].y0, 30.0);
        assert_eq!(s[2].of_first, 0.25);
        assert_eq!(s[2].of_previous, 0.5);
    }

    #[test]
    fn blocks_keep_their_own_width() {
        let s = funnel(&[10.0, 5.0], 100.0, 20.0, 0.0, Shape::Blocks);
        assert_eq!(s[0].bottom, s[0].top);
    }

    #[test]
    fn gaps_come_out_of_the_height() {
        let s = funnel(&[1.0, 1.0], 10.0, 21.0, 1.0, Shape::Blocks);
        assert_eq!((s[0].y1, s[1].y0, s[1].y1), (10.0, 11.0, 21.0));
    }

    #[test]
    fn it_finds_the_stage_under_a_point() {
        let s = funnel(&[100.0, 50.0], 200.0, 20.0, 0.0, Shape::Continuous);
        assert!(s[0].contains(Pt::new(100.0, 5.0)));
        assert!(!s[0].contains(Pt::new(5.0, 9.0)));
        assert!(s[1].contains(Pt::new(100.0, 15.0)));
    }

    #[test]
    fn empty_and_zero_are_safe() {
        assert!(funnel(&[], 10.0, 10.0, 0.0, Shape::Blocks).is_empty());
        let s = funnel(&[0.0, 0.0], 10.0, 10.0, 0.0, Shape::Continuous);
        assert_eq!(s[0].of_first, 0.0);
    }
}
