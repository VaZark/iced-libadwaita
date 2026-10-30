//! Animations modeled on libadwaita's `AdwTimedAnimation` and `AdwSpringAnimation`.
//!
//! * [`Easing`]: the 35 `AdwEasing` curves (Penner equations plus the four CSS
//!   cubic-beziers added in 1.7).
//! * [`TimedAnimation`]: interpolates `from` → `to` over a duration with an easing,
//!   with repeat, reverse and alternate.
//! * [`SpringParams`] / [`SpringAnimation`]: a damped spring (mass, stiffness,
//!   damping or damping ratio), with initial velocity, epsilon and clamping.
//! * [`Tween`]: the runtime value a widget keeps; retargeting starts a new
//!   animation from the current value (and velocity, for springs).
//! * [`presets`]: the parameters libadwaita itself uses, with their sources.
//!
//! Like libadwaita, animations are skipped when the system disables them
//! (`gtk-enable-animations`); see [`enabled`] / [`set_enabled`].
//!
//! The math is pure and unit-tested; widgets drive it from iced's redraw events.

use std::f64::consts::PI;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};

/// The `AdwEasing` curves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    /// CSS `ease`: `cubic-bezier(0.25, 0.1, 0.25, 1)`. libadwaita's usual default.
    Ease,
    /// CSS `ease-in`: `cubic-bezier(0.42, 0, 1, 1)`.
    EaseIn,
    /// CSS `ease-out`: `cubic-bezier(0, 0, 0.58, 1)`.
    EaseOut,
    /// CSS `ease-in-out`: `cubic-bezier(0.42, 0, 0.58, 1)`.
    EaseInOut,
    /// Any cubic-bezier, e.g. the stylesheet's `$ease-out-quad`
    /// ([`Easing::STYLE_EASE_OUT_QUAD`]).
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// `$ease-out-quad: cubic-bezier(0.25, 0.46, 0.45, 0.94)` from libadwaita's
    /// stylesheet, used by every 200ms widget transition.
    pub const STYLE_EASE_OUT_QUAD: Easing = Easing::CubicBezier(0.25, 0.46, 0.45, 0.94);

    /// Maps linear progress `t` in `[0, 1]` to eased progress (`adw_easing_ease`).
    pub fn ease(self, t: f64) -> f64 {
        use Easing::*;
        let t = t.clamp(0.0, 1.0);
        match self {
            Linear => t,
            EaseInQuad => t * t,
            EaseOutQuad => -t * (t - 2.0),
            EaseInOutQuad => {
                if t < 0.5 { 2.0 * t * t } else { -2.0 * t * t + 4.0 * t - 1.0 }
            }
            EaseInCubic => t * t * t,
            EaseOutCubic => (t - 1.0).powi(3) + 1.0,
            EaseInOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0 }
            }
            EaseInQuart => t.powi(4),
            EaseOutQuart => 1.0 - (t - 1.0).powi(4),
            EaseInOutQuart => {
                if t < 0.5 { 8.0 * t.powi(4) } else { 1.0 - 8.0 * (t - 1.0).powi(4) }
            }
            EaseInQuint => t.powi(5),
            EaseOutQuint => (t - 1.0).powi(5) + 1.0,
            EaseInOutQuint => {
                if t < 0.5 { 16.0 * t.powi(5) } else { 1.0 + 16.0 * (t - 1.0).powi(5) }
            }
            EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            EaseOutSine => (t * PI / 2.0).sin(),
            EaseInOutSine => -((PI * t).cos() - 1.0) / 2.0,
            EaseInExpo => if t == 0.0 { 0.0 } else { 2f64.powf(10.0 * (t - 1.0)) },
            EaseOutExpo => if t == 1.0 { 1.0 } else { 1.0 - 2f64.powf(-10.0 * t) },
            EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2f64.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2f64.powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            EaseOutCirc => (1.0 - (t - 1.0) * (t - 1.0)).sqrt(),
            EaseInOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }
            EaseInElastic => {
                let c4 = 2.0 * PI / 3.0;
                if t == 0.0 { 0.0 } else if t == 1.0 { 1.0 } else { -(2f64.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin() }
            }
            EaseOutElastic => {
                let c4 = 2.0 * PI / 3.0;
                if t == 0.0 { 0.0 } else if t == 1.0 { 1.0 } else { 2f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0 }
            }
            EaseInOutElastic => {
                let c5 = 2.0 * PI / 4.5;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -(2f64.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                } else {
                    2f64.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin() / 2.0 + 1.0
                }
            }
            EaseInBack => {
                let (c1, c3) = (1.70158, 2.70158);
                c3 * t * t * t - c1 * t * t
            }
            EaseOutBack => {
                let (c1, c3) = (1.70158, 2.70158);
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            EaseInOutBack => {
                let c2 = 1.70158 * 1.525;
                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0
                }
            }
            EaseInBounce => 1.0 - bounce_out(1.0 - t),
            EaseOutBounce => bounce_out(t),
            EaseInOutBounce => {
                if t < 0.5 { (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0 } else { (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0 }
            }
            Ease => bezier(0.25, 0.1, 0.25, 1.0, t),
            EaseIn => bezier(0.42, 0.0, 1.0, 1.0, t),
            EaseOut => bezier(0.0, 0.0, 0.58, 1.0, t),
            EaseInOut => bezier(0.42, 0.0, 0.58, 1.0, t),
            CubicBezier(a, b, c, d) => bezier(a as f64, b as f64, c as f64, d as f64, t),
        }
    }
}

