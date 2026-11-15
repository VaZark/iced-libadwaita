//! Trying a theme out: the gallery and the toy windows (`theme-toys <design>`), each started with
//! `IAD_THEME` pointing at a temp file that holds the values being edited, never at the file
//! itself.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Child, Command};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum LaunchScheme {
    #[default]
    System,
    Light,
    Dark,
}

impl LaunchScheme {
    pub(crate) const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
    /// Set only the child process environment; System clears inherited launch overrides.
    pub(crate) fn apply_environment(self, command: &mut Command) {
        for key in ["TOYS_SCHEME", "GALLERY_SCHEME"] {
            match self.value() {
                Some(value) => {
                    command.env(key, value);
                }
                None => {
                    command.env_remove(key);
                }
            }
        }
    }

    fn value(self) -> Option<&'static str> {
        match self {
            Self::System => None,
            Self::Light => Some("light"),
            Self::Dark => Some("dark"),
        }
    }
}

/// The design languages a `theme-toys` build is asked about when it cannot be run for its list.
pub(crate) const FALLBACK: [(&str, &str); 5] = [
    ("win95", "Windows 95"),
    ("winxp", "Windows XP"),
    ("win7", "Windows 7"),
    ("mac", "Mac"),
    ("gnome2", "GNOME 2"),
];

/// What can be tried out besides the gallery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Toy {
    pub(crate) name: String,
    pub(crate) label: String,
}

/// Which programs to start: `IAD_GALLERY` and `IAD_TOYS` name them, else the ones next to the
/// editor, else `cargo run` in the workspace.
#[derive(Debug, Clone, Default)]
pub(crate) struct Bins {
    pub(crate) gallery: Option<PathBuf>,
    pub(crate) toys: Option<PathBuf>,
}

impl Bins {
    pub(crate) fn from_env() -> Self {
        let sibling = |name: &str| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join(name)))
                .filter(|p| p.is_file())
        };
        Self {
            gallery: std::env::var_os("IAD_GALLERY")
                .map(PathBuf::from)
                .or_else(|| sibling("gallery")),
            toys: std::env::var_os("IAD_TOYS")
                .map(PathBuf::from)
                .or_else(|| sibling("theme-toys")),
        }
    }
}

/// The toys a `theme-toys --list` printed: one per line, a name then an optional label
/// (`win95  Windows 95`, `win95: Windows 95` or `win95<TAB>Windows 95`).
pub(crate) fn parse_list(text: &str) -> Vec<Toy> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (name, label) = line
                .split_once(|c: char| c.is_whitespace() || c == ':')
                .map_or((line, ""), |(n, l)| {
                    (n, l.trim_start_matches([' ', '\t', ':', '-']))
                });
            let name = name.trim();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return None;
            }
            let label = if label.trim().is_empty() {
                name
            } else {
                label.trim()
            };
            Some(Toy {
                name: name.to_string(),
                label: label.to_string(),
            })
        })
        .collect()
}

/// The fixed list, used when the toys program cannot say.
pub(crate) fn fallback() -> Vec<Toy> {
    FALLBACK
        .iter()
        .map(|(n, l)| Toy {
            name: (*n).to_string(),
            label: (*l).to_string(),
        })
        .collect()
}

/// What `theme-toys --list` says, or the fixed five when there is no program to ask.
pub(crate) fn list_toys(bins: &Bins) -> Vec<Toy> {
    let Some(bin) = &bins.toys else {
        return fallback();
    };
    match Command::new(bin).arg("--list").output() {
        Ok(out) if out.status.success() => {
            let toys = parse_list(&String::from_utf8_lossy(&out.stdout));
            if toys.is_empty() { fallback() } else { toys }
        }
        _ => fallback(),
    }
}

/// The program that runs `kind` (`gallery`, or a toy's name).
fn command(kind: &str, bins: &Bins) -> Command {
    let workspace = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    if kind == "gallery" {
        return match &bins.gallery {
            Some(path) => Command::new(path),
            None => {
                let mut c = Command::new("cargo");
                c.args(["run", "-q", "-p", "iced-adaptive-design-gallery"])
                    .current_dir(workspace);
                c
            }
        };
    }
    match &bins.toys {
        Some(path) => {
            let mut c = Command::new(path);
            c.arg(kind);
            c
        }
        None => {
            let mut c = Command::new("cargo");
            c.args([
                "run",
                "-q",
                "-p",
                "iced-adaptive-design-theme-toys",
                "--",
                kind,
            ])
            .current_dir(workspace);
            c
        }
    }
}

/// The windows the editor started, one per kind: starting one again replaces the old one.
#[derive(Default)]
pub(crate) struct Runner {
    children: BTreeMap<String, Child>,
}

impl Runner {
    /// Writes `theme_text` to a temp file of its own and starts `kind` with `IAD_THEME` pointing
    /// at it. Returns the temp file's path.
    #[cfg(test)]
    pub(crate) fn launch(
        &mut self,
        kind: &str,
        theme_text: &str,
        bins: &Bins,
    ) -> Result<PathBuf, String> {
        self.launch_with_scheme(kind, theme_text, bins, LaunchScheme::System)
    }

    pub(crate) fn launch_with_scheme(
        &mut self,
        kind: &str,
        theme_text: &str,
        bins: &Bins,
        scheme: LaunchScheme,
    ) -> Result<PathBuf, String> {
        if let Some(mut old) = self.children.remove(kind) {
            let _ = old.kill();
            let _ = old.wait();
        }
        let file = std::env::temp_dir().join(format!(
            "iad-theme-editor-{}-{kind}.conf",
            std::process::id()
        ));
        std::fs::write(&file, theme_text)
            .map_err(|e| format!("Could not write {}: {e}", file.display()))?;
        let mut cmd = command(kind, bins);
        cmd.env("IAD_THEME", &file);
        scheme.apply_environment(&mut cmd);
        if let Some(value) = scheme.value() {
            // Gallery consumes the shared desktop configuration; toys additionally expose
            // a process-local override so even opt-out toy themes can be forced.
            let config_file = file.with_extension("desktop.conf");
            let mut config = String::new();
            for path in iced_adaptive_design::config::Config::search_paths() {
                if let Ok(text) = std::fs::read_to_string(path) {
                    config.push_str(&text);
                    config.push('\n');
                }
            }
            config.push_str(&format!("\n[appearance]\ncolor-scheme={value}\n"));
            std::fs::write(&config_file, config)
                .map_err(|e| format!("Could not write launch configuration: {e}"))?;
            cmd.env("ICED_ADAPTIVE_DESIGN_CONFIG", &config_file);
        }
        let child = cmd
            .spawn()
            .map_err(|e| format!("Could not start {kind}: {e}"))?;
        self.children.insert(kind.to_string(), child);
        Ok(file)
    }

    /// Whether `kind` was started and is still there.
    #[cfg(test)]
    pub(crate) fn running(&mut self, kind: &str) -> bool {
        self.children
            .get_mut(kind)
            .is_some_and(|c| matches!(c.try_wait(), Ok(None)))
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        for (_, mut c) in std::mem::take(&mut self.children) {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}
