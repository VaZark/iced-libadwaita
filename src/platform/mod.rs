//! The platform an app runs on, and how it identifies itself there.
//!
//! An app picks nothing by hand: [`Target::current`] says which target the build is for, and
//! [`windows::settings_for`](crate::windows::settings_for) configures the window for it. Only
//! Wayland is supported on Linux (no X11). Everything an app still has to ship itself (its icon,
//! its name, its categories) is data for [`linux::desktop_entry`].

pub mod linux;

/// What the build targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Linux under a Wayland compositor. The window's app id must equal the basename of the
    /// app's `.desktop` file, which is how GNOME Resources and the shell group its windows.
    LinuxWayland,
    /// Any other OS: plain iced window settings, no app identity.
    Other,
}

impl Target {
    /// The target this build is for.
    pub const fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::LinuxWayland
        } else {
            Self::Other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_matches_the_build() {
        assert_eq!(Target::current() == Target::LinuxWayland, cfg!(target_os = "linux"));
    }
}
