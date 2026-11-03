//! Stacked series: each value sits on the ones below it (d3-shape's `stack`).

/// What the stack starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Offset {
    /// Values keep their size: totals differ per category.
    #[default]
    Zero,
    /// Every category sums to 1 (a 100% stack).
    Expand,
}

/// `(low, high)` of every value, per series then category. Positive values stack upwards from
/// zero and negative ones downwards, so a series that dips below zero does not disturb the rest.
/// A missing value counts as zero.
pub fn stack(series: &[Vec<f32>], offset: Offset) -> Vec<Vec<(f32, f32)>> {
    let n = series.iter().map(Vec::len).max().unwrap_or(0);
    let mut up = vec![0.0f32; n];
    let mut down = vec![0.0f32; n];
    let mut out: Vec<Vec<(f32, f32)>> = series
        .iter()
        .map(|s| {
            (0..n)
                .map(|i| {
                    let v = s.get(i).copied().unwrap_or(0.0);
                    if v >= 0.0 {
                        let lo = up[i];
                        up[i] += v;
                        (lo, up[i])
                    } else {
                        let hi = down[i];
                        down[i] += v;
                        (down[i], hi)
                    }
                })
                .collect()
        })
        .collect();
    if offset == Offset::Expand {
        for i in 0..n {
            let total = up[i] - down[i];
            if total > 0.0 {
                for layer in out.iter_mut() {
                    layer[i].0 = (layer[i].0 - down[i]) / total;
                    layer[i].1 = (layer[i].1 - down[i]) / total;
                }
            }
        }
    }
    out
}

/// The lowest and highest value any layer reaches (`(0, 0)` when empty).
pub fn extent(layers: &[Vec<(f32, f32)>]) -> (f32, f32) {
    layers
        .iter()
        .flatten()
        .fold((0.0f32, 0.0f32), |(lo, hi), (a, b)| {
            (lo.min(*a), hi.max(*b))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_sit_on_each_other() {
        let s = stack(&[vec![1.0, 2.0], vec![3.0, 1.0]], Offset::Zero);
        assert_eq!(s[0], vec![(0.0, 1.0), (0.0, 2.0)]);
        assert_eq!(s[1], vec![(1.0, 4.0), (2.0, 3.0)]);
        assert_eq!(extent(&s), (0.0, 4.0));
    }

    #[test]
    fn negatives_stack_downwards() {
        let s = stack(&[vec![2.0], vec![-1.0], vec![1.0]], Offset::Zero);
        assert_eq!(s[1][0], (-1.0, 0.0));
        assert_eq!(s[2][0], (2.0, 3.0));
        assert_eq!(extent(&s), (-1.0, 3.0));
    }

    #[test]
    fn expand_makes_every_column_one() {
        let s = stack(&[vec![1.0, 0.0], vec![3.0, 0.0]], Offset::Expand);
        assert_eq!(s[1][0], (0.25, 1.0));
        // An empty column stays empty rather than dividing by zero.
        assert_eq!(s[1][1], (0.0, 0.0));
    }

    #[test]
    fn short_series_count_as_zero() {
        let s = stack(&[vec![1.0, 2.0], vec![5.0]], Offset::Zero);
        assert_eq!(s[1][1], (2.0, 2.0));
    }
}