fn bounce_out(t: f64) -> f64 {
    let (n1, d1) = (7.5625, 2.75);
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

/// CSS `cubic-bezier(x1, y1, x2, y2)`: solves `x(s) = t` then returns `y(s)`.
fn bezier(x1: f64, y1: f64, x2: f64, y2: f64, t: f64) -> f64 {
    let sample = |a: f64, b: f64, s: f64| 3.0 * a * (1.0 - s).powi(2) * s + 3.0 * b * (1.0 - s) * s * s + s.powi(3);
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if sample(x1, x2, mid) < t { lo = mid } else { hi = mid }
    }
    sample(y1, y2, (lo + hi) / 2.0)
}

/// How many times a [`TimedAnimation`] plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    Times(u32),
    Forever,
}

/// `AdwTimedAnimation`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedAnimation {
    pub from: f64,
    pub to: f64,
    pub duration: Duration,
    pub easing: Easing,
    pub repeat: Repeat,
    /// Run from `to` to `from` (time is mirrored before easing).
    pub reverse: bool,
    /// Flip the direction on every other iteration.
    pub alternate: bool,
}

impl TimedAnimation {
    pub fn new(from: f64, to: f64, duration: Duration, easing: Easing) -> Self {
        Self { from, to, duration, easing, repeat: Repeat::Times(1), reverse: false, alternate: false }
    }
    pub fn repeat(mut self, r: Repeat) -> Self {
        self.repeat = r;
        self
    }
    pub fn reverse(mut self, r: bool) -> Self {
        self.reverse = r;
        self
    }
    pub fn alternate(mut self, a: bool) -> Self {
        self.alternate = a;
        self
    }

    /// Total running time, or `None` when repeating forever.
    pub fn total_duration(&self) -> Option<Duration> {
        match self.repeat {
            Repeat::Times(n) => Some(self.duration * n.max(1)),
            Repeat::Forever => None,
        }
    }

    pub fn is_finished(&self, elapsed: Duration) -> bool {
        self.total_duration().is_some_and(|d| elapsed >= d)
    }

    /// The animated value `elapsed` after the start.
    pub fn value_at(&self, elapsed: Duration) -> f64 {
        let d = self.duration.as_secs_f64();
        if d <= 0.0 {
            return self.end_value();
        }
        let t = elapsed.as_secs_f64() / d;
        let (iteration, local) = match self.repeat {
            Repeat::Times(n) => {
                let n = n.max(1) as f64;
                if t >= n {
                    (n - 1.0, 1.0)
                } else {
                    (t.floor(), t - t.floor())
                }
            }
            Repeat::Forever => (t.floor(), t - t.floor()),
        };
        let flipped = self.reverse ^ (self.alternate && (iteration as u64) % 2 == 1);
        let p = if flipped { 1.0 - local } else { local };
        self.from + (self.to - self.from) * self.easing.ease(p)
    }

    fn end_value(&self) -> f64 {
        if self.reverse { self.from } else { self.to }
    }
}

