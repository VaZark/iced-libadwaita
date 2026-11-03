//! Development switches every app that screenshots itself wants, from the environment, so they
//! are one helper instead of each app's own.
//!
//! | Variable | Does |
//! | --- | --- |
//! | `ICED_ADAPTIVE_DESIGN_SCREENSHOT=path.png` | saves a screenshot of the window to that file |
//! | `ICED_ADAPTIVE_DESIGN_SHOT_DELAY_MS=1500` | waits this long first (a slow debug build, a page that loads) |
//! | `ICED_ADAPTIVE_DESIGN_HOLD_ANIMATIONS=0.5` | holds every animation half way (`0..=1`), so a slide or a reveal is captured at a known frame |
//!
//! ```ignore
//! let dev = dev::Switches::from_env();
//! dev.apply();                                     // holds animations, if asked
//! // in update, once the window is open:
//! if let Some(task) = dev.capture(window_id) {
//!     return task.map(Message::Captured);           // Result<PathBuf, String>; exit on Ok
//! }
//! ```
//!
//! None of it does anything unless a variable is set.

use iced::Task;
use iced::window;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// `ICED_ADAPTIVE_DESIGN_SCREENSHOT`.
pub const SCREENSHOT: &str = "ICED_ADAPTIVE_DESIGN_SCREENSHOT";
/// `ICED_ADAPTIVE_DESIGN_SHOT_DELAY_MS`.
pub const SHOT_DELAY_MS: &str = "ICED_ADAPTIVE_DESIGN_SHOT_DELAY_MS";
/// `ICED_ADAPTIVE_DESIGN_HOLD_ANIMATIONS`.
pub const HOLD_ANIMATIONS: &str = "ICED_ADAPTIVE_DESIGN_HOLD_ANIMATIONS";

/// The development switches that are set.
#[derive(Debug, Clone, PartialEq)]
pub struct Switches {
    /// Where to save a screenshot, if asked.
    pub screenshot: Option<PathBuf>,
    /// How long to wait before taking it (default 1.5 s).
    pub delay: Duration,
    /// The fraction of the way animations are held at, if asked.
    pub hold: Option<f32>,
}

impl Default for Switches {
    fn default() -> Self {
        Self {
            screenshot: None,
            delay: Duration::from_millis(1500),
            hold: None,
        }
    }
}

impl Switches {
    /// Reads the switches from the process environment.
    pub fn from_env() -> Self {
        Self::parse(|k| std::env::var(k).ok())
    }

    /// Reads the switches through `get`, so a test or an app with its own config can feed them.
    /// An unreadable value is ignored.
    pub fn parse(get: impl Fn(&str) -> Option<String>) -> Self {
        let default = Self::default();
        Self {
            screenshot: get(SCREENSHOT).filter(|p| !p.is_empty()).map(PathBuf::from),
            delay: get(SHOT_DELAY_MS)
                .and_then(|v| v.trim().parse::<u64>().ok())
                .map_or(default.delay, Duration::from_millis),
            hold: get(HOLD_ANIMATIONS)
                .and_then(|v| v.trim().parse::<f32>().ok())
                .filter(|f| f.is_finite())
                .map(|f| f.clamp(0.0, 1.0)),
        }
    }

    /// Whether any switch is on.
    pub fn any(&self) -> bool {
        self.screenshot.is_some() || self.hold.is_some()
    }

    /// Applies what applies at startup: holds the animations, if asked.
    pub fn apply(&self) {
        crate::animation::set_hold(self.hold);
    }

    /// A task that waits [`delay`](Self::delay), screenshots `window` and saves it as a PNG, or
    /// `None` when no screenshot was asked for. It finishes with the path, or what went wrong.
    pub fn capture(&self, window: window::Id) -> Option<Task<Result<PathBuf, String>>> {
        let path = self.screenshot.clone()?;
        let delay = self.delay;
        let wait = Task::future(async move {
            let (tx, rx) = iced::futures::channel::oneshot::channel();
            std::thread::spawn(move || {
                std::thread::sleep(delay);
                let _ = tx.send(());
            });
            let _ = rx.await;
        });
        Some(
            wait.then(move |_| window::screenshot(window))
                .map(move |shot| save_png(&shot, &path).map(|()| path.clone())),
        )
    }
}

/// Writes a screenshot as a PNG, creating the folder if needed.
pub fn save_png(shot: &window::Screenshot, path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    image::save_buffer(
        path,
        &shot.rgba,
        shot.size.width,
        shot.size.height,
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(pairs: &'static [(&'static str, &'static str)]) -> Switches {
        Switches::parse(|k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.to_string())
        })
    }

    #[test]
    fn nothing_set_does_nothing() {
        let s = with(&[]);
        assert_eq!(s, Switches::default());
        assert!(!s.any());
    }

    #[test]
    fn the_switches_are_read() {
        let s = with(&[
            (SCREENSHOT, "target/shot.png"),
            (SHOT_DELAY_MS, "2500"),
            (HOLD_ANIMATIONS, "0.5"),
        ]);
        assert_eq!(s.screenshot, Some(PathBuf::from("target/shot.png")));
        assert_eq!(s.delay, Duration::from_millis(2500));
        assert_eq!(s.hold, Some(0.5));
        assert!(s.any());
    }

    #[test]
    fn bad_values_are_ignored_and_the_hold_is_clamped() {
        let s = with(&[
            (SHOT_DELAY_MS, "soon"),
            (HOLD_ANIMATIONS, "7"),
            (SCREENSHOT, ""),
        ]);
        assert_eq!(s.delay, Duration::from_millis(1500));
        assert_eq!(s.hold, Some(1.0));
        assert_eq!(s.screenshot, None);
        assert_eq!(with(&[(HOLD_ANIMATIONS, "x")]).hold, None);
    }
}
