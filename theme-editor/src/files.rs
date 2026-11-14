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
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    /// Where the entry points, when the file is a symlink.
    pub(crate) target: Option<PathBuf>,
    /// `default.conf` points at this file: the one apps use when `IAD_THEME` is unset.
    pub(crate) is_default: bool,
}

/// What a click on the list or on New asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    File(PathBuf),
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
                .map(|v| v.split(':').filter(|p| !p.is_empty()).map(PathBuf::from).collect())
                .unwrap_or_default()
        };
        Self {
            last_file: get("last-file").filter(|v| !v.is_empty()).map(PathBuf::from),
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
            self.last_file.as_ref().map_or(String::new(), |p| p.display().to_string()),
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

/// The file `default.conf` in `dir` points at.
pub(crate) fn default_target(dir: &std::path::Path) -> Option<PathBuf> {
    let link = dir.join("default.conf");
    (std::fs::symlink_metadata(&link).is_ok())
        .then(|| std::fs::canonicalize(link).ok())
        .flatten()
}

/// The symlinks in `dir` (other than `default.conf`) that point at `file`.
pub(crate) fn links_to(dir: &std::path::Path, file: &std::path::Path) -> Vec<PathBuf> {
    let target = canon(file);
    scan_folder(dir)
        .into_iter()
        .filter(|p| is_link(p) && !is_default_name(p) && canon(p) == target)
        .collect()
}

/// Whether apps see `file`: it sits in `dir` itself or a symlink there points at it.
pub(crate) fn is_registered(dir: &std::path::Path, file: &std::path::Path) -> bool {
    let inside = canon(file).parent() == Some(canon(dir).as_path()) && !is_default_name(file);
    inside || !links_to(dir, file).is_empty()
}

/// Registers `file` for every app: a symlink `<stem>.conf` in `dir` pointing at it. An old symlink
/// of that name is replaced; a regular file of that name is never overwritten.
pub(crate) fn register(dir: &std::path::Path, file: &std::path::Path) -> Result<PathBuf, String> {
    if is_registered(dir, file) {
        return Err("already registered".into());
    }
    let stem = file
        .file_stem()
        .ok_or_else(|| "the file has no name".to_string())?
        .to_string_lossy()
        .into_owned();
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let link = dir.join(format!("{stem}.conf"));
    if std::fs::symlink_metadata(&link).is_ok() {
        if !is_link(&link) {
            return Err(format!("{} is a regular file: not overwritten", link.display()));
        }
        std::fs::remove_file(&link).map_err(|e| format!("{}: {e}", link.display()))?;
    }
    std::os::unix::fs::symlink(canon(file), &link).map_err(|e| format!("{}: {e}", link.display()))?;
    Ok(link)
}

/// Removes the symlinks in `dir` that register `file`; never a regular file.
pub(crate) fn unregister(dir: &std::path::Path, file: &std::path::Path) -> Result<usize, String> {
    let links = links_to(dir, file);
    if links.is_empty() {
        return Err("not registered by a symlink".into());
    }
    for l in &links {
        std::fs::remove_file(l).map_err(|e| format!("{}: {e}", l.display()))?;
    }
    Ok(links.len())
}

/// Makes `file` the one apps use without `IAD_THEME`: `default.conf` in `dir` points at it.
pub(crate) fn set_default(dir: &std::path::Path, file: &std::path::Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let link = dir.join("default.conf");
    if std::fs::symlink_metadata(&link).is_ok() {
        if !is_link(&link) {
            return Err(format!("{} is a regular file: not overwritten", link.display()));
        }
        std::fs::remove_file(&link).map_err(|e| format!("{}: {e}", link.display()))?;
    }
    std::os::unix::fs::symlink(canon(file), &link).map_err(|e| format!("{}: {e}", link.display()))?;
    Ok(link)
}

/// The theme files available, each once: the registered ones (the files and symlinks in the data
/// dirs `dirs`), then those in the folders the user added, then the recently opened ones.
pub(crate) fn list_presets(dirs: &[PathBuf], folders: &[PathBuf], recent: &[PathBuf]) -> Vec<Preset> {
    let default = dirs.iter().find_map(|d| default_target(d));
    let mut out: Vec<Preset> = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut add = |group: &'static str, files: Vec<PathBuf>| {
        for f in files {
            let key = canon(&f);
            if seen.contains(&key) {
                continue;
            }
            out.push(Preset {
                group,
                name: theme_name(&f),
                target: is_link(&f).then(|| key.clone()),
                is_default: default.as_ref() == Some(&key),
                path: f,
            });
            seen.push(key);
        }
    };
    add(
        "Registered",
        dirs.iter()
            .flat_map(|d| scan_folder(d))
            .filter(|p| !is_default_name(p))
            .collect(),
    );
    if let Some(d) = &default {
        add("Registered", vec![d.clone()]);
    }
    for folder in folders {
        add("Folders", scan_folder(folder));
    }
    add("Recent", recent.iter().filter(|p| p.is_file()).cloned().collect());
    out
}

/// What the registering buttons do.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Registry {
    Register,
    Unregister,
    Default,
}

