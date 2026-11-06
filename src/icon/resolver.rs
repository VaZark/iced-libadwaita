//! Metadata-driven freedesktop icon theme lookup.
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

/// A resolved installed asset. Missing or unreadable files return `None`.
#[derive(Debug, Clone)]
pub struct ResolvedIcon {
    pub path: PathBuf,
    pub bytes: Arc<[u8]>,
    pub symbolic: bool,
}

type Key = (String, u32, u32, bool);
#[derive(Default)]
struct Cache {
    theme: String,
    entries: HashMap<Key, Option<ResolvedIcon>>,
}
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
impl Cache {
    fn get(
        &mut self,
        theme: &str,
        key: Key,
        lookup: impl FnOnce() -> Option<ResolvedIcon>,
    ) -> Option<ResolvedIcon> {
        if self.theme != theme {
            self.theme = theme.into();
            self.entries.clear();
        }
        self.entries.entry(key).or_insert_with(lookup).clone()
    }
}

/// Resolve a name at a logical pixel size and integer output scale.
/// Symbolic requests prefer `name-symbolic`, then the regular name.
pub fn resolve(name: &str, size: u32, scale: u32, symbolic: bool) -> Option<ResolvedIcon> {
    let theme = crate::system::icon_theme();
    let key = (name.to_owned(), size.max(1), scale.max(1), symbolic);
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    cache.get(&theme, key, || {
        lookup(&roots(), &theme, name, size.max(1), scale.max(1), symbolic)
    })
}

/// Installed theme identifiers with readable `index.theme` metadata, sorted and deduplicated.
/// This discovers available themes; it does not choose the desktop's active theme.
pub fn available_themes() -> Vec<String> {
    discover_themes(&roots())
}

fn discover_themes(roots: &[PathBuf]) -> Vec<String> {
    let mut themes = std::collections::BTreeSet::new();
    for root in roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if valid(name) && metadata(&entry.path().join("index.theme")).is_some() {
                    themes.insert(name.to_owned());
                }
            }
        }
    }
    themes.into_iter().collect()
}

fn roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".icons"));
    }
    if let Some(data) = std::env::var_os("XDG_DATA_HOME").filter(|p| Path::new(p).is_absolute()) {
        roots.push(PathBuf::from(data).join("icons"));
    } else if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join(".local/share/icons"));
    }
    for dir in std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into())
        .split(':')
    {
        if Path::new(dir).is_absolute() {
            roots.push(Path::new(dir).join("icons"));
        }
    }
    roots.push("/usr/share/pixmaps".into());
    roots
}