/// `AdwSpringParams`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringParams {
    pub mass: f64,
    pub stiffness: f64,
    pub damping: f64,
}

impl SpringParams {
    /// From a damping ratio (`adw_spring_params_new`): 0 undamped, <1 underdamped,
    /// 1 critically damped (fastest without oscillating), >1 overdamped.
    pub fn new(damping_ratio: f64, mass: f64, stiffness: f64) -> Self {
        Self { mass, stiffness, damping: damping_ratio * 2.0 * (mass * stiffness).sqrt() }
    }
    /// From raw damping (`adw_spring_params_new_full`).
    pub fn new_full(mass: f64, stiffness: f64, damping: f64) -> Self {
        Self { mass, stiffness, damping }
    }
    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.mass * self.stiffness).sqrt())
    }
}

/// `AdwSpringAnimation`: a physically simulated damped spring from `from` to `to`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringAnimation {
    pub from: f64,
    pub to: f64,
    pub initial_velocity: f64,
    pub params: SpringParams,
    /// Settling threshold (libadwaita default `0.001`).
    pub epsilon: f64,
    /// End as soon as `to` is reached; never overshoot.
    pub clamp: bool,
}

impl SpringAnimation {
    pub fn new(from: f64, to: f64, params: SpringParams) -> Self {
        Self { from, to, initial_velocity: 0.0, params, epsilon: 0.001, clamp: false }
    }
    pub fn initial_velocity(mut self, v: f64) -> Self {
        self.initial_velocity = v;
        self
    }
    pub fn epsilon(mut self, e: f64) -> Self {
        self.epsilon = e;
        self
    }
    pub fn clamp(mut self, c: bool) -> Self {
        self.clamp = c;
        self
    }

    fn raw(&self, t: f64) -> f64 {
        let SpringParams { mass: m, stiffness: k, damping: b } = self.params;
        let beta = b / (2.0 * m);
        let omega0 = (k / m).sqrt();
        let x0 = self.from - self.to;
        let v0 = self.initial_velocity;
        let envelope = (-beta * t).exp();
        let osc = if (beta - omega0).abs() < 1e-9 {
            x0 + (beta * x0 + v0) * t
        } else if beta < omega0 {
            let w = (omega0 * omega0 - beta * beta).sqrt();
            x0 * (w * t).cos() + (beta * x0 + v0) / w * (w * t).sin()
        } else {
            let w = (beta * beta - omega0 * omega0).sqrt();
            x0 * (w * t).cosh() + (beta * x0 + v0) / w * (w * t).sinh()
        };
        self.to + envelope * osc
    }

    /// The value `elapsed` after the start (`adw_spring_animation_calculate_value`).
    pub fn value_at(&self, elapsed: Duration) -> f64 {
        let t = elapsed.as_secs_f64();
        if self.clamp && self.crossed(t) {
            return self.to;
        }
        if self.is_finished(elapsed) { self.to } else { self.raw(t) }
    }

    /// The velocity `elapsed` after the start (`adw_spring_animation_calculate_velocity`).
    pub fn velocity_at(&self, elapsed: Duration) -> f64 {
        let t = elapsed.as_secs_f64();
        if self.is_finished(elapsed) {
            return 0.0;
        }
        let h = 1e-4;
        (self.raw(t + h) - self.raw((t - h).max(0.0))) / (t + h - (t - h).max(0.0))
    }

    fn crossed(&self, t: f64) -> bool {
        // Has the spring passed `to` from `from`'s side?
        (self.raw(t) - self.to) * (self.from - self.to) <= 0.0 && t > 0.0
    }

