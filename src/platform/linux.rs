//! Linux (Wayland) app identity: the application id and the `.desktop` entry that goes with it.

/// Whether `id` is a valid application id: reverse-DNS, at least two dot-separated segments of
/// ASCII letters, digits, `_` or `-`, none empty and none starting with a digit.
///
/// The window's app id and the `.desktop` file basename are this same string.
pub fn valid_app_id(id: &str) -> bool {
    let mut segments = 0;
    for s in id.split('.') {
        let mut chars = s.chars();
        let ok = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '-')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !ok {
            return false;
        }
        segments += 1;
    }
    segments >= 2
}

/// What a `.desktop` entry says about an app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry<'a> {
    /// The application id; the file is named `<app_id>.desktop`.
    pub app_id: &'a str,
    pub name: &'a str,
    pub comment: &'a str,
    /// The command to run, as installed (a binary name on `PATH`).
    pub exec: &'a str,
    /// An icon name, normally the app id.
    pub icon: &'a str,
    /// Menu categories, such as `AudioVideo` and `Audio`.
    pub categories: &'a [&'a str],
}

impl DesktopEntry<'_> {
    /// The file name to install it as.
    pub fn file_name(&self) -> String {
        format!("{}.desktop", self.app_id)
    }

    /// The text of the `.desktop` file. No `StartupWMClass`: Wayland matches on the file name.
    pub fn render(&self) -> String {
        let mut categories = self.categories.join(";");
        if !categories.is_empty() {
            categories.push(';');
        }
        format!(
            "[Desktop Entry]\nType=Application\nName={}\nComment={}\nExec={}\nIcon={}\nCategories={}\nTerminal=false\n",
            self.name, self.comment, self.exec, self.icon, categories
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_ids() {
        assert!(valid_app_id("io.github.vazark.Weave"));
        assert!(valid_app_id("org.example.app-1"));
        assert!(!valid_app_id("weave"));
        assert!(!valid_app_id("io..weave"));
        assert!(!valid_app_id("io.1weave"));
        assert!(!valid_app_id("io.weave app"));
        assert!(!valid_app_id(""));
    }

    #[test]
    fn renders_a_desktop_entry() {
        let e = DesktopEntry {
            app_id: "io.example.App",
            name: "App",
            comment: "Does things",
            exec: "app",
            icon: "io.example.App",
            categories: &["AudioVideo", "Audio"],
        };
        assert_eq!(e.file_name(), "io.example.App.desktop");
        let text = e.render();
        assert!(text.starts_with("[Desktop Entry]\nType=Application\n"));
        assert!(text.contains("\nCategories=AudioVideo;Audio;\n"));
        assert!(!text.contains("StartupWMClass"));
    }
}
