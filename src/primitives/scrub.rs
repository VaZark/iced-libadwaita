//! Shared relative scrubbing and oriented track coordinates for value controls.

/// How large a step of a drag is: a modifier key trades speed for precision, as in a design tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Precision {
    /// One `step`.
    #[default]
    Normal,
    /// Ten steps (Shift).
    Coarse,
    /// One unit of the last digit (Alt).
    Fine,
}

impl Precision {
    /// Shift is coarse and Alt fine; both, or neither, is normal.
    pub fn from_modifiers(shift: bool, alt: bool) -> Self {
        match (shift, alt) {
            (true, false) => Self::Coarse,
            (false, true) => Self::Fine,
            _ => Self::Normal,
        }
    }
}

/// The direction a drag changes a value in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Axis {
    /// Dragging right increases, left decreases.
    Horizontal,
    /// Dragging up increases, down decreases.
    #[default]
    Vertical,
}

/// A drag along an [`Axis`] as steps: `px_per_step` of travel is one step, and nothing counts as
/// a drag until it has travelled `threshold` px along the axis (so a click stays a click).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scrub {
    pub axis: Axis,
    pub px_per_step: f32,
    pub threshold: f32,
}

impl Scrub {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            px_per_step: 4.0,
            threshold: 4.0,
        }
    }
    pub fn px_per_step(mut self, px: f32) -> Self {
        self.px_per_step = px.max(0.5);
        self
    }

    /// Travel from `from` to `to` along the axis: positive in the direction that increases.
    pub fn travel(&self, from: (f32, f32), to: (f32, f32)) -> f32 {
        match self.axis {
            Axis::Horizontal => to.0 - from.0,
            Axis::Vertical => from.1 - to.1,
        }
    }
    /// Whether the pointer has moved far enough along the axis for this to be a drag.
    pub fn started(&self, from: (f32, f32), to: (f32, f32)) -> bool {
        self.travel(from, to).abs() >= self.threshold
    }
    /// Where the drag's anchor goes after `steps` steps were taken from it: moved along the axis
    /// by the travel they used, keeping the part of the pointer's travel that is not yet a step.
    pub fn advance(&self, anchor: (f32, f32), steps: i64) -> (f32, f32) {
        let travel = steps as f32 * self.px_per_step;
        match self.axis {
            Axis::Horizontal => (anchor.0 + travel, anchor.1),
            Axis::Vertical => (anchor.0, anchor.1 - travel),
        }
    }
    /// The steps a drag from `from` to `to` has made: whole steps, towards zero.
    pub fn steps(&self, from: (f32, f32), to: (f32, f32)) -> i64 {
        (self.travel(from, to) / self.px_per_step).trunc() as i64
    }
}

impl Axis {
    /// Pointer coordinate increasing with the value (right or up).
    pub fn coordinate(self, point: (f32, f32)) -> f32 {
        match self {
            Self::Horizontal => point.0,
            Self::Vertical => -point.1,
        }
    }
    /// Position along a track, clamped to its endpoints. A collapsed track stays at zero.
    pub fn fraction(self, point: (f32, f32), origin: (f32, f32), length: f32) -> f32 {
        if !length.is_finite() || length <= 0.0 {
            return 0.0;
        }
        ((self.coordinate(point) - self.coordinate(origin)) / length).clamp(0.0, 1.0)
    }
    /// Point at a fraction along a track, beginning at its low-value end.
    pub fn point(self, origin: (f32, f32), length: f32, fraction: f32) -> (f32, f32) {
        let distance = length.max(0.0) * fraction.clamp(0.0, 1.0);
        match self {
            Self::Horizontal => (origin.0 + distance, origin.1),
            Self::Vertical => (origin.0, origin.1 - distance),
        }
    }
}

/// A relative scrub gesture, independent of the value's numeric representation.
#[derive(Debug, Clone, Copy)]
pub struct Gesture {
    from: (f32, f32),
    anchor: (f32, f32),
    dragging: bool,
}
impl Gesture {
    pub fn new(at: (f32, f32)) -> Self {
        Self {
            from: at,
            anchor: at,
            dragging: false,
        }
    }
    pub fn dragging(&self) -> bool {
        self.dragging
    }
    /// Returns newly consumed whole steps, retaining sub-step travel and reversals.
    pub fn update(&mut self, scrub: Scrub, at: (f32, f32)) -> i64 {
        self.dragging |= scrub.started(self.from, at);
        if !self.dragging {
            return 0;
        }
        let steps = scrub.steps(self.anchor, at);
        self.anchor = scrub.advance(self.anchor, steps);
        steps
    }
}

/// Pointer offset from a thumb's centre, avoiding a jump when grabbing its edge.
#[derive(Debug, Clone, Copy, Default)]
pub struct Grab {
    offset: f32,
}
impl Grab {
    pub fn new(axis: Axis, pointer: (f32, f32), center: (f32, f32)) -> Self {
        Self {
            offset: axis.coordinate(pointer) - axis.coordinate(center),
        }
    }
    pub fn center(self, axis: Axis, pointer: (f32, f32)) -> (f32, f32) {
        match axis {
            Axis::Horizontal => (pointer.0 - self.offset, pointer.1),
            Axis::Vertical => (pointer.0, pointer.1 + self.offset),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gestures_keep_substep_travel_and_ignore_cross_axis_motion() {
        for axis in [Axis::Horizontal, Axis::Vertical] {
            let scrub = Scrub::new(axis).px_per_step(5.0);
            let origin = (100.0, 100.0);
            let at = |distance: f32| axis.point(origin, 100.0, distance / 100.0);
            let mut gesture = Gesture::new(origin);
            let cross = match axis {
                Axis::Horizontal => (100.0, 140.0),
                Axis::Vertical => (140.0, 100.0),
            };
            assert_eq!(gesture.update(scrub, cross), 0);
            assert!(!gesture.dragging());
            assert_eq!(gesture.update(scrub, at(3.0)), 0);
            assert!(!gesture.dragging());
            assert_eq!(gesture.update(scrub, at(12.0)), 2);
            assert!(gesture.dragging());
            assert_eq!(gesture.update(scrub, at(14.0)), 0);
            assert_eq!(gesture.update(scrub, at(15.0)), 1);
            assert_eq!(gesture.update(scrub, origin), -3);
        }
    }
    #[test]
    fn oriented_tracks_roundtrip_clamp_and_preserve_grab_offset() {
        for axis in [Axis::Horizontal, Axis::Vertical] {
            let origin = (20.0, 220.0);
            for fraction in [0.0, 0.25, 0.5, 1.0] {
                let point = axis.point(origin, 200.0, fraction);
                assert_eq!(axis.fraction(point, origin, 200.0), fraction);
            }
            assert_eq!(axis.fraction(origin, origin, 0.0), 0.0);
            let center = axis.point(origin, 200.0, 0.5);
            let pointer = axis.point(origin, 200.0, 0.525);
            let grab = Grab::new(axis, pointer, center);
            assert_eq!(grab.center(axis, pointer), center);
            let moved = axis.point(origin, 200.0, 0.775);
            assert_eq!(axis.fraction(grab.center(axis, moved), origin, 200.0), 0.75);
            assert_eq!(
                axis.fraction(axis.point(origin, 400.0, 1.0), origin, 200.0),
                1.0
            );
        }
    }
}
