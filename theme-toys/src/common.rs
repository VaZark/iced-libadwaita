//! What every toy shares: one state, one message type and a few layout helpers. A toy window is
//! drawn with the library's widgets and carries **no colors of its own**: the theme in effect
//! (`IAD_THEME`) decides every surface, border, radius, padding and shadow.

pub use iad::Element;
use iad::Metrics;
use iad::components::typography as ty;
use iad::primitives::tree::{Event as TreeEvent, TreeNode, TreeState};
use iad::widgets as w;
use iced::widget::{container, row};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

pub type El<'a> = Element<'a, Msg>;

/// Every message any toy sends. A toy uses the part it needs.
#[derive(Debug, Clone)]
pub enum Msg {
    Window(iad::windows::Action),
    WindowState(iad::windows::State),
    Scheme,
    Edit(iad::components::TextAction),
    SaveDocument,
    ChooseDocument,
    LoadDocument(String),
    Find(bool),
    FindQuery(String),
    ShowChild(usize),
    ShowDesktop,
    OpenXpItem(usize),
    SelectAllXp,
    XpProperties(Option<usize>),
    EditorToolbar(bool),
    PinProgram(usize),
    UndoDocument,
    RedoDocument,
    Start(bool),
    Launch(usize),
    Tab(usize),
    Pick(usize),
    Radio(usize),
    Check(usize, bool),
    Slide(usize, f32),
    Text(String),
    Drop(usize, String),
    Section(usize),
    Open(usize),
    #[allow(dead_code)] // Retained for Windows model compatibility tests.
    Tree(TreeEvent),
    Places(TreeEvent),
    Dialog(bool),
    BrowseWallpaper,
    CancelDisplay,
    Toast(Option<String>),
    Status(String),
    Apply,
    Discard,
    Resized,
    Shot(Option<iced::window::Id>),
    Captured(Result<std::path::PathBuf, String>),
}

/// The state of every toy: small, flat and shared, so a design only reads what it draws.
pub struct Toy {
    undo: Vec<String>,
    redo: Vec<String>,
    pub saved_document: String,
    pub find_open: bool,
    pub find_query: String,
    pub pinned_programs: Vec<usize>,
    pub xp_all_selected: bool,
    pub xp_properties: Option<usize>,
    pub editor_toolbar: bool,
    pub active_child: usize,
    pub document: iad::components::TextContent,
    pub start_open: bool,
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
    pub places: Vec<TreeNode>,
    pub places_state: TreeState,
    pub dialog: bool,
    pub toast: Option<String>,
    pub status: String,
    pub progress: f32,
    saved: ([bool; 6], [f32; 3], [String; 2], usize, usize),
}