    /// Estimated running time (`adw_spring_animation_get_estimated_duration`);
    /// springs have no fixed duration. `None` for an undamped spring.
    pub fn estimated_duration(&self) -> Option<Duration> {
        if self.params.damping <= 0.0 {
            return None;
        }
        if (self.from - self.to).abs() < self.epsilon && self.initial_velocity.abs() < self.epsilon {
            return Some(Duration::ZERO);
        }
        // Bound the scan with the decay envelope instead of simulating 30 seconds.
        let SpringParams { mass: m, stiffness: k, damping: b } = self.params;
        let beta = b / (2.0 * m);
        let omega0 = (k / m).sqrt();
        let x0 = (self.from - self.to).abs();
        let w = (omega0 * omega0 - beta * beta).abs().sqrt().max(1e-3);
        let amplitude = x0 + (beta * x0 + self.initial_velocity.abs()) / w + 1.0;
        let decay = if beta > omega0 { (beta - w).max(1e-3) } else { beta };
        let t_max = ((amplitude / self.epsilon).ln() / decay * 1.5 + 0.1).clamp(0.05, 30.0);
        let step = 0.002;
        let mut last_unsettled = 0.0;
        let mut t = 0.0;
        while t < t_max {
            let v = self.raw(t);
            let vel = (self.raw(t + 1e-4) - v) / 1e-4;
            if (v - self.to).abs() > self.epsilon || vel.abs() > self.epsilon {
                last_unsettled = t;
            }
            t += step;
        }
        Some(Duration::from_secs_f64(last_unsettled + step))
    }

    pub fn is_finished(&self, elapsed: Duration) -> bool {
        if self.clamp && self.crossed(elapsed.as_secs_f64()) {
            return true;
        }
        self.estimated_duration().is_some_and(|d| elapsed >= d)
    }
}

/// The parameters libadwaita itself uses (read from its source, `main` branch).
pub mod presets {
    use super::*;

    /// `AdwNavigationView` page transition: critically damped, stiffness 1000.
    pub fn navigation() -> SpringParams {
        SpringParams::new(1.0, 1.0, 1000.0)
    }
    /// `AdwOverlaySplitView` / `AdwCarousel` default: damping ratio 1, mass 0.5, stiffness 500.
    pub fn sidebar() -> SpringParams {
        SpringParams::new(1.0, 0.5, 500.0)
    }
    /// `AdwBottomSheet` (and floating dialogs): damping ratio 0.8, mass 1, stiffness 400;
    /// ratio 1.0 when the user asks for reduced motion. Epsilon `0.0001`.
    pub fn sheet(reduce_motion: bool) -> SpringParams {
        SpringParams::new(if reduce_motion { 1.0 } else { 0.8 }, 1.0, 400.0)
    }
    /// Sheet spring epsilon.
    pub const SHEET_EPSILON: f64 = 0.0001;
    /// Bottom-bar / revealer-style reveal: 250ms, `ADW_EASE` (`REVEAL_BOTTOM_BAR_DURATION`).
    pub const REVEAL: (Duration, Easing) = (Duration::from_millis(250), Easing::Ease);
    /// `AdwToast` show (slide up): 300ms, `ADW_EASE`; 500ms when replacing a toast.
    pub const TOAST_SHOW: (Duration, Easing) = (Duration::from_millis(300), Easing::Ease);
    pub const TOAST_REPLACE: Duration = Duration::from_millis(500);
    /// `AdwToast` hide: 300ms; opacity eased with `ADW_EASE_OUT_CUBIC`, scale 0.95 → 1.
    pub const TOAST_HIDE: (Duration, Easing) = (Duration::from_millis(300), Easing::Ease);
    pub const TOAST_HIDE_SCALE: f64 = 0.05;
    /// Widget state transitions in the stylesheet (buttons, toggles, switch focus,
    /// separators): 200ms, `$ease-out-quad`.
    pub const STYLE_TRANSITION: (Duration, Easing) = (Duration::from_millis(200), Easing::STYLE_EASE_OUT_QUAD);
    /// Scrollbar slider/trough: 200ms linear.
    pub const SCROLLBAR: (Duration, Easing) = (Duration::from_millis(200), Easing::Linear);
}

// ---------------------------------------------------------------------------
// Global enable switch (`gtk-enable-animations`)
// ---------------------------------------------------------------------------

/// 0 = follow the system, 1 = forced on, 2 = forced off.
static OVERRIDE: AtomicU8 = AtomicU8::new(0);

/// Forces animations on/off for the whole process; `None` follows the system again.
pub fn set_enabled(enabled: Option<bool>) {
    OVERRIDE.store(
        match enabled {
            None => 0,
            Some(true) => 1,
            Some(false) => 2,
        },
        Ordering::Relaxed,
    );
}

