//! Synced lines: lines that carry a time, a position that moves through them, and where to
//! scroll. Lyrics are the first use; subtitles, a transcript or a changelog timeline are the same
//! thing. Pure: the application owns the position and asks these functions what it means.

/// One line, shown from `at` until the next line starts.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// Seconds from the start.
    pub at: f32,
    pub text: String,
}

impl Line {
    pub fn new(at: f32, text: impl Into<String>) -> Self {
        Self {
            at,
            text: text.into(),
        }
    }
}

/// The line active at `pos`: the last one that started. `None` before the first.
/// `lines` must be ordered by time.
pub fn current(lines: &[Line], pos: f32) -> Option<usize> {
    lines.partition_point(|l| l.at <= pos).checked_sub(1)
}

/// The scroll offset that brings line `current` into view in a list of `total` lines, as a
/// fraction (`0..=1`) of the scrollable range, vertically. With no current line it is the top.
pub fn scroll_to(current: Option<usize>, total: usize) -> f32 {
    let y = match current {
        Some(i) if total > 1 => i as f32 / (total - 1) as f32,
        _ => 0.0,
    };
    y.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> Vec<Line> {
        [0.0, 5.0, 9.5]
            .iter()
            .map(|at| Line::new(*at, ""))
            .collect()
    }

    #[test]
    fn the_current_line_is_the_last_one_started() {
        let l = lines();
        assert_eq!(current(&l, -1.0), None);
        assert_eq!(current(&l, 0.0), Some(0));
        assert_eq!(current(&l, 4.99), Some(0));
        assert_eq!(current(&l, 5.0), Some(1));
        assert_eq!(current(&l, 100.0), Some(2));
        assert_eq!(current(&[], 3.0), None);
    }

    #[test]
    fn scrolling_follows_the_line() {
        assert_eq!(scroll_to(None, 10), 0.0);
        assert_eq!(scroll_to(Some(0), 10), 0.0);
        assert_eq!(scroll_to(Some(9), 10), 1.0);
        assert_eq!(scroll_to(Some(3), 1), 0.0);
        assert!((scroll_to(Some(1), 3) - 0.5).abs() < 1e-6);
    }
}
