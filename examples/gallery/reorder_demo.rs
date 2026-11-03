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
    self as w, MetaComponent, MetaParentComponent, MetaRegistry, Mode, ReorderState,
    ReorderableList, When, typography as ty,
};
use adw::{Element, Metrics};
use iced::Length;
use iced::widget::column;

#[derive(Debug, Clone, PartialEq)]
enum Part {
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
    Edit,
    Reorder(usize, usize),
    Submit,
    Cancel,
    /// Switch the children that announced `Search` (nobody reacts but the search child).
    Search,
}

pub struct State {
    order: ReorderState<&'static str>,
    modes: MetaRegistry<Part, Key>,
}

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
        Self { order, modes }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Edit => {
                self.order.edit();
                self.modes.notify(&Key::Mode(Mode::Edit));
            }
            Msg::Reorder(from, to) => {
                self.order.reorder(from, to);
            }
            Msg::Submit => {
                self.order.submit();
                self.modes.notify(&Key::Mode(Mode::Preview));
            }
            Msg::Cancel => {
                self.order.cancel();
                self.modes.notify(&Key::Mode(Mode::Preview));
            }
            Msg::Search => {
                self.modes.notify(&Key::Search);
            }
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let items = self.order.items();
        let parent = MetaParentComponent::new(&self.modes)
            .spacing(sp)
            .listener(Part::List, |mode| {
                ReorderableList::new(items, |name| w::action_row(*name).into())
                    .title("Folders")
                    .editing(mode == Some(&Key::Mode(Mode::Edit)))
                    .on_edit(Msg::Edit)
                    .on_reorder(Msg::Reorder)
                    .on_submit(Msg::Submit)
                    .on_cancel(Msg::Cancel)
                    .metrics(m)
                    .into()
            })
            .meta(Part::Hint, hint())
            .meta(Part::Search, search())
            .child(ty::caption(
                "A plain child: it never registered, so nothing reaches it.",
            ));
        let disabled = ReorderableList::new(&NAMES[..3], |name| w::action_row(*name).into())
            .title("Disabled")
            .disabled(true)
            .on_edit(Msg::Edit)
            .metrics(m);
        column![
            group(
                m,
                "Reorderable list in a meta parent",
                "Edit shows the grips; drag a row by its grip; Done keeps the order, Cancel restores it.",
                parent.into()
            ),
            group(
                m,
                "Notify an unrelated action",
                "Search is announced by the search child only; the list and the hint do not move.",
                w::raised_button(ty::label("Notify Search"))
                    .on_press(Msg::Search)
                    .into()
            ),
            group(m, "Disabled", "A disabled list cannot enter edit mode.", disabled.into()),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