/// Whether animations play: the process override ([`set_enabled`]), else the desktop config
/// (`[motion] animations`), else the system's `enable-animations` (GNOME setting, default on). Disabled animations jump straight to the end value,
/// like libadwaita's `follow-enable-animations-setting`.
pub fn enabled() -> bool {
    match OVERRIDE.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => match crate::config::current().animations {
            crate::config::Pick::Fixed(on) => on,
            crate::config::Pick::System => crate::system::animations_enabled(),
        },
    }
}

// ---------------------------------------------------------------------------
// Runtime value
// ---------------------------------------------------------------------------

/// How a [`Tween`] moves to a new target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spec {
    Timed(Duration, Easing),
    Spring(SpringParams),
}

impl Spec {
    /// The stylesheet's 200ms `$ease-out-quad` widget transition.
    pub const STYLE: Spec = Spec::Timed(presets::STYLE_TRANSITION.0, presets::STYLE_TRANSITION.1);
}

#[derive(Debug, Clone, Copy)]
enum Running {
    Timed(TimedAnimation),
    /// The spring and its (cached) estimated duration.
    Spring(SpringAnimation, Option<Duration>),
}

/// An animated `f32` kept by a widget. Retargeting starts a new animation from
/// the current value (and, for springs, the current velocity) so motion stays
/// continuous when interrupted.
#[derive(Debug, Clone)]
pub struct Tween {
    value: f64,
    target: f64,
    start: Option<Instant>,
    running: Option<Running>,
}

impl Tween {
    pub fn new(value: f32) -> Self {
        Self { value: value as f64, target: value as f64, start: None, running: None }
    }

    pub fn target(&self) -> f32 {
        self.target as f32
    }

    /// Sets the value immediately (e.g. while the user drags), cancelling any animation.
    pub fn set_immediate(&mut self, value: f32) {
        self.value = value as f64;
        self.target = value as f64;
        self.running = None;
        self.start = None;
    }

    /// Like [`set_target`](Self::set_target), but a spring starts with `velocity` (value units
    /// per second): how a released drag hands its momentum to the animation
    /// (`AdwSpringAnimation:initial-velocity`). Always retargets, even to the current target.
    pub fn fling_to(&mut self, target: f32, velocity: f32, now: Instant, spec: Spec) {
        let current = self.value_at(now);
        self.target = target as f64;
        if !enabled() || (current - target as f64).abs() < 1e-6 && velocity.abs() < 1e-3 {
            self.value = target as f64;
            self.running = None;
            self.start = None;
            return;
        }
        self.value = current;
        self.start = Some(now);
        self.running = Some(match spec {
            Spec::Timed(d, e) => Running::Timed(TimedAnimation::new(current, target as f64, d, e)),
            Spec::Spring(p) => {
                let a = SpringAnimation::new(current, target as f64, p).initial_velocity(velocity as f64);
                Running::Spring(a, a.estimated_duration())
            }
        });
    }

    /// Moves toward `target` using `spec`. Jumps instantly when animations are disabled.
    pub fn set_target(&mut self, target: f32, now: Instant, spec: Spec) {
        let target = target as f64;
        if (target - self.target).abs() < f64::EPSILON && self.running.is_none() {
            return;
        }
        if (target - self.target).abs() < f64::EPSILON {
            return; // already heading there
        }
        let current = self.value_at(now);
        let velocity = match (&self.running, self.start) {
            (Some(Running::Spring(s, _)), Some(st)) => s.velocity_at(now.saturating_duration_since(st)),
            _ => 0.0,
        };
        self.target = target;
        if !enabled() || (current - target).abs() < 1e-6 {
            self.value = target;
            self.running = None;
            self.start = None;
            return;
        }
        self.value = current;
        self.start = Some(now);
        self.running = Some(match spec {
            Spec::Timed(d, e) => Running::Timed(TimedAnimation::new(current, target, d, e)),
            Spec::Spring(p) => {
                let a = SpringAnimation::new(current, target, p).initial_velocity(velocity);
                Running::Spring(a, a.estimated_duration())
            }
        });
    }

    /// The value at `now`.
    pub fn value_at(&self, now: Instant) -> f64 {
        match (&self.running, self.start) {
            (Some(Running::Timed(a)), Some(s)) => a.value_at(now.saturating_duration_since(s)),
            (Some(Running::Spring(a, est)), Some(s)) => {
                let e = now.saturating_duration_since(s);
                if est.is_some_and(|d| e >= d) { a.to } else { a.value_at(e) }
            }
            _ => self.value,
        }
    }