impl Toy {
    pub fn new() -> Self {
        let node = |v: &str, kids: Vec<TreeNode>| TreeNode::branch(v, v, kids);
        let leaf = |v: &str| TreeNode::new(v, v);
        let nav = vec![
            node(
                "Computer",
                vec![
                    node(
                        "Local Disk (C:)",
                        vec![leaf("Documents"), leaf("Pictures"), leaf("Music")],
                    ),
                    leaf("CD Drive (D:)"),
                ],
            ),
            node("Favorites", vec![leaf("Desktop"), leaf("Downloads")]),
            leaf("Network"),
        ];
        let mut nav_state = TreeState::new();
        nav_state.set_expanded(["Computer".to_string(), "Local Disk (C:)".to_string()]);
        let places = crate::mate::places();
        let mut places_state = TreeState::new();
        places_state.set_expanded(["~".to_string()]);
        places_state.select("~");
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            saved_document: crate::mate::DOCUMENT.into(),
            find_open: false,
            find_query: String::new(),
            pinned_programs: Vec::new(),
            xp_all_selected: false,
            xp_properties: None,
            editor_toolbar: true,
            active_child: 0,
            document: iad::components::TextContent::with_text(crate::mate::DOCUMENT),
            start_open: true,
            places,
            places_state,
            win: iad::windows::State::default(),
            tab: 0,
            pick: 0,
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
            progress: 0.4,
            saved: (
                [true, false, true, false, false, false],
                [40.0, 70.0, 25.0],
                ["Windows Default".into(), "Medium".into()],
                0,
                0,
            ),
        }
    }

    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Edit(action) => {
                let before = self.document.text();
                self.document.perform(action);
                if self.document.text() != before {
                    self.undo.push(before);
                    self.redo.clear();
                    self.status = "Modified — not saved".into();
                }
            }
            Msg::UndoDocument => {
                if let Some(text) = self.undo.pop() {
                    self.redo.push(self.document.text());
                    self.document = iad::components::TextContent::with_text(&text);
                }
            }
            Msg::RedoDocument => {
                if let Some(text) = self.redo.pop() {
                    self.undo.push(self.document.text());
                    self.document = iad::components::TextContent::with_text(&text);
                }
            }
            Msg::ChooseDocument => self.dialog = true,
            Msg::LoadDocument(text) => {
                let before = self.document.text();
                if before != text {
                    self.undo.push(before);
                    self.redo.clear();
                    self.document = iad::components::TextContent::with_text(&text);
                }
                self.dialog = false;
                self.open[0] = true;
                self.active_child = 0;
                self.status = "Opened bundled document in preview memory".into();
            }
            Msg::Find(open) => self.find_open = open,
            Msg::FindQuery(query) => self.find_query = query,
            Msg::ShowDesktop => {
                self.open[0] = false;
                self.open[1] = false;
            }
            Msg::ShowChild(index) => {
                if index < 2 {
                    self.open[index] = true;
                    self.active_child = index;
                }
            }
            Msg::SelectAllXp => self.xp_all_selected = true,
            Msg::XpProperties(index) => {
                if index.is_none_or(|i| crate::winxp::item_name(i).is_some()) {
                    self.xp_properties = index;
                    self.dialog = true;
                }
            }
            Msg::EditorToolbar(visible) => self.editor_toolbar = visible,
            Msg::OpenXpItem(index) => {
                if let Some(name) = crate::winxp::item_name(index) {
                    self.pick = index;
                    self.text = name.into();
                    self.status = format!("Preview location: {name}");
                }
            }
            Msg::PinProgram(index) => {
                if crate::win7::PROGRAMS.get(index).is_some() {
                    if self.pinned_programs.contains(&index) {
                        self.pinned_programs.retain(|i| *i != index);
                    } else {
                        self.pinned_programs.push(index);
                    }
                    self.status = "Start menu pin updated in preview memory".into();
                }
            }
            Msg::SaveDocument => {
                self.saved_document = self.document.text();
                self.status = "Saved to preview memory (no disk write)".into();
            }
            Msg::Start(open) => {
                self.start_open = open;
                self.dialog = false;
            }
            Msg::Launch(index) => {
                if let Some(name) = crate::win7::PROGRAMS.get(index) {
                    self.pick = index;
                    self.status = format!("Preview launch: {name}");
                }
            }
            Msg::Tab(i) => self.tab = i,
            Msg::Pick(i) => {
                self.pick = i;
                self.xp_all_selected = false;
            }
            Msg::Radio(i) => self.radio = i,
            Msg::Check(i, on) => {
                if i == 4 || i == 5 {
                    self.active_child = i - 4;
                }
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
            Msg::Section(i) => {
                self.section = i;
                self.text.clear();
            }
            Msg::Open(i) => {
                if let Some(o) = self.open.get_mut(i) {
                    *o = !*o;
                }
            }
            Msg::Tree(e) => {
                let _ = self.nav_state.apply(&mut self.nav, e);
            }
            Msg::Places(e) => {
                if let TreeEvent::Click(path) = &e {
                    self.status = format!("Location: {path}");
                }
                let _ = self.places_state.apply(&mut self.places, e);
            }
            Msg::Dialog(open) => self.dialog = open,
            Msg::BrowseWallpaper => {
                self.section = 1;
                self.dialog = true;
            }
            Msg::CancelDisplay => {
                self.section = 0;
                self.dialog = true;
            }
            Msg::Toast(t) => self.toast = t,
            Msg::Status(s) => self.status = s,
            Msg::Apply => {
                self.saved = (
                    self.checks,
                    self.sliders,
                    self.drops.clone(),
                    self.pick,
                    self.radio,
                );
                self.status = "Settings applied".into();
                self.toast = Some("Settings applied to the preview".into());
            }
            Msg::Discard => {
                (self.checks, self.sliders, self.drops, self.pick, self.radio) = self.saved.clone();
                self.progress = self.sliders[0] / 100.0;
                self.dialog = false;
                self.status = "Changes discarded".into();
            }
            Msg::Window(_)
            | Msg::WindowState(_)
            | Msg::Scheme
            | Msg::Resized
            | Msg::Shot(_)
            | Msg::Captured(_) => {}
        }
    }
}

