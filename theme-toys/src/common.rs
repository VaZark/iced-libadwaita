//! What every toy shares: one state, one message type and a few layout helpers. A toy window is
//! drawn with the library's widgets and carries **no colors of its own**: the theme in effect
//! (`IAD_THEME`) decides every surface, border, radius, padding and shadow.

use iad::components::typography as ty;
use iad::primitives::tree::{Event as TreeEvent, TreeNode, TreeState};
use iad::widgets as w;
pub use iad::Element;
use iad::Metrics;
use iced::widget::{container, row, scrollable};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;


pub type El<'a> = Element<'a, Msg>;

/// Every message any toy sends. A toy uses the part it needs.
#[derive(Debug, Clone)]
pub enum Msg {
    Window(iad::windows::Action),
    WindowState(iad::windows::State),
    Scheme,
    Tab(usize),
    Pick(usize),
    Radio(usize),
    Check(usize, bool),
    Slide(usize, f32),
    Text(String),
    Drop(usize, String),
    Section(usize),
    Open(usize),
    Tree(TreeEvent),
    Dialog(bool),
    Toast(Option<String>),
    Status(String),
    Resized,
    Shot(Option<iced::window::Id>),
    Captured(Result<std::path::PathBuf, String>),
}

/// The state of every toy: small, flat and shared, so a design only reads what it draws.
pub struct Toy {
    pub win: iad::windows::State,
    pub tab: usize,
    pub pick: usize,
    pub radio: usize,
    pub checks: [bool; 6],
    pub sliders: [f32; 3],
    pub text: String,
    pub drops: [String; 2],
    pub section: usize,
    pub open: [bool; 4],
    pub nav: Vec<TreeNode>,
    pub nav_state: TreeState,
    pub dialog: bool,
    pub toast: Option<String>,
    pub status: String,
    pub progress: f32,
}

impl Toy {
    pub fn new() -> Self {
        let node = |v: &str, kids: Vec<TreeNode>| TreeNode::branch(v, v, kids);
        let leaf = |v: &str| TreeNode::new(v, v);
        let nav = vec![
            node(
                "Computer",
                vec![
                    node("Local Disk (C:)", vec![leaf("Documents"), leaf("Pictures"), leaf("Music")]),
                    leaf("CD Drive (D:)"),
                ],
            ),
            node("Favorites", vec![leaf("Desktop"), leaf("Downloads")]),
            leaf("Network"),
        ];
        let mut nav_state = TreeState::new();
        nav_state.set_expanded(["Computer".to_string(), "Local Disk (C:)".to_string()]);
        Self {
            win: iad::windows::State::default(),
            tab: 0,
            pick: 1,
            radio: 0,
            checks: [true, false, true, false, false, false],
            sliders: [40.0, 70.0, 25.0],
            text: String::new(),
            drops: ["Windows Default".into(), "Medium".into()],
            section: 0,
            open: [true, true, false, false],
            nav,
            nav_state,
            dialog: false,
            toast: None,
            status: "Ready".into(),
            progress: 0.62,
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Tab(i) => self.tab = i,
            Msg::Pick(i) => self.pick = i,
            Msg::Radio(i) => self.radio = i,
            Msg::Check(i, on) => {
                if let Some(c) = self.checks.get_mut(i) {
                    *c = on;
                }
            }
            Msg::Slide(i, v) => {
                if let Some(s) = self.sliders.get_mut(i) {
                    *s = v;
                }
                self.progress = (self.sliders[0] / 100.0).clamp(0.0, 1.0);
            }
            Msg::Text(t) => self.text = t,
            Msg::Drop(i, v) => {
                if let Some(d) = self.drops.get_mut(i) {
                    *d = v;
                }
            }
            Msg::Section(i) => self.section = i,
            Msg::Open(i) => {
                if let Some(o) = self.open.get_mut(i) {
                    *o = !*o;
                }
            }
            Msg::Tree(e) => {
                let _ = self.nav_state.apply(&mut self.nav, e);
            }
            Msg::Dialog(open) => self.dialog = open,
            Msg::Toast(t) => self.toast = t,
            Msg::Status(s) => self.status = s,
            Msg::Window(_)
            | Msg::WindowState(_)
            | Msg::Scheme
            | Msg::Resized
            | Msg::Shot(_)
            | Msg::Captured(_) => {}
        }
    }
}

/// A line of text.
pub fn label<'a>(s: impl Into<String>) -> El<'a> {
    ty::body(s.into()).into()
}

/// A dim line of text.
pub fn caption<'a>(s: impl Into<String>) -> El<'a> {
    ty::caption(s.into()).into()
}

/// A bold section heading.
pub fn heading<'a>(s: impl Into<String>) -> El<'a> {
    ty::heading(s.into()).into()
}

/// A status bar: one line under the window content, in the theme's header surface.
pub fn status_bar<'a>(text: impl Into<String>, right: impl Into<String>) -> El<'a> {
    let m = Metrics::desktop();
    container(
        row![ty::caption(text.into()), iced::widget::space().width(Length::Fill), ty::caption(right.into())]
            .align_y(Alignment::Center),
    )
    .padding([m.space_xxs, m.space_m])
    .width(Length::Fill)
    .style(iad::styles::container::headerbar)
    .into()
}

/// A scrolling pane.
pub fn pane<'a>(content: impl Into<El<'a>>) -> El<'a> {
    scrollable(container(content).padding(Metrics::desktop().inset)).height(Length::Fill).into()
}

/// A tree of `nav` with lines and folder icons.
pub fn nav_tree(t: &Toy) -> El<'_> {
    w::Tree::new(&t.nav, &t.nav_state)
        .lines(true)
        .icons(|n, _| {
            Some(if n.has_children() {
                iad::Icon::Folder
            } else {
                iad::Icon::Document
            })
        })
        .on_event(Msg::Tree)
        .into()
}

/// The window around a toy's `content`: the toolkit's own client-side decorations with a title
/// bar, so the chrome is themed too.
pub fn window<'a>(t: &'a Toy, title: &'a str, content: El<'a>) -> El<'a> {
    let chrome = iad::windows::Chrome::new(t.win);
    let bar = w::header_bar().title(title).chrome(&chrome, Msg::Window);
    let body = iced::widget::column![Element::from(bar), content];
    chrome.frame(body, Msg::Window)
}

/// `content` with the modal `dialog` over it while `open`, and the toast, if any.
pub fn overlays<'a>(t: &'a Toy, content: El<'a>, dialog: El<'a>) -> El<'a> {
    let radius = iad::windows::Chrome::new(t.win).radius();
    let content = w::present_dialog(
        content,
        t.dialog.then_some(dialog),
        Some(Msg::Dialog(false)),
        radius,
    );
    let toast = t
        .toast
        .as_ref()
        .map(|s| w::Toast::new(s.clone()).on_dismiss(Msg::Toast(None)));
    w::toast_overlay(content, toast)
}
