//! Resizing by a handle, like a textarea's grip: the rules that turn a drag into a size.
//!
//! Pure and controlled: the app owns the `Size`; a widget turns pointer movement into
//! [`drag`] calls and reports the result. Used to try a responsive component at any width and
//! height (`MetaComponent`, an app shell, a table) without resizing the window.
//!
//! * [`Grip`]: which edge or corner is dragged, and so which way a movement grows the size (the
//!   left and top edges grow the opposite way to the pointer);
//! * [`Constraints`]: the axes that may change, `min` / `max`, and an optional snap (a grid, or a
//!   breakpoint list: [`Constraints::snap_to`]);
//! * [`drag`] and [`step`] (the keyboard).

use iced::{Size, Vector};

/// The edge or corner of the frame that is dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Grip {
    /// The bottom-right corner: both axes, like a textarea.
    #[default]
    BottomRight,
    /// The right edge: width only.
    Right,
    /// The bottom edge: height only.
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Top,
    TopRight,
}

impl Grip {
    /// The sign a pointer movement has on (width, height): `1` grows with the pointer, `-1`
    /// against it, `0` not at all.
    pub fn signs(self) -> (f32, f32) {
        match self {
            Grip::BottomRight => (1.0, 1.0),
            Grip::Right => (1.0, 0.0),
            Grip::Bottom => (0.0, 1.0),
            Grip::BottomLeft => (-1.0, 1.0),
            Grip::Left => (-1.0, 0.0),
            Grip::TopLeft => (-1.0, -1.0),
            Grip::Top => (0.0, -1.0),
            Grip::TopRight => (1.0, -1.0),
        }
    }
    /// Whether dragging it changes the width, the height.
    pub fn axes(self) -> (bool, bool) {
        let (x, y) = self.signs();
        (x != 0.0, y != 0.0)
    }
}

/// What may change, and how far.
#[derive(Debug, Clone, PartialEq)]
pub struct Constraints {
    pub min: Size,
    pub max: Size,
    /// Round each axis to a multiple of this (a grid). `None`: any size.
    pub step: Option<f32>,
    /// Sizes an axis snaps to when within `snap_radius` of one (for the width: the breakpoints,
    /// so the edges of each width class are easy to hit and to stay on).
    pub snap_points: Vec<f32>,
    pub snap_radius: f32,
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            min: Size::new(120.0, 80.0),
            max: Size::new(f32::INFINITY, f32::INFINITY),
            step: None,
            snap_points: Vec::new(),
            snap_radius: 8.0,
        }
    }
}

impl Constraints {
    pub fn new(min: Size, max: Size) -> Self {
        Self {
            min,
            max,
            ..Self::default()
        }
    }
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step.max(1.0));
        self
    }
    /// Snap an axis to these values when within `radius` of one.
    pub fn snap_to(mut self, points: impl IntoIterator<Item = f32>, radius: f32) -> Self {
        self.snap_points = points.into_iter().collect();
        self.snap_radius = radius;
        self
    }

    /// `v` held within `min..=max`, snapped to the grid and to the snap points.
    fn fit(&self, v: f32, min: f32, max: f32, snap_points: bool) -> f32 {
        let mut v = v;
        if let Some(step) = self.step {
            v = (v / step).round() * step;
        }
        if snap_points
            && let Some(p) = self
                .snap_points
                .iter()
                .copied()
                .filter(|p| (p - v).abs() <= self.snap_radius)
                .min_by(|a, b| (a - v).abs().total_cmp(&(b - v).abs()))
        {
            v = p;
        }
        v.max(min).min(max.max(min))
    }

    /// `size` with each axis in `min..=max` (and on the grid and snap points). The snap points
    /// apply to the width only.
    pub fn clamp(&self, size: Size) -> Size {
        Size::new(
            self.fit(size.width, self.min.width, self.max.width, true),
            self.fit(size.height, self.min.height, self.max.height, false),
        )
    }
}

/// The size after the pointer moved by `delta` since the drag began at `start`, dragging `grip`.
/// An axis the grip does not change stays at `start`.
pub fn drag(start: Size, delta: Vector, grip: Grip, c: &Constraints) -> Size {
    let (sx, sy) = grip.signs();
    let want = Size::new(start.width + sx * delta.x, start.height + sy * delta.y);
    let fitted = c.clamp(want);
    Size::new(
        if sx != 0.0 { fitted.width } else { start.width },
        if sy != 0.0 {
            fitted.height
        } else {
            start.height
        },
    )
}