fn valid(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

type Section = HashMap<String, String>;
fn metadata(path: &Path) -> Option<HashMap<String, Section>> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut sections: HashMap<String, Section> = HashMap::new();
    let mut section = String::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|s| !s.starts_with('#') && !s.is_empty())
    {
        if let Some(s) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = s.into();
        } else if let Some((k, v)) = line.split_once('=') {
            sections
                .entry(section.clone())
                .or_default()
                .insert(k.trim().into(), v.trim().into());
        }
    }
    sections.contains_key("Icon Theme").then_some(sections)
}
fn list(s: Option<&String>) -> Vec<String> {
    s.into_iter()
        .flat_map(|s| s.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
fn number(s: &Section, k: &str, default: u32) -> u32 {
    s.get(k).and_then(|v| v.parse().ok()).unwrap_or(default)
}
fn distance(s: &Section, size: u32, scale: u32) -> (bool, u64) {
    let nominal = number(s, "Size", 0) as u64;
    let ds = number(s, "Scale", 1).max(1) as u64;
    let (min, max) = match s.get("Type").map(String::as_str).unwrap_or("Threshold") {
        "Fixed" => (nominal, nominal),
        "Scalable" => (
            number(s, "MinSize", nominal as u32) as u64,
            number(s, "MaxSize", nominal as u32) as u64,
        ),
        _ => {
            let t = number(s, "Threshold", 2) as u64;
            (nominal.saturating_sub(t), nominal + t)
        }
    };
    let exact = ds == scale as u64 && (min..=max).contains(&(size as u64));
    let target = size as u64 * scale as u64;
    (
        exact,
        if target < min * ds {
            min * ds - target
        } else {
            target.saturating_sub(max.saturating_mul(ds))
        },
    )
}
fn file(dir: &Path, name: &str) -> Option<ResolvedIcon> {
    for ext in ["png", "svg", "xpm"] {
        let path = dir.join(format!("{name}.{ext}"));
        if let Ok(bytes) = std::fs::read(&path) {
            return Some(ResolvedIcon {
                path,
                bytes: bytes.into(),
                symbolic: name.ends_with("-symbolic"),
            });
        }
    }
    None
}
fn theme_lookup(
    roots: &[PathBuf],
    theme: &str,
    name: &str,
    size: u32,
    scale: u32,
    seen: &mut HashSet<String>,
) -> Option<ResolvedIcon> {
    if !valid(theme) || !seen.insert(theme.into()) {
        return None;
    }
    let meta = roots
        .iter()
        .find_map(|r| metadata(&r.join(theme).join("index.theme")))?;
    let header = &meta["Icon Theme"];
    let mut dirs = list(header.get("Directories"));
    for d in list(header.get("ScaledDirectories")) {
        if !dirs.contains(&d) {
            dirs.push(d);
        }
    }
    let mut nearest: Option<(u64, ResolvedIcon)> = None;
    for dir in dirs {
        if Path::new(&dir).is_absolute()
            || Path::new(&dir)
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            continue;
        }
        let Some(section) = meta.get(&dir) else {
            continue;
        };
        let (exact, distance) = distance(section, size, scale);
        for root in roots {
            if let Some(icon) = file(&root.join(theme).join(&dir), name) {
                if exact {
                    return Some(icon);
                }
                if nearest.as_ref().is_none_or(|(d, _)| distance < *d) {
                    nearest = Some((distance, icon));
                }
            }
        }
    }
    if let Some((_, icon)) = nearest {
        return Some(icon);
    }
    for parent in list(header.get("Inherits")) {
        if let Some(icon) = theme_lookup(roots, &parent, name, size, scale, seen) {
            return Some(icon);
        }
    }
    None
}

pub(super) fn lookup(
    roots: &[PathBuf],
    theme: &str,
    name: &str,
    size: u32,
    scale: u32,
    symbolic: bool,
) -> Option<ResolvedIcon> {
    if !valid(name) {
        return None;
    }
    let requested_symbolic = symbolic || name.ends_with("-symbolic");
    let regular = name.strip_suffix("-symbolic").unwrap_or(name);
    let names = if symbolic || name.ends_with("-symbolic") {
        vec![format!("{regular}-symbolic"), regular.into()]
    } else {
        vec![name.into()]
    };
    for name in names {
        let mut seen = HashSet::new();
        if let Some(mut icon) = theme_lookup(roots, theme, &name, size, scale, &mut seen)
            .or_else(|| theme_lookup(roots, "hicolor", &name, size, scale, &mut seen))
            .or_else(|| roots.iter().find_map(|r| file(r, &name)))
        {
            // A regular-name fallback still fulfills a symbolic rendering request.
            icon.symbolic = requested_symbolic;
            return Some(icon);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Fixture {
        base: PathBuf,
        roots: Vec<PathBuf>,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let base = std::env::temp_dir().join(format!(
                "iad-icons-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let roots = vec![base.join("user"), base.join("system"), base.join("pixmaps")];
            Self { base, roots }
        }
        fn write(&self, root: usize, path: &str, data: &str) -> PathBuf {
            let path = self.roots[root].join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, data).unwrap();
            path
        }
        fn theme(&self, name: &str, parents: &str) {
            self.write(0,&format!("{name}/index.theme"),&format!("[Icon Theme]\nName={name}\nDirectories=16/actions,32/actions,scalable/actions\nScaledDirectories=16@2/actions\nInherits={parents}\n[16/actions]\nSize=16\nType=Fixed\n[32/actions]\nSize=32\nType=Fixed\n[scalable/actions]\nSize=48\nType=Scalable\nMinSize=40\nMaxSize=64\n[16@2/actions]\nSize=16\nScale=2\nType=Fixed\n"));
        }
        fn get(&self, name: &str, size: u32, scale: u32, symbolic: bool) -> Option<ResolvedIcon> {
            lookup(&self.roots, "selected", name, size, scale, symbolic)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }
    #[test]
    fn discovers_installed_themes_without_duplicates_or_icon_files() {
        let f = Fixture::new();
        f.theme("selected", "");
        f.theme("hicolor", "");
        f.write(1, "selected/index.theme", "[Icon Theme]\nName=selected\n");
        f.write(0, "unindexed/test.svg", "not a theme");
        f.write(0, "bad/index.theme", "invalid metadata");
        assert_eq!(discover_themes(&f.roots), vec!["hicolor", "selected"]);
    }

    #[test]
    fn theme_change_invalidates_positive_and_negative_cache_entries() {
        let f = Fixture::new();
        f.theme("selected", "");
        f.theme("other", "");
        f.write(0, "selected/16/actions/test.svg", "first");
        f.write(0, "other/16/actions/test.svg", "second");
        f.write(0, "other/16/actions/missing.svg", "now available");
        let mut cache = Cache::default();
        let key = ("test".to_owned(), 16, 1, false);
        let first = cache
            .get("selected", key.clone(), || f.get("test", 16, 1, false))
            .unwrap();
        assert_eq!(first.bytes.as_ref(), b"first");
        assert!(
            cache
                .get("selected", ("missing".into(), 16, 1, false), || None)
                .is_none()
        );
        cache.get("selected", key.clone(), || {
            panic!("cache hit must not perform I/O")
        });
        let second = cache
            .get("other", key, || {
                lookup(&f.roots, "other", "test", 16, 1, false)
            })
            .unwrap();
        assert_eq!(second.bytes.as_ref(), b"second");
        assert_eq!(cache.entries.len(), 1);
        assert!(
            cache
                .get("other", ("missing".into(), 16, 1, false), || lookup(
                    &f.roots, "other", "missing", 16, 1, false
                ))
                .is_some()
        );
    }

    #[test]
    fn precedence_split_themes_and_size_scale_selection() {
        let f = Fixture::new();
        f.theme("selected", "parent");
        f.theme("parent", "");
        let user = f.write(0, "selected/16/actions/test.svg", "user");
        f.write(1, "selected/16/actions/test.svg", "system");
        let large = f.write(1, "selected/32/actions/test.png", "large");
        let scaled = f.write(1, "selected/16@2/actions/test.svg", "scaled");
        let vector = f.write(1, "selected/scalable/actions/test.svg", "vector");
        f.write(0, "parent/16/actions/test.svg", "parent");
        assert_eq!(f.get("test", 16, 1, false).unwrap().path, user);
        assert_eq!(f.get("test", 30, 1, false).unwrap().path, large);
        assert_eq!(f.get("test", 16, 2, false).unwrap().path, scaled);
        assert_eq!(f.get("test", 56, 1, false).unwrap().path, vector);
    }
    #[test]
    fn ordered_depth_first_inheritance_cycles_and_fallbacks() {
        let f = Fixture::new();
        f.theme("selected", "first,second");
        f.theme("first", "grandparent");
        f.theme("grandparent", "selected");
        f.theme("second", "");
        f.theme("hicolor", "");
        let first = f.write(0, "grandparent/16/actions/test.svg", "grandparent");
        f.write(0, "second/16/actions/test.svg", "second");
        assert_eq!(f.get("test", 16, 1, false).unwrap().path, first);
        let hc = f.write(0, "hicolor/16/actions/fallback.png", "fallback");
        assert_eq!(f.get("fallback", 16, 1, false).unwrap().path, hc);
        let unthemed = f.write(2, "loose.xpm", "unthemed");
        assert_eq!(f.get("loose", 16, 1, false).unwrap().path, unthemed);
        assert!(f.get("absent", 16, 1, false).is_none());
        assert!(f.get("../test", 16, 1, false).is_none());
    }
    #[test]
    fn symbolic_preference_and_regular_fallback() {
        let f = Fixture::new();
        f.theme("selected", "parent");
        f.theme("parent", "");
        let regular = f.write(0, "selected/16/actions/test.svg", "regular");
        let fallback = f.get("test", 16, 1, true).unwrap();
        assert_eq!(fallback.path, regular);
        assert!(fallback.symbolic);
        assert!(!f.get("test", 16, 1, false).unwrap().symbolic);
        let symbolic = f.write(0, "parent/16/actions/test-symbolic.svg", "symbolic");
        let icon = f.get("test", 16, 1, true).unwrap();
        assert!(icon.symbolic);
        assert_eq!(icon.path, symbolic);
        assert_eq!(f.get("test-symbolic", 16, 1, true).unwrap().path, symbolic);
        assert_eq!(f.get("test", 16, 1, false).unwrap().path, regular);
        assert!(f.get("missing-symbolic", 16, 1, true).is_none());
    }
    #[test]
    fn threshold_directory_defaults_and_unreadable_files() {
        let f = Fixture::new();
        f.write(0,"selected/index.theme","[Icon Theme]\nDirectories=small,large\n[small]\nSize=16\n[large]\nSize=20\nType=Fixed\n");
        let small = f.write(0, "selected/small/test.svg", "small");
        f.write(0, "selected/large/test.svg", "large");
        assert_eq!(f.get("test", 18, 1, false).unwrap().path, small);
        std::fs::create_dir_all(f.roots[0].join("selected/small/unreadable.png")).unwrap();
        let fallback = f.write(1, "selected/small/unreadable.svg", "readable");
        assert_eq!(f.get("unreadable", 16, 1, false).unwrap().path, fallback);
    }
}
