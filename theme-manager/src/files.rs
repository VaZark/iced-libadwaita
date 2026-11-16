//! The files the editor works with: the theme list, the registry of symlinks apps see, and what
//! the editor remembers between runs.

use iced_adaptive_design as iad;
use std::path::PathBuf;

/// Which variant of the theme is being edited: the base, or the one for a color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Variant {
    Base,
    Light,
    Dark,
}

impl Variant {
    pub(crate) const ALL: [Variant; 3] = [Variant::Base, Variant::Light, Variant::Dark];
    pub(crate) fn label(self) -> &'static str {
        match self {
            Variant::Base => "Base",
            Variant::Light => "Light",
            Variant::Dark => "Dark",
        }
    }
    pub(crate) fn key(self) -> &'static str {
        match self {
            Variant::Base => "base",
            Variant::Light => "light",
            Variant::Dark => "dark",
        }
    }
    pub(crate) fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.key() == key.trim())
    }
    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Variant::Base => "",
            Variant::Light => "light:",
            Variant::Dark => "dark:",
        }
    }
}

/// One entry of the Themes list: a real file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Preset {
    pub(crate) group: &'static str,
    /// The key of the bundled theme this entry stands for; `path` is empty then.
    pub(crate) bundled: Option<&'static str>,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    /// Where the entry points, when the file is a symlink.
    pub(crate) target: Option<PathBuf>,
    /// `default.conf` points at this file: the global theme, the one apps use when `IAD_THEME` is unset.
    pub(crate) is_default: bool,
}

/// What a click on the list or on New asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    File(PathBuf),
    Bundled(&'static str),
    New,
}

/// What the editor remembers between runs, in `$XDG_CONFIG_HOME/iad-theme-editor/state.conf`:
/// `[state] last-file`, `folders`, `recent` (paths joined with `:`), `section`, `variant`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct State {
    pub(crate) last_file: Option<PathBuf>,
    pub(crate) folders: Vec<PathBuf>,
    pub(crate) recent: Vec<PathBuf>,
    pub(crate) section: Option<String>,
    pub(crate) variant: Option<Variant>,
}

pub(crate) const RECENT_MAX: usize = 8;

impl State {
    pub(crate) fn parse(text: &str) -> Self {
        let kv = iad::config::parse_keyfile(text);
        let get = |k: &str| kv.get(&("state".to_string(), k.to_string()));
        let paths = |k: &str| {
            get(k)
                .map(|v| {
                    v.split(':')
                        .filter(|p| !p.is_empty())
                        .map(PathBuf::from)
                        .collect()
                })
                .unwrap_or_default()
        };
        Self {
            last_file: get("last-file")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from),
            folders: paths("folders"),
            recent: paths("recent"),
            section: get("section").filter(|v| !v.is_empty()).cloned(),
            variant: get("variant").and_then(|v| Variant::from_key(v)),
        }
    }

    pub(crate) fn to_text(&self) -> String {
        let join = |ps: &[PathBuf]| {
            ps.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(":")
        };
        format!(
            "[state]\nlast-file={}\nfolders={}\nrecent={}\nsection={}\nvariant={}\n",
            self.last_file
                .as_ref()
                .map_or(String::new(), |p| p.display().to_string()),
            join(&self.folders),
            join(&self.recent),
            self.section.clone().unwrap_or_default(),
            self.variant.map_or("base", Variant::key),
        )
    }

    /// Puts `path` first in the recent list, once, keeping at most [`RECENT_MAX`].
    pub(crate) fn remember(&mut self, path: &std::path::Path) {
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_path_buf());
        self.recent.truncate(RECENT_MAX);
    }
}

/// Where the state is kept: `THEME_EDITOR_STATE`, else the user's config directory.
pub(crate) fn state_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("THEME_EDITOR_STATE").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|d| d.join("iad-theme-editor/state.conf"))
}

/// The `*.conf` files directly in `dir`, by name.
pub(crate) fn scan_folder(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "conf"))
        .collect();
    files.sort();
    files
}

/// A theme file's display name: its `[theme] name`, else its file name without `.conf`.
pub(crate) fn theme_name(path: &std::path::Path) -> String {
    let stem = path
        .file_stem()
        .map_or(String::new(), |s| s.to_string_lossy().into_owned());
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| {
            iad::config::parse_keyfile(&t)
                .get(&("theme".to_string(), "name".to_string()))
                .filter(|n| !n.is_empty())
                .cloned()
        })
        .unwrap_or(stem)
}

pub(crate) fn canon(p: &std::path::Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

pub(crate) fn is_link(p: &std::path::Path) -> bool {
    std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink())
}

pub(crate) fn is_default_name(p: &std::path::Path) -> bool {
    p.file_name().is_some_and(|n| n == "default.conf")
}

// DECISION (user, 2026-11-16): every theme is a file of its own, always, and `default.conf` in the
// standard theme directory is always a symlink to the current global theme's file. "Set as global
// theme" is the one way to share a theme: it points that symlink at the theme (copying a bundled
// theme into the directory first). A regular file that is in the way is kept as a theme of its own,
// never overwritten. There is no registering by symlink, and no unregistering.

/// The file `default.conf` in `dir` leads to (through the symlink), if there is one.
pub(crate) fn default_target(dir: &std::path::Path) -> Option<PathBuf> {
    let link = dir.join("default.conf");
    std::fs::symlink_metadata(&link)
        .ok()
        .and_then(|_| std::fs::canonicalize(link).ok())
}

