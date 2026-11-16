//! Theme toys: one small window per design language (Windows 95, XP, 7, MATE 1.26),
//! drawn with the library's widgets and styled only by the theme in effect (`IAD_THEME=<file>`;
//! with none, the bare look).
//!
//! ```text
//! theme-toys <win95|winxp|win7|mate>
//! theme-toys --list
//! ```

// DECISION (user, 2026-11): canonical targets are Windows Classic/95, XP, Win7 and modern MATE.
// macOS is dropped entirely; MATE replaces GNOME2 (`gnome2` stays only as a compatibility
// alias, not a duplicate target). Toys may opt out of desktop styling with their own per-design
// base. Revisit if another target is added.
mod common;
mod design;
#[path = "gnome2.rs"]
mod mate;
mod references;
mod win7;
mod win95;
mod winxp;

use common::{El, Msg, Toy};
use iced::{Subscription, Task};
use iced_adaptive_design as iad;

/// The designs, as `(name, window title, what it shows)`.
pub const DESIGNS: [(&str, &str, &str); 4] = [
    ("win95", win95::TITLE, "Windows Classic: Display Properties"),
    ("winxp", winxp::TITLE, "Windows XP: My Computer"),
    ("win7", win7::TITLE, "Windows 7: Start menu"),
    (
        "mate",
        mate::TITLE,
        "MATE 1.26: Pluma and Desktop User Guide",
    ),
];

/// What the command line asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    List,
    Run(usize),
    Usage,
}

/// Parses the arguments after the program name.
pub fn parse(args: &[String]) -> Cli {
    match args {
        [a] if a == "--list" => Cli::List,
        [a] if a == "gnome2" => Cli::Run(3),
        [a] => DESIGNS
            .iter()
            .position(|(n, ..)| n == a)
            .map_or(Cli::Usage, Cli::Run),
        _ => Cli::Usage,
    }
}

fn initial_size(design: usize) -> iced::Size {
    design::size(design)
}

fn usage() -> String {
    let names: Vec<_> = DESIGNS.iter().map(|(n, ..)| *n).collect();
    format!("usage: theme-toys <{}> | --list", names.join("|"))
}

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let design = match parse(&args) {
        Cli::List => {
            for (name, _, what) in DESIGNS {
                println!("{name}\t{what}");
            }
            return Ok(());
        }
        Cli::Usage => {
            eprintln!("{}", usage());
            std::process::exit(2);
        }
        Cli::Run(i) => i,
    };
    // Independent opt-out base, with the editor-provided IAD_THEME applied above it.
    iad::config::install(iad::config::Config::default().with_theme_config(&design::config(design)));
    // `TOYS_SCHEME=light|dark` pins the scheme; the old era looks are light, so the default
    // for a toy is the portal's.
    match std::env::var("TOYS_SCHEME").ok().as_deref() {
        Some("dark") => {
            iad::utils::set_scheme_override(Some(iad::Scheme::Dark));
        }
        Some("light") => {
            iad::utils::set_scheme_override(Some(iad::Scheme::Light));
        }
        _ if std::env::var_os("TOYS_DARK").is_some() => {
            iad::utils::set_scheme_override(Some(iad::Scheme::Dark));
        }
        _ => {}
    }
    iad::dev::Switches::from_env().apply();
    let app_id = format!("org.iad.ThemeToys.{}", DESIGNS[design].0);
    iced::application(move || App::new(design), App::update, App::view)
        .title(move |_: &App| DESIGNS[design].1.to_string())
        .theme(App::theme)
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size: initial_size(design),
            ..iad::windows::settings_for(&app_id)
        })
        .settings(iced::Settings {
            antialiasing: true,
            default_text_size: iad::Typography::desktop().body.into(),
            ..Default::default()
        })
        .run()
}

struct App {
    design: usize,
    toy: Toy,
    theme: iad::Theme,
}

impl App {
    fn new(design: usize) -> (Self, Task<Msg>) {
        let boot = if iad::dev::Switches::from_env().screenshot.is_some() {
            iced::window::latest().map(Msg::Shot)
        } else {
            Task::none()
        };
        (
            Self {
                design,
                toy: {
                    let mut toy = Toy::new();
                    if design == 0 {
                        toy.tab = 1;
                        toy.pick = 0;
                    }
                    if design == 1 {
                        toy.pick = usize::MAX;
                        toy.open[2] = true;
                    }
                    if design == 2 {
                        toy.pick = 19;
                    }
                    toy
                },
                theme: iad::utils::theme(),
            },
            boot,
        )
    }

