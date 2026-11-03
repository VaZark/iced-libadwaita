//! The Reorderable list page: a list that edits by dragging, inside a `MetaParentComponent`.
//!
//! Three children share one parent. Each registered on its own and announced what it
//! implements; `Mode::Edit` / `Mode::Preview` reach only those that announced them:
//!
//! * the **reorderable list** (announces both modes);
//! * a **hint** `MetaComponent` (announces `Edit`): it explains the grip while editing;
//! * a **search** `MetaComponent` (announces only `Search`): unrelated, so it never changes;
//! * a plain caption, which never registered.
//!
//! `GALLERY_REORDER=edit` starts in edit mode.

use iced_libadwaita as adw;

use adw::widget::{
    self as w, ChildEvent, MetaComponent, MetaParentComponent, MetaRegistry, Mode, ReorderEvent,
    ReorderState, ReorderableList, When, typography as ty,
};
use adw::{Element, Metrics};
use iced::Length;
use iced::widget::column;

#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    List,
    Hint,
    Search,
}

#[derive(Debug, Clone, PartialEq)]
enum Key {
    Mode(Mode),
    Search,
}

#[derive(Debug, Clone)]
pub enum Msg {
    /// The standalone list: all it needs is `apply`.
    Plain(ReorderEvent),
    /// The list inside the meta parent (its own state).
    List(ReorderEvent),
    /// Switch the children that announced `Search` (nobody reacts but the search child).
    Search,
    /// What a child reported to the parent: (child, event).
    Child(Part, ChildEvent),
}

pub struct State {
    plain: ReorderState<&'static str>,
    order: ReorderState<&'static str>,
    modes: MetaRegistry<Part, Key>,
    /// The parent's reaction, shown under the list.
    last: String,
}

const TOOLS: [&str; 4] = ["Pencil", "Brush", "Eraser", "Ruler"];

const NAMES: [&str; 5] = ["Inbox", "Drafts", "Sent", "Archive", "Trash"];

fn hint<'a>() -> MetaComponent<'a, Key, Msg> {
    MetaComponent::new()
        .height(Length::Fixed(24.0))
        .when_selected(Key::Mode(Mode::Edit), |_| {
            ty::body("Drag the grips to reorder, then press Done.").into()
        })
        .variant(Key::Mode(Mode::Preview), When::always(), |_| {
            ty::body("Press Edit to rearrange the folders.").into()
        })
}

fn search<'a>() -> MetaComponent<'a, Key, Msg> {
    MetaComponent::new()
        .height(Length::Fixed(24.0))
        .when_selected(Key::Search, |_| ty::body("Searching…").into())
        .variant(Key::Mode(Mode::Preview), When::always(), |_| {
            ty::body("Not searching (unrelated to Edit).").into()
        })
}

fn group<'a>(
    m: Metrics,
    title: &'static str,
    description: &'static str,
    body: Element<'a, Msg>,
) -> w::PreferencesGroup<'a, Msg> {
    w::PreferencesGroup::<Msg>::new()
        .title(title)
        .description(description)
        .metrics(m)
        .push(w::card(
            iced::widget::container(body)
                .padding(m.space_m)
                .width(Length::Fill),
        ))
}

impl Default for State {
    fn default() -> Self {
        let mut modes = MetaRegistry::new();
        // Each child registers separately and announces its own actions.
        modes.register(
            Part::List,
            ReorderableList::<&str, Msg>::actions().map(Key::Mode),
        );
        MetaParentComponent::register(&mut modes, Part::Hint, &hint());
        MetaParentComponent::register(&mut modes, Part::Search, &search());
        let mut order = ReorderState::new(NAMES.to_vec());
        if std::env::var("GALLERY_REORDER").as_deref() == Ok("edit") {
            order.edit();
            modes.notify(&Key::Mode(Mode::Edit));
        }
        Self {
            plain: ReorderState::new(TOOLS.to_vec()),
            order,
            modes,
            last: "No order change yet.".into(),
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Plain(event) => {
                self.plain.apply(event);
            }
            Msg::List(event) => {
                self.order.apply(event);
                // The parent's other children follow the list's mode.
                match event {
                    ReorderEvent::Edit => {
                        self.modes.notify(&Key::Mode(Mode::Edit));
                    }
                    ReorderEvent::Submit | ReorderEvent::Cancel => {
                        self.modes.notify(&Key::Mode(Mode::Preview));
                    }
                    ReorderEvent::Reorder { .. } => {}
                }
            }
            Msg::Child(id, event) => {
                // Independent of the list's own `Reorder`: only registered children count.
                if let Some(e) = self.modes.emit(&id, event) {
                    let ChildEvent::Reordered { from, to } = e.event;
                    self.last = format!("Parent: {:?} moved row {from} to {to}.", e.child);
                }
            }
            Msg::Search => {
                self.modes.notify(&Key::Search);
            }
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let parent = MetaParentComponent::new(&self.modes)
            .spacing(sp)
            .on_child_event(Msg::Child)
            .listener(Part::List, |mode, report| {
                ReorderableList::of(&self.order, |name| w::action_row(*name).into())
                    .title("Folders")
                    .editing(mode == Some(&Key::Mode(Mode::Edit)))
                    .on_event(Msg::List)
                    .on_change(report.on_change())
                    .metrics(m)
                    .into()
            })
            .meta(Part::Hint, hint())
            .meta(Part::Search, search())
            .child(ty::caption(
                "A plain child: it never registered, so nothing reaches it.",
            ));
        let plain = ReorderableList::of(&self.plain, |name| w::action_row(*name).into())
            .title("Tools")
            .on_event(Msg::Plain)
            .metrics(m);
        let disabled =
            ReorderableList::<_, Msg>::new(&TOOLS[..3], |name| w::action_row(*name).into())
                .title("Disabled")
                .disabled(true)
                .metrics(m);
        column![
            group(
                m,
                "Reorderable list",
                "Edit shows the grips; drag a row by its grip; Done keeps the order, Cancel restores it.",
                plain.into()
            ),
            group(m, "Disabled", "A disabled list cannot enter edit mode.", disabled.into()),
            group(
                m,
                "In a meta parent",
                "Its own list, registry and children: Edit also switches the hint; Search is announced by the search child only, so nothing else moves.",
                column![
                    parent,
                    w::raised_button(ty::label("Notify Search")).on_press(Msg::Search)
                ]
                .spacing(sp)
                .into()
            ),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