/// A key press on a focused handle: `dx` / `dy` in steps (-1, 0, 1) of `amount` pixels (the
/// grid step if there is one, else `amount`).
pub fn step(size: Size, dx: i32, dy: i32, amount: f32, grip: Grip, c: &Constraints) -> Size {
    let unit = c.step.unwrap_or(amount);
    let (sx, sy) = grip.signs();
    let want = Size::new(
        size.width + sx.abs() * dx as f32 * unit,
        size.height + sy.abs() * dy as f32 * unit,
    );
    c.clamp(want)
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: Size = Size::new(400.0, 300.0);

    fn c() -> Constraints {
        Constraints::new(Size::new(100.0, 80.0), Size::new(800.0, 600.0))
    }

    #[test]
    fn the_corner_grows_both_axes_with_the_pointer() {
        let s = drag(START, Vector::new(50.0, -20.0), Grip::BottomRight, &c());
        assert_eq!(s, Size::new(450.0, 280.0));
    }

    #[test]
    fn an_edge_changes_one_axis_only() {
        let s = drag(START, Vector::new(50.0, 70.0), Grip::Right, &c());
        assert_eq!(s, Size::new(450.0, 300.0));
        let s = drag(START, Vector::new(50.0, 70.0), Grip::Bottom, &c());
        assert_eq!(s, Size::new(400.0, 370.0));
    }

    #[test]
    fn the_left_and_top_edges_grow_against_the_pointer() {
        let s = drag(START, Vector::new(-30.0, 0.0), Grip::Left, &c());
        assert_eq!(s.width, 430.0);
        let s = drag(START, Vector::new(0.0, 40.0), Grip::Top, &c());
        assert_eq!(s.height, 260.0);
        let s = drag(START, Vector::new(-10.0, 10.0), Grip::TopLeft, &c());
        assert_eq!(s, Size::new(410.0, 290.0));
    }

    #[test]
    fn min_and_max_hold() {
        let s = drag(START, Vector::new(-999.0, -999.0), Grip::BottomRight, &c());
        assert_eq!(s, Size::new(100.0, 80.0));
        let s = drag(START, Vector::new(999.0, 999.0), Grip::BottomRight, &c());
        assert_eq!(s, Size::new(800.0, 600.0));
    }

    #[test]
    fn a_grid_rounds_each_axis() {
        let c = c().step(20.0);
        let s = drag(START, Vector::new(13.0, 27.0), Grip::BottomRight, &c);
        assert_eq!(s, Size::new(420.0, 320.0));
    }

    #[test]
    fn the_width_snaps_to_breakpoints_within_the_radius() {
        let c = c().snap_to([576.0, 768.0], 8.0);
        // 400 + 172 = 572: within 8 of 576.
        let s = drag(START, Vector::new(172.0, 0.0), Grip::BottomRight, &c);
        assert_eq!(s.width, 576.0);
        // 600 is not near any: unchanged.
        let s = drag(START, Vector::new(200.0, 0.0), Grip::BottomRight, &c);
        assert_eq!(s.width, 600.0);
        // The height never snaps to them: 573 is within 8 of 576 and stays 573.
        let s = drag(
            Size::new(400.0, 570.0),
            Vector::new(0.0, 3.0),
            Grip::BottomRight,
            &c,
        );
        assert_eq!(s.height, 573.0);
    }

    #[test]
    fn keys_step_the_size() {
        let s = step(START, 1, 0, 10.0, Grip::BottomRight, &c());
        assert_eq!(s, Size::new(410.0, 300.0));
        let s = step(START, 0, -1, 10.0, Grip::BottomRight, &c());
        assert_eq!(s, Size::new(400.0, 290.0));
        // An edge handle ignores the other axis.
        assert_eq!(step(START, 0, 1, 10.0, Grip::Right, &c()), START);
        // The grid step wins over the amount.
        assert_eq!(
            step(START, 1, 0, 10.0, Grip::Right, &c().step(25.0)).width,
            425.0
        );
    }
}
