//! Scales: a domain mapped to a range, as in d3-scale.

/// The "nice" step for about `count` ticks over `span`: 1, 2 or 5 times a power of ten.
pub fn tick_step(span: f32, count: usize) -> f32 {
    step64(span as f64, count) as f32
}

fn step64(span: f64, count: usize) -> f64 {
    let raw = span / count.max(1) as f64;
    if raw <= 0.0 || !raw.is_finite() {
        return 1.0;
    }
    let power = raw.log10().floor();
    let base = 10f64.powf(power);
    let error = raw / base;
    let factor = if error >= 50f64.sqrt() {
        10.0
    } else if error >= 10f64.sqrt() {
        5.0
    } else if error >= 2f64.sqrt() {
        2.0
    } else {
        1.0
    };
    factor * base
}

/// Evenly spaced round values covering `a..=b`, about `count` of them.
pub fn ticks(a: f32, b: f32, count: usize) -> Vec<f32> {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    if lo == hi {
        return vec![lo];
    }
    let step = step64(hi as f64 - lo as f64, count);
    // The tolerance absorbs the error of an f32 bound that is meant to be a multiple of the step.
    let first = (lo as f64 / step - 1e-6).ceil() as i64;
    let last = (hi as f64 / step + 1e-6).floor() as i64;
    // Multiply the integer index, never accumulate: no float drift.
    (first..=last).map(|i| (i as f64 * step) as f32).collect()
}

/// The domain widened to round values, so the axis starts and ends on a tick.
pub fn nice(a: f32, b: f32, count: usize) -> (f32, f32) {
    if a == b {
        return (a, b + 1.0);
    }
    let step = step64((b as f64 - a as f64).abs(), count);
    (
        ((a as f64 / step).floor() * step) as f32,
        ((b as f64 / step).ceil() * step) as f32,
    )
}

/// A tick as text, with only as many decimals as the step needs (`2.5`, `1000`).
pub fn format_tick(v: f32, step: f32) -> String {
    let decimals = if step >= 1.0 {
        0
    } else {
        (-(step as f64).log10().floor()) as usize
    };
    let s = format!("{v:.decimals$}");
    if s == "-0" { "0".into() } else { s }
}

/// A continuous scale: `domain` linearly mapped to `range` (either may run backwards, so a
/// y axis maps up to down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    domain: (f32, f32),
    range: (f32, f32),
}

impl Linear {
    pub fn new(domain: (f32, f32), range: (f32, f32)) -> Self {
        Self { domain, range }
    }
    pub fn domain(&self) -> (f32, f32) {
        self.domain
    }
    pub fn range(&self) -> (f32, f32) {
        self.range
    }
    /// The position of `v`. A zero-width domain maps to the middle of the range.
    pub fn map(&self, v: f32) -> f32 {
        let span = self.domain.1 - self.domain.0;
        if span == 0.0 {
            return (self.range.0 + self.range.1) / 2.0;
        }
        self.range.0 + (v - self.domain.0) / span * (self.range.1 - self.range.0)
    }
    /// The value at position `px`.
    pub fn invert(&self, px: f32) -> f32 {
        let span = self.range.1 - self.range.0;
        if span == 0.0 {
            return self.domain.0;
        }
        self.domain.0 + (px - self.range.0) / span * (self.domain.1 - self.domain.0)
    }
    /// The domain widened to round values.
    pub fn nice(self, count: usize) -> Self {
        let (a, b) = nice(self.domain.0, self.domain.1, count);
        Self {
            domain: (a, b),
            ..self
        }
    }
    pub fn ticks(&self, count: usize) -> Vec<f32> {
        ticks(self.domain.0, self.domain.1, count)
    }
}

/// A scale for categories: `n` bands over a range, with space between them (d3's `scaleBand`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    n: usize,
    range: (f32, f32),
    inner: f32,
    outer: f32,
}

impl Band {
    /// `n` bands across `range`, with `padding` (0 to 1) between and around them.
    pub fn new(n: usize, range: (f32, f32), padding: f32) -> Self {
        let p = padding.clamp(0.0, 1.0);
        Self {
            n,
            range,
            inner: p,
            outer: p,
        }
    }
    fn step(&self) -> f32 {
        let span = self.range.1 - self.range.0;
        span / (self.n as f32 - self.inner + 2.0 * self.outer).max(1.0)
    }
    /// The width of one band.
    pub fn bandwidth(&self) -> f32 {
        self.step() * (1.0 - self.inner)
    }
    /// Where band `i` starts.
    pub fn start(&self, i: usize) -> f32 {
        let span = self.range.1 - self.range.0;
        let used = self.step() * (self.n as f32 - self.inner);
        self.range.0 + (span - used) / 2.0 + self.step() * i as f32
    }
    pub fn center(&self, i: usize) -> f32 {
        self.start(i) + self.bandwidth() / 2.0
    }
    /// The band whose slot (band and half the gap either side) holds `px`.
    pub fn index_at(&self, px: f32) -> Option<usize> {
        if self.n == 0 {
            return None;
        }
        let i = ((px - self.start(0) + self.step() * self.inner / 2.0) / self.step()).floor();
        (i >= 0.0 && (i as usize) < self.n).then_some(i as usize)
    }
    pub fn len(&self) -> usize {
        self.n
    }
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_land_on_round_values() {
        assert_eq!(ticks(0.0, 10.0, 5), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(ticks(0.0, 1.0, 4), vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!(ticks(3.0, 3.0, 5), vec![3.0]);
        assert_eq!(ticks(7.0, 0.0, 7), ticks(0.0, 7.0, 7));
    }

    #[test]
    fn nice_widens_the_domain() {
        assert_eq!(nice(0.3, 9.2, 5), (0.0, 10.0));
        assert_eq!(nice(2.0, 2.0, 5), (2.0, 3.0));
    }

    #[test]
    fn ticks_print_without_noise() {
        assert_eq!(format_tick(0.6, 0.2), "0.6");
        assert_eq!(format_tick(2500.0, 500.0), "2500");
        assert_eq!(format_tick(-0.0, 1.0), "0");
    }

    #[test]
    fn linear_maps_and_inverts() {
        let y = Linear::new((0.0, 100.0), (200.0, 0.0));
        assert_eq!(y.map(0.0), 200.0);
        assert_eq!(y.map(25.0), 150.0);
        assert_eq!(y.invert(150.0), 25.0);
        assert_eq!(Linear::new((5.0, 5.0), (0.0, 10.0)).map(5.0), 5.0);
        assert_eq!(
            Linear::new((0.0, 9.3), (0.0, 1.0)).nice(5).domain(),
            (0.0, 10.0)
        );
    }

    #[test]
    fn bands_share_the_range() {
        let b = Band::new(4, (0.0, 100.0), 0.0);
        assert_eq!(b.bandwidth(), 25.0);
        assert_eq!(b.start(2), 50.0);
        assert_eq!(b.index_at(60.0), Some(2));
        assert_eq!(b.index_at(120.0), None);
        let p = Band::new(2, (0.0, 100.0), 0.5);
        assert!(p.bandwidth() < 50.0);
        assert!(p.start(0) > 0.0);
        assert!(p.start(1) + p.bandwidth() < 100.0);
        assert_eq!(p.index_at(p.center(1)), Some(1));
    }
}