/// A name for `text`'s theme to keep a file under in `dir`: its `[theme] name` as a file name, else
/// `fallback`, made unique so no existing file is overwritten.
fn free_name(dir: &std::path::Path, text: &str, fallback: &str) -> PathBuf {
    let name = crate::bundled::name_of(text, fallback);
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = if stem.trim_matches('_').is_empty() {
        fallback.to_string()
    } else {
        stem
    };
    let mut candidate = dir.join(format!("{stem}.conf"));
    let mut n = 1;
    while std::fs::symlink_metadata(&candidate).is_ok() {
        n += 1;
        candidate = dir.join(format!("{stem}-{n}.conf"));
    }
    candidate
}

/// What setting the global theme did.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Linked {
    /// `default.conf`.
    pub(crate) link: PathBuf,
    /// A regular `default.conf` that was in the way, kept as a theme of its own at this path.
    pub(crate) kept: Option<PathBuf>,
}

/// Points `default.conf` in `dir` at `file`: it creates or replaces the symlink, atomically (a
/// temporary symlink is renamed over it), so an app never finds it missing. A regular file called
/// `default.conf` is not overwritten: it is renamed to a theme file of its own first, and when it
/// is `file` itself the link leads to that new name (`Linked::kept`).
pub(crate) fn set_default(dir: &std::path::Path, file: &std::path::Path) -> Result<Linked, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let link = dir.join("default.conf");
    let mut target = canon(file);
    let mut kept = None;
    if std::fs::symlink_metadata(&link).is_ok_and(|m| !m.file_type().is_symlink()) {
        let is_file = target == canon(&link);
        let text = std::fs::read_to_string(&link).unwrap_or_default();
        let new = free_name(dir, &text, "previous-default");
        std::fs::rename(&link, &new).map_err(|e| format!("{}: {e}", link.display()))?;
        if is_file {
            target = canon(&new);
        }
        kept = Some(new);
    }
    let temp = dir.join(format!(".default.conf.{}.tmp", std::process::id()));
    let _ = std::fs::remove_file(&temp);
    std::os::unix::fs::symlink(&target, &temp).map_err(|e| format!("{}: {e}", temp.display()))?;
    std::fs::rename(&temp, &link).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("{}: {e}", link.display())
    })?;
    Ok(Linked { link, kept })
}

/// Makes the bundled theme `key` the global theme: its text is installed as its own file,
/// `<key>.conf` in `dir` (an installed file of that name with other text is the user's and is
/// never overwritten), and `default.conf` points at it.
pub(crate) fn set_global_bundled(dir: &std::path::Path, key: &str) -> Result<Linked, String> {
    let text = crate::bundled::text(key).ok_or_else(|| format!("no bundled theme {key}"))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let file = dir.join(format!("{key}.conf"));
    match std::fs::read_to_string(&file) {
        Ok(existing) if existing == text => {}
        Ok(_) => {
            return Err(format!(
                "{} already exists with other values: open it to use it, or remove it first",
                file.display()
            ));
        }
        Err(_) => std::fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?,
    }
    set_default(dir, &file)
}

/// Removes the global theme: the `default.conf` symlink in `dir` (never the theme it leads to).
/// Apps then show the bare toolkit look, or their own base theme.
pub(crate) fn clear_global(dir: &std::path::Path) -> Result<(), String> {
    let link = dir.join("default.conf");
    match std::fs::symlink_metadata(&link) {
        Err(_) => Err("there is no global theme".into()),
        Ok(m) if m.file_type().is_symlink() => {
            std::fs::remove_file(&link).map_err(|e| format!("{}: {e}", link.display()))
        }
        Ok(_) => Err(format!("{} is a regular file: not removed", link.display())),
    }
}

/// The themes available, each once: the installed ones (the files in the data dirs `dirs`, and
/// the file `default.conf` leads to, marked), the bundled ones not installed unchanged, those in
/// the folders the user added, then the recently opened ones.
pub(crate) fn list_presets(
    dirs: &[PathBuf],
    folders: &[PathBuf],
    recent: &[PathBuf],
) -> Vec<Preset> {
    let default = dirs.iter().find_map(|d| default_target(d));
    let mut out: Vec<Preset> = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();
    let add = |out: &mut Vec<Preset>,
               seen: &mut Vec<PathBuf>,
               group: &'static str,
               files: Vec<PathBuf>| {
        for f in files {
            let key = canon(&f);
            if seen.contains(&key) {
                continue;
            }
            out.push(Preset {
                group,
                bundled: None,
                name: theme_name(&f),
                target: is_link(&f).then(|| key.clone()),
                is_default: default.as_ref() == Some(&key),
                path: f,
            });
            seen.push(key);
        }
    };
    add(
        &mut out,
        &mut seen,
        "Installed",
        dirs.iter()
            .flat_map(|d| scan_folder(d))
            .filter(|p| !is_default_name(p))
            .collect(),
    );
    if let Some(d) = &default {
        add(&mut out, &mut seen, "Installed", vec![d.clone()]);
    }
    // A bundled theme already installed unchanged is listed once, as the installed file.
    let installed: Vec<String> = out
        .iter()
        .filter_map(|p| std::fs::read_to_string(&p.path).ok())
        .collect();
    for (key, text) in crate::bundled::BUNDLED {
        if installed.iter().any(|t| t == text) {
            continue;
        }
        out.push(Preset {
            group: "Bundled",
            bundled: Some(key),
            name: crate::bundled::name_of(text, key),
            path: PathBuf::new(),
            target: None,
            is_default: false,
        });
    }
    for folder in folders {
        add(&mut out, &mut seen, "Folders", scan_folder(folder));
    }
    add(
        &mut out,
        &mut seen,
        "Recent",
        recent.iter().filter(|p| p.is_file()).cloned().collect(),
    );
    out
}