    /// Advances to `now`: returns the value and drops finished animations.
    pub fn tick(&mut self, now: Instant) -> f32 {
        let v = self.value_at(now);
        if self.is_finished(now) {
            self.value = self.target;
            self.running = None;
            self.start = None;
            return self.target as f32;
        }
        v as f32
    }

    pub fn is_animating(&self) -> bool {
        self.running.is_some()
    }

    fn is_finished(&self, now: Instant) -> bool {
        match (&self.running, self.start) {
            (Some(Running::Timed(a)), Some(s)) => a.is_finished(now.saturating_duration_since(s)),
            (Some(Running::Spring(a, est)), Some(s)) => {
                let e = now.saturating_duration_since(s);
                est.is_some_and(|d| e >= d) || (a.clamp && a.is_finished(e))
            }
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn every_easing_starts_at_zero_and_ends_at_one() {
        use Easing::*;
        for e in [
            Linear, EaseInQuad, EaseOutQuad, EaseInOutQuad, EaseInCubic, EaseOutCubic, EaseInOutCubic,
            EaseInQuart, EaseOutQuart, EaseInOutQuart, EaseInQuint, EaseOutQuint, EaseInOutQuint,
            EaseInSine, EaseOutSine, EaseInOutSine, EaseInExpo, EaseOutExpo, EaseInOutExpo, EaseInCirc,
            EaseOutCirc, EaseInOutCirc, EaseInElastic, EaseOutElastic, EaseInOutElastic, EaseInBack,
            EaseOutBack, EaseInOutBack, EaseInBounce, EaseOutBounce, EaseInOutBounce, Ease, EaseIn,
            EaseOut, EaseInOut, Easing::STYLE_EASE_OUT_QUAD,
        ] {
            assert!(e.ease(0.0).abs() < 1e-3, "{e:?} at 0 = {}", e.ease(0.0));
            assert!((e.ease(1.0) - 1.0).abs() < 1e-3, "{e:?} at 1 = {}", e.ease(1.0));
        }
    }

    #[test]
    fn easing_shapes() {
        assert!(approx(Easing::EaseInQuad.ease(0.5), 0.25));
        assert!(approx(Easing::EaseOutQuad.ease(0.5), 0.75));
        assert!(approx(Easing::EaseInOutCubic.ease(0.5), 0.5));
        assert!(Easing::EaseOutBack.ease(0.8) > 1.0, "back overshoots");
        assert!(Easing::EaseInBack.ease(0.2) < 0.0, "back backtracks first");
        // CSS ease-out starts fast: more than half-way a third of the way in.
        assert!(Easing::EaseOut.ease(0.33) > 0.45);
        // ease-in-out is symmetric around the middle.
        assert!((Easing::EaseInOut.ease(0.5) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn timed_animation_interpolates_and_finishes() {
        let a = TimedAnimation::new(10.0, 20.0, Duration::from_millis(200), Easing::Linear);
        assert!(approx(a.value_at(Duration::ZERO), 10.0));
        assert!(approx(a.value_at(Duration::from_millis(100)), 15.0));
        assert!(approx(a.value_at(Duration::from_millis(500)), 20.0));
        assert!(a.is_finished(Duration::from_millis(200)) && !a.is_finished(Duration::from_millis(199)));
    }

    #[test]
    fn timed_reverse_repeat_alternate() {
        let base = TimedAnimation::new(0.0, 1.0, Duration::from_millis(100), Easing::Linear);
        // reverse runs from `to` back to `from`.
        let r = base.reverse(true);
        assert!(approx(r.value_at(Duration::ZERO), 1.0));
        assert!(approx(r.value_at(Duration::from_millis(100)), 0.0));
        // repeat twice: restarts at `from`.
        let rep = base.repeat(Repeat::Times(2));
        assert!(approx(rep.value_at(Duration::from_millis(150)), 0.5));
        assert!(approx(rep.value_at(Duration::from_millis(1000)), 1.0));
        // alternate flips every other iteration.
        let alt = base.repeat(Repeat::Times(2)).alternate(true);
        assert!(approx(alt.value_at(Duration::from_millis(125)), 0.75));
        // forever never finishes.
        assert!(!base.repeat(Repeat::Forever).is_finished(Duration::from_secs(100)));
    }

    #[test]
    fn spring_params_damping_ratio_round_trips() {
        let p = SpringParams::new(0.8, 1.0, 400.0);
        assert!(approx(p.damping_ratio(), 0.8));
        assert!(approx(p.damping, 2.0 * 0.8 * 20.0));
        assert!(approx(SpringParams::new_full(1.0, 400.0, 32.0).damping_ratio(), 0.8));
    }

    #[test]
    fn critically_damped_spring_never_overshoots_and_settles() {
        let a = SpringAnimation::new(0.0, 1.0, presets::navigation());
        let mut prev = -1.0;
        for ms in 0..400 {
            let v = a.value_at(Duration::from_millis(ms));
            assert!(v <= 1.0 + 1e-9 && v >= prev - 1e-9, "monotonic, no overshoot");
            prev = v;
        }
        let d = a.estimated_duration().unwrap();
        assert!(d > Duration::from_millis(100) && d < Duration::from_millis(800), "{d:?}");
        assert!(approx(a.value_at(d + Duration::from_millis(10)), 1.0));
    }

    #[test]
    fn underdamped_spring_overshoots_unless_clamped() {
        let p = presets::sheet(false); // damping ratio 0.8
        let a = SpringAnimation::new(0.0, 1.0, p);
        let peak = (0..600).map(|ms| a.value_at(Duration::from_millis(ms))).fold(0.0, f64::max);
        assert!(peak > 1.0, "ratio 0.8 overshoots, got {peak}");
        let clamped = a.clamp(true);
        let peak = (0..600).map(|ms| clamped.value_at(Duration::from_millis(ms))).fold(0.0, f64::max);
        assert!(peak <= 1.0 + 1e-9, "clamp never overshoots, got {peak}");
        // reduced motion uses ratio 1: no overshoot.
        let calm = SpringAnimation::new(0.0, 1.0, presets::sheet(true));
        assert!((0..600).all(|ms| calm.value_at(Duration::from_millis(ms)) <= 1.0 + 1e-9));
    }

    #[test]
    fn spring_initial_velocity_and_undamped() {
        let a = SpringAnimation::new(1.0, 1.0, presets::navigation()).initial_velocity(5.0);
        let peak = (0..300).map(|ms| a.value_at(Duration::from_millis(ms))).fold(1.0, f64::max);
        assert!(peak > 1.0, "equal from/to with velocity bounces");
        assert!(SpringAnimation::new(0.0, 1.0, SpringParams::new(0.0, 1.0, 100.0)).estimated_duration().is_none());
    }

    #[test]
    fn tween_retargets_continuously_and_respects_the_switch() {
        set_enabled(Some(true));
        let t0 = Instant::now();
        let mut t = Tween::new(0.0);
        t.set_target(1.0, t0, Spec::Timed(Duration::from_millis(200), Easing::Linear));
        let mid = t.value_at(t0 + Duration::from_millis(100));
        assert!((mid - 0.5).abs() < 1e-6);
        // Retarget mid-flight: restarts from the current value, no jump.
        let t1 = t0 + Duration::from_millis(100);
        t.set_target(0.0, t1, Spec::Timed(Duration::from_millis(200), Easing::Linear));
        assert!((t.value_at(t1) - 0.5).abs() < 1e-6);
        assert!(t.is_animating());
        assert_eq!(t.tick(t1 + Duration::from_millis(300)), 0.0);
        assert!(!t.is_animating());
        // Animations disabled: jumps straight to the target.
        set_enabled(Some(false));
        let mut j = Tween::new(0.0);
        j.set_target(1.0, t0, Spec::STYLE);
        assert_eq!(j.value_at(t0), 1.0);
        assert!(!j.is_animating());
        set_enabled(None);
    }

    #[test]
    fn presets_match_libadwaita() {
        assert!(approx(presets::navigation().stiffness, 1000.0));
        assert!(approx(presets::sidebar().mass, 0.5));
        assert!(approx(presets::sheet(false).damping_ratio(), 0.8));
        assert_eq!(presets::REVEAL.0, Duration::from_millis(250));
        assert_eq!(presets::TOAST_SHOW.0, Duration::from_millis(300));
        assert_eq!(presets::STYLE_TRANSITION.0, Duration::from_millis(200));
    }
}
