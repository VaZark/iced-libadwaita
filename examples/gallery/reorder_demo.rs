//! The Reorderable list and Meta parent pages, each a playground (see `playground.rs`): one
//! live component, a panel of controls for its properties and the code that builds it.
//!
//! The meta parent's children: the **reorderable list** (announces both modes), a **hint**
//! `MetaComponent` (announces `Edit`), a **search** `MetaComponent` (announces only `Search`,
//! so Edit never reaches it) and a plain caption that never registered.
//!
//! `GALLERY_REORDER=edit` starts the meta parent's list in edit mode.

use iced_libadwaita as adw;

use crate::playground::{self, Playground, Spec};
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
    /// The playground list: all it needs is `apply`.
    Plain(ReorderEvent),
    /// The playground's controls.
    Play(playground::Msg),
    /// The meta parent playground's controls.
    ParentPlay(playground::Msg),
    /// The list inside the meta parent (its own state).
    List(ReorderEvent),
    /// Switch the children that announced `Search` (nobody reacts but the search child).
    Search,
    /// What a child reported to the parent: (child, event).
    Child(Part, ChildEvent),
}

pub struct State {
    plain: ReorderState<&'static str>,
    play: Playground,
    parent_play: Playground,
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

fn row<'a>(name: &&'static str) -> Element<'a, Msg> {
    w::action_row(*name).into()
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
            play: Playground::new(vec![
                Spec::bool("title", "Title", true),
                Spec::bool("controls", "Controls bar", true),
                Spec::bool("disabled", "Disabled", false),
                Spec::bool("edit_row", "Separate edit row", false),
                Spec::choice("labels", "Button labels", &["Default", "Custom"], 0),
                Spec::number("lift", "Held row lift", 1.0..=1.1, 1.02),
            ]),
            parent_play: Playground::new(vec![
                Spec::bool("hint", "Hint child (announces Edit)", true),
                Spec::bool("search", "Search child (does not)", true),
                Spec::bool("plain", "Plain child (never registered)", true),
                Spec::number("spacing", "Spacing", 0.0..=24.0, 12.0),
            ]),
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
            Msg::Play(m) => self.play.update(m),
            Msg::ParentPlay(m) => self.parent_play.update(m),
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

    /// The Reorderable list page.
    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let p = &self.play;
        let mut list = ReorderableList::of(&self.plain, row)
            .controls(p.bool("controls"))
            .disabled(p.bool("disabled"))
            .lift(p.number("lift"))
            .on_event(Msg::Plain)
            .metrics(m);
        if p.bool("title") {
            list = list.title("Tools");
        }
        if p.bool("edit_row") {
            // Preview rows say more; the edit view's rows are compact.
            list = ReorderableList::of(&self.plain, |name: &&'static str| {
                w::action_row(*name).subtitle("Drawing tool").into()
            })
            .edit_row(row)
            .controls(p.bool("controls"))
            .disabled(p.bool("disabled"))
            .lift(p.number("lift"))
            .on_event(Msg::Plain)
            .metrics(m);
            if p.bool("title") {
                list = list.title("Tools");
            }
        }
        if p.choice("labels") == 1 {
            list = list.labels("Rearrange", "Save", "Discard");
        }
        // Without the bar the app drives the mode with its own buttons.
        let own: Element<'_, Msg> = if p.bool("controls") {
            list.into()
        } else if self.plain.is_editing() {
            column![
                list,
                iced::widget::row![
                    w::raised_button(ty::label("Cancel"))
                        .on_press(Msg::Plain(ReorderEvent::Cancel)),
                    w::suggested_button(ty::label("Save"))
                        .on_press(Msg::Plain(ReorderEvent::Submit)),
                ]
                .spacing(m.space_s)
            ]
            .spacing(m.space_m)
            .into()
        } else {
            column![
                list,
                w::raised_button(ty::label("Arrange tools"))
                    .on_press(Msg::Plain(ReorderEvent::Edit))
            ]
            .spacing(m.space_m)
            .into()
        };

        let mut code =
            String::from("ReorderableList::of(&self.order, |tool| action_row(*tool).into())\n");
        let mut line = |s: &str| {
            code.push_str("    ");
            code.push_str(s);
            code.push('\n');
        };
        if p.bool("title") {
            line(".title(\"Tools\")");
        }
        if p.changed("controls") {
            line(".controls(false)");
        }
        if p.changed("disabled") {
            line(".disabled(true)");
        }
        if p.bool("edit_row") {
            line(".edit_row(|tool| action_row(*tool).into())");
        }
        if p.changed("labels") {
            line(".labels(\"Rearrange\", \"Save\", \"Discard\")");
        }
        if p.changed("lift") {
            line(&format!(".lift({:.2})", p.number("lift")));
        }
        line(".on_event(Msg::List)");
        code.push_str("\n// update: Msg::List(event) => { self.order.apply(event); }");
        p.view(m, own, code, Msg::Play)
    }

    /// The Meta parent page: a list, a hint, an unrelated child and a plain one.
    pub fn parent_view(&self, m: Metrics) -> Element<'_, Msg> {
        let p = &self.parent_play;
        let mut parent = MetaParentComponent::new(&self.modes)
            .spacing(p.number("spacing"))
            .on_child_event(Msg::Child)
            .listener(Part::List, |mode, report| {
                ReorderableList::of(&self.order, row)
                    .title("Folders")
                    .editing(mode == Some(&Key::Mode(Mode::Edit)))
                    .on_event(Msg::List)
                    .on_change(report.on_change())
                    .metrics(m)
                    .into()
            });
        if p.bool("hint") {
            parent = parent.meta(Part::Hint, hint());
        }
        if p.bool("search") {
            parent = parent.meta(Part::Search, search());
        }
        parent = parent.child(ty::caption(self.last.clone()));
        if p.bool("plain") {
            parent = parent.child(ty::caption(
                "A plain child: it never registered, so nothing reaches it.",
            ));
        }
        let preview: Element<'_, Msg> = column![
            parent,
            w::raised_button(ty::label("Notify Search")).on_press(Msg::Search)
        ]
        .spacing(m.space_m)
        .into();

        let mut code = String::from(
            "MetaParentComponent::new(&self.modes)\n    .on_child_event(Msg::Child)\n    .listener(Part::List, |mode, report| {\n        ReorderableList::of(&self.order, row)\n            .editing(mode == Some(&Key::Mode(Mode::Edit)))\n            .on_event(Msg::List)\n            .on_change(report.on_change())\n            .into()\n    })\n",
        );
        if p.bool("hint") {
            code.push_str("    .meta(Part::Hint, hint())\n");
        }
        if p.bool("search") {
            code.push_str("    .meta(Part::Search, search())\n");
        }
        if p.bool("plain") {
            code.push_str("    .child(caption)\n");
        }
        if p.changed("spacing") {
            code.push_str(&format!("    .spacing({:.0})\n", p.number("spacing")));
        }
        code.push_str("\n// update: self.modes.notify(&Key::Mode(Mode::Edit)) reaches only\n// the children that announced Edit; Search is for the search child.");
        p.view(m, preview, code, Msg::ParentPlay)
    }
}