/// Conventional desktop menus keep check marks/icons on the leading edge.
pub fn native_menu(title: &str) -> w::Menu<Msg> {
    w::Menu::new(title)
        .icon_side(w::MenuIconSide::Start)
        .check_style(w::MenuCheckStyle::Mark)
}

/// Compact text trigger; height comes from the scene token, not global button minimums.
pub fn menu_trigger<'a>(title: &str, menu: w::Menu<Msg>, height: f32) -> El<'a> {
    let m = Metrics::desktop();
    w::menu_button(iad::Icon::OpenMenu, menu)
        .text_trigger(title)
        .props(w::MenuProps {
            label: iad::components::TextProps::default().size(iad::Typography::desktop().body),
            trigger: iad::components::ContainerProps::default()
                .height(Length::Fixed(height))
                .padding(iced::Padding::from([0.0, m.space_xs])),
            ..Default::default()
        })
        .presentation(w::MenuPresentation::Dropdown)
        .into()
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
        row![
            ty::caption(text.into()),
            iced::widget::space().width(Length::Fill),
            ty::caption(right.into())
        ]
        .align_y(Alignment::Center),
    )
    .padding([m.space_xxs, m.space_m])
    .width(Length::Fill)
    .style(iad::styles::container::card)
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

/// Case-insensitive literal search shared by file and preference views.
pub fn matches_query(name: &str, query: &str) -> bool {
    name.to_lowercase().contains(&query.trim().to_lowercase())
}

#[cfg(test)]
pub mod menu_tests {
    use super::*;
    use iced::advanced::{Shell, layout, renderer::Headless, widget::Tree};
    use iced::{Event, Rectangle, Size, keyboard, mouse};

    fn key(named: keyboard::key::Named) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named),
            modified_key: keyboard::Key::Named(named),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::empty(),
            text: None,
            repeat: false,
        })
    }

    /// Open the real wrapper, check containment, and activate its enabled row by keyboard.
    /// All fixture rectangles come from the scene/schema rather than unrelated test sizes.
    pub fn activate(mut element: El<'_>, design: usize, context: bool, row: usize) -> (Size, Msg) {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(iad::Typography::desktop().body),
            Some("tiny-skia"),
        ))
        .unwrap();
        let viewport = Rectangle::with_size(crate::design::size(design));
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let size = node.size();
        let mut messages = Vec::new();
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(if context {
                mouse::Button::Right
            } else {
                mouse::Button::Left
            })),
            layout::Layout::new(&node),
            mouse::Cursor::Available(node.bounds().center()),
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport,
        );
        assert!(
            messages.is_empty(),
            "opening is not a command or modal: {messages:?}"
        );
        let mut overlay = element
            .as_widget_mut()
            .overlay(
                &mut tree,
                layout::Layout::new(&node),
                &renderer,
                &viewport,
                iced::Vector::ZERO,
            )
            .expect("native menu overlay");
        let menu_node = overlay.as_overlay_mut().layout(&renderer, viewport.size());
        let bounds = menu_node.bounds();
        assert!(
            bounds.x >= viewport.x && bounds.y >= viewport.y,
            "{bounds:?}"
        );
        assert!(
            bounds.x + bounds.width <= viewport.width
                && bounds.y + bounds.height <= viewport.height,
            "{bounds:?}"
        );
        let events = std::iter::once(key(keyboard::key::Named::Home))
            .chain((0..row).map(|_| key(keyboard::key::Named::ArrowDown)))
            .chain(std::iter::once(key(keyboard::key::Named::Enter)));
        for event in events {
            overlay.as_overlay_mut().update(
                &event,
                layout::Layout::new(&menu_node),
                mouse::Cursor::Unavailable,
                &renderer,
                &mut iced::advanced::clipboard::Null,
                &mut Shell::new(&mut messages),
            );
        }
        assert_eq!(messages.len(), 1, "one menu command: {messages:?}");
        (size, messages.remove(0))
    }
}
