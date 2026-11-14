//! Theme toys: one small window per design language (Windows 95, XP, 7, classic Mac OS, GNOME 2),
//! drawn with the library's widgets and styled only by the theme in effect (`IAD_THEME=<file>`;
//! with none, the bare look). The theme editor launches them to try a theme.
//!
//! ```text
//! theme-toys <win95|winxp|win7|mac|gnome2>
//! theme-toys --list
//! ```

mod common;
mod gnome2;
mod mac;
mod win7;
mod win95;
mod winxp;

use common::{El, Msg, Toy};
use iced::{Subscription, Task};
use iced_adaptive_design as iad;

/// The designs, as `(name, window title, what it shows)`.
pub const DESIGNS: [(&str, &str, &str); 5] = [
    ("win95", win95::TITLE, "Windows 95: Display Properties"),
    ("winxp", winxp::TITLE, "Windows XP: My Computer"),
    ("win7", win7::TITLE, "Windows 7: Explorer"),
    ("mac", mac::TITLE, "Classic Mac OS: System Preferences"),
    ("gnome2", gnome2::TITLE, "GNOME 2: file browser"),
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
        [a] => DESIGNS
            .iter()
            .position(|(n, ..)| n == a)
            .map_or(Cli::Usage, Cli::Run),
        _ => Cli::Usage,
    }
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
    // The look is the theme in effect: IAD_THEME, else an installed default.conf, else bare.
    iad::config::install(iad::config::Config::load());
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
            size: iced::Size::new(900.0, 640.0),
            ..iad::windows::settings_for(&app_id)
        })
        .settings(iced::Settings {
            antialiasing: true,
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
                toy: Toy::new(),
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
            3 => mac::view(&self.toy),
            _ => gnome2::view(&self.toy),
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
        assert_eq!(parse(&args(&["gnome2"])), Cli::Run(4));
        assert_eq!(parse(&args(&["beos"])), Cli::Usage);
        assert_eq!(parse(&args(&[])), Cli::Usage);
        assert_eq!(parse(&args(&["win95", "x"])), Cli::Usage);
        assert!(usage().contains("win95|winxp|win7|mac|gnome2"));
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
                iad::windows::State { maximized: true, fullscreen: false },
            ] {
                app.toy.win = state;
                let _ = app.view();
            }
            assert!(!name.is_empty());
        }
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