    fn theme(&self) -> iad::Theme {
        self.theme.clone()
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Window(action) => {
                return iced::window::oldest().and_then(move |id| action.perform(id));
            }
            Msg::WindowState(state) => self.toy.win = state,
            Msg::Scheme => self.theme = iad::utils::theme(),
            Msg::Resized => return iad::windows::refresh().map(Msg::WindowState),
            Msg::Shot(Some(id)) => {
                if let Some(shot) = iad::dev::Switches::from_env().capture(id) {
                    return shot.map(Msg::Captured);
                }
            }
            Msg::Captured(done) => {
                if let Err(e) = &done {
                    eprintln!("screenshot: {e}");
                }
                return iced::exit();
            }
            other => self.toy.update(other),
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            iad::windows::changes().map(|()| Msg::Resized),
            iad::utils::follow_theme().map(|_| Msg::Scheme),
        ])
    }

    fn view(&self) -> El<'_> {
        match self.design {
            0 => win95::view(&self.toy),
            1 => winxp::view(&self.toy),
            2 => win7::view(&self.toy),
            _ => mate::view(&self.toy),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_command_line_picks_a_design_lists_or_asks_for_usage() {
        assert_eq!(parse(&args(&["--list"])), Cli::List);
        assert_eq!(parse(&args(&["win95"])), Cli::Run(0));
        assert_eq!(parse(&args(&["gnome2"])), Cli::Run(3));
        assert_eq!(parse(&args(&["mate"])), Cli::Run(3));
        assert_eq!(parse(&args(&["mac"])), Cli::Usage);
        assert!(
            !DESIGNS
                .iter()
                .any(|(key, ..)| matches!(*key, "mac" | "gnome2"))
        );
        assert_eq!(parse(&args(&["beos"])), Cli::Usage);
        assert_eq!(parse(&args(&[])), Cli::Usage);
        assert_eq!(parse(&args(&["win95", "x"])), Cli::Usage);
        assert!(usage().contains("win95|winxp|win7|mate"));
    }

    #[test]
    fn every_design_builds_its_view_in_every_state() {
        for (i, (name, ..)) in DESIGNS.iter().enumerate() {
            let mut app = App::new(i).0;
            let _ = app.view();
            app.toy.dialog = true;
            app.toy.toast = Some("hello".into());
            app.toy.tab = 1;
            for state in [
                iad::windows::State::default(),
                iad::windows::State {
                    maximized: true,
                    fullscreen: false,
                },
            ] {
                app.toy.win = state;
                let _ = app.view();
            }
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn apply_and_discard_restore_the_last_applied_settings() {
        let mut t = Toy::new();
        t.update(Msg::Pick(4));
        t.update(Msg::Check(1, true));
        t.update(Msg::Drop(0, "Mystify".into()));
        t.update(Msg::Slide(0, 82.0));
        t.update(Msg::Radio(2));
        t.update(Msg::Apply);
        t.update(Msg::Pick(0));
        t.update(Msg::Check(1, false));
        t.update(Msg::Drop(0, "3D Pipes".into()));
        t.update(Msg::Slide(0, 10.0));
        t.update(Msg::Radio(0));
        t.update(Msg::Dialog(true));
        t.update(Msg::Discard);
        assert_eq!(t.pick, 4);
        assert!(t.checks[1]);
        assert_eq!(t.drops[0], "Mystify");
        assert_eq!(t.sliders[0], 82.0);
        assert_eq!(t.radio, 2);
        assert!((t.progress - 0.82).abs() < 0.001);
        assert!(!t.dialog);
    }

    #[test]
    fn shipped_reference_themes_resolve_without_invalid_values() {
        for (name, text) in [
            ("win95", include_str!("../../themes/win95.ron")),
            ("winxp", include_str!("../../themes/winxp.ron")),
            ("win7", include_str!("../../themes/win7.ron")),
            ("mate", include_str!("../../themes/mate.ron")),
            ("gnome2", include_str!("../../themes/gnome2.ron")),
        ] {
            let config = design::resolve(
                match name {
                    "win95" => 0,
                    "winxp" => 1,
                    "win7" => 2,
                    _ => 3,
                },
                text,
            );
            assert!(
                config.warnings().is_empty(),
                "{name}: {:?}",
                config.warnings()
            );
            assert_eq!(config.text("theme", "key"), Some(name));
            let expected_height = match name {
                "win95" => 20.0,
                "winxp" | "mate" | "gnome2" => 22.0,
                _ => 24.0,
            };
            assert_eq!(config.number("tree", "row-height"), Some(expected_height));
            assert_eq!(config.number("tree", "icon-size"), Some(16.0));
            assert!(config.color("button.opaque", "bg").is_some(), "{name}");
        }
    }

    #[test]
    fn search_is_literal_case_insensitive_and_trims_whitespace() {
        assert!(common::matches_query("Report.docx", " DOCX "));
        assert!(common::matches_query("Desktop", ""));
        assert!(!common::matches_query("Report.docx", ".*"));
        assert!(!common::matches_query("Music", "Pictures"));
    }

    #[test]
    fn scene_views_and_empty_search_results_build() {
        let mut t = Toy::new();
        t.update(Msg::Text("no matching file or preference".into()));
        let _ = win7::view(&t);
        let _ = mate::view(&t);
        // Out-of-range control messages must not corrupt another control.
        let checks = t.checks;
        t.update(Msg::Check(99, false));
        t.update(Msg::Slide(99, 100.0));
        t.update(Msg::Drop(99, "ignored".into()));
        assert_eq!(checks, t.checks);
    }

    #[test]
    fn reference_sizes_and_win95_preview_states_build() {
        assert_eq!(initial_size(0), iced::Size::new(404.0, 448.0));
        assert!(initial_size(2).width > initial_size(0).width);
        assert_eq!(initial_size(3), iced::Size::new(1280.0, 675.0));
        let mut t = Toy::new();
        for wallpaper in 0..5 {
            for placement in 0..3 {
                t.update(Msg::Pick(wallpaper));
                t.update(Msg::Radio(placement));
                let _ = win95::view(&t);
            }
        }
        for tab in 0..3 {
            t.update(Msg::Tab(tab));
            let _ = win95::view(&t);
        }
    }

    #[test]
    fn unix_places_have_independent_selection_and_expansion() {
        use iad::primitives::tree::{Event, TreeNode};
        fn assert_unix(nodes: &[TreeNode]) {
            for node in nodes {
                assert!(!node.label.contains("C:"));
                assert!(!node.label.contains("D:"));
                assert!(!node.label.contains("Local Disk"));
                assert_unix(&node.children);
            }
        }
        let mut t = Toy::new();
        assert_unix(&t.places);
        assert_eq!(
            t.places
                .iter()
                .map(|n| n.label.as_str())
                .collect::<Vec<_>>(),
            ["Home", "Desktop", "File System", "Network"]
        );
        assert!(t.places_state.is_expanded("~"));
        t.update(Msg::Places(Event::Toggle("/".into())));
        assert!(t.places_state.is_expanded("/"));
        t.update(Msg::Places(Event::Click("/etc".into())));
        assert_eq!(t.places_state.selected(), &["/etc".to_string()]);
        assert_eq!(t.status, "Location: /etc");
        assert!(t.nav_state.selected().is_empty());
        assert!(t.nav_state.is_expanded("Local Disk (C:)"));
        t.update(Msg::Tree(Event::Click("Documents".into())));
        assert_eq!(t.places_state.selected(), &["/etc".to_string()]);
        let _ = mate::view(&t);
    }

    #[test]
    fn document_history_start_menu_and_preview_window_controls_work() {
        let mut toy = Toy::new();
        let initial = toy.document.text();
        toy.update(Msg::Edit(iad::components::TextAction::Edit(
            iced::widget::text_editor::Edit::Insert('X'),
        )));
        let changed = toy.document.text();
        assert_ne!(changed, initial);
        toy.update(Msg::UndoDocument);
        assert_eq!(toy.document.text(), initial);
        toy.update(Msg::RedoDocument);
        assert_eq!(toy.document.text(), changed);
        toy.update(Msg::SaveDocument);
        assert!(toy.status.contains("no disk write"));
        assert_eq!(toy.saved_document, changed);
        toy.update(Msg::Launch(19));
        assert_eq!(toy.pick, 19);
        assert!(toy.status.contains("Word"));
        assert_eq!(win7::filtered_programs("  word ").count(), 1);
        toy.update(Msg::Dialog(true));
        toy.update(Msg::Start(false));
        assert!(!toy.start_open && !toy.dialog);
        toy.update(Msg::Start(true));
        assert!(toy.start_open);
        for index in 0..2 {
            toy.update(Msg::Open(index));
            assert!(!toy.open[index]);
            let _ = mate::view(&toy);
            toy.update(Msg::Open(index));
            toy.update(Msg::Check(4 + index, true));
            let _ = mate::view(&toy);
            toy.update(Msg::Check(4 + index, false));
        }
        toy.update(Msg::BrowseWallpaper);
        assert!(toy.dialog && toy.section == 1);
        let _ = win95::view(&toy);
        toy.update(Msg::CancelDisplay);
        assert_eq!(toy.section, 0);
    }

    #[test]
    fn the_toy_state_updates() {
        let mut t = Toy::new();
        t.update(Msg::Check(0, false));
        t.update(Msg::Pick(3));
        t.update(Msg::Slide(0, 80.0));
        assert!(!t.checks[0] && t.pick == 3 && t.progress > 0.7);
    }
}
