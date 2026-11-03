//! Components that came up from the weave music app: the virtual list, editable text, the detail
//! header and the icon set.

use iced_adaptive_design as iad;

use iad::primitives::virtualize::Viewport;
use iad::widget::{self as w, DetailHeader, EditableText, PreferencesGroup, VirtualList};
use iad::{Element, Icon, Metrics};
use iced::widget::{column, container, row, text};
use iced::{Color, Length};

const TOTAL: usize = 10_000;
const ROW: f32 = 40.0;
const BIO: &str = "Recorded over three winters in a converted barn. The songs were written on the road and finished in the barn, where the whole band could play at once. Most of what you hear is the first or second take, with the room left in.";

#[derive(Debug, Clone)]
pub enum Msg {
    Scroll(Viewport),
    Edit,
    Input(String),
    Done,
    Expand(bool),
    NickEdit,
    NickInput(String),
    NickDone,
    Narrow(bool),
    Noop,
}

pub struct State {
    viewport: Viewport,
    bio: String,
    editing: bool,
    expanded: bool,
    nick: String,
    nick_editing: bool,
    narrow: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            viewport: Viewport::new(320.0),
            bio: BIO.into(),
            editing: false,
            expanded: false,
            nick: String::new(),
            nick_editing: false,
            narrow: false,
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Scroll(v) => self.viewport = v,
            Msg::Edit => self.editing = true,
            Msg::Input(s) => self.bio = s,
            Msg::Done => self.editing = false,
            Msg::Expand(e) => self.expanded = e,
            Msg::NickEdit => self.nick_editing = true,
            Msg::NickInput(s) => self.nick = s,
            Msg::NickDone => self.nick_editing = false,
            Msg::Narrow(n) => self.narrow = n,
            Msg::Noop => {}
        }
    }

    fn group<'a>(title: &'static str, desc: &'static str, m: Metrics) -> PreferencesGroup<'a, Msg> {
        PreferencesGroup::<Msg>::new()
            .title(title)
            .description(desc)
            .metrics(m)
    }

    pub fn virtual_view(&self, m: Metrics) -> Element<'_, Msg> {
        let built = self.viewport.window(TOTAL, ROW, 4);
        let list = VirtualList::new(TOTAL, ROW, &self.viewport, move |i| {
            row![
                text(format!("Track {}", i + 1)).width(Length::Fill),
                text(format!("{}:{:02}", 2 + i % 4, (i * 7) % 60)).size(12),
            ]
            .padding([0.0, 12.0])
            .align_y(iced::Alignment::Center)
            .into()
        })
        .on_scroll(Msg::Scroll);
        Self::group(
            "Virtual list",
            "Ten thousand rows of one height. Only the rows in view are built.",
            m,
        )
        .push(w::card(
            column![
                container(list).height(Length::Fixed(320.0)),
                text(format!(
                    "Built rows {} to {} of {TOTAL}",
                    built.start + 1,
                    built.end
                ))
                .size(12),
            ]
            .spacing(m.space_s),
        ))
        .into()
    }

    pub fn editable_view(&self, m: Metrics) -> Element<'_, Msg> {
        let bio = EditableText::new(&self.bio, Msg::Input)
            .placeholder("Add a description")
            .editing(self.editing)
            .expanded(self.expanded)
            .max_height(60.0)
            .on_edit(Msg::Edit)
            .on_done(Msg::Done)
            .on_expand(Msg::Expand);
        let nick = EditableText::new(&self.nick, Msg::NickInput)
            .placeholder("Add a nickname")
            .editing(self.nick_editing)
            .on_edit(Msg::NickEdit)
            .on_done(Msg::NickDone);
        column![
            Self::group(
                "Clamped text",
                "A long text is clamped; expanding it pushes what is below down. The pencil swaps it for an entry.",
                m,
            )
            .push(w::card(column![bio, text("Content below the text").size(12)].spacing(m.space_m))),
            Self::group("Empty", "The placeholder is the edit button.", m).push(w::card(nick)),
        ]
        .spacing(m.group_spacing)
        .into()
    }

    pub fn header_view(&self, m: Metrics) -> Element<'_, Msg> {
        let tint = Color::from_rgb8(0x9c, 0x4a, 0xd1);
        let cover = container(text(""))
            .width(Length::Fixed(if self.narrow { 160.0 } else { 180.0 }))
            .height(Length::Fixed(if self.narrow { 160.0 } else { 180.0 }))
            .style(move |_: &iad::Theme| container::Style {
                background: Some(tint.into()),
                border: iced::Border::default().rounded(10),
                ..Default::default()
            });
        let header = DetailHeader::new("Evening drive")
            .kind("Playlist")
            .cover(cover)
            .description(text("Slow songs for a long road."))
            .facts(text("24 songs, 1 hr 32 min").size(12))
            .primary(w::suggested_circular_button(Icon::Play.view(m.icon_size)).on_press(Msg::Noop))
            .action(w::icon_button(Icon::Heart).on_press(Msg::Noop))
            .action(w::icon_button(Icon::ViewMore).on_press(Msg::Noop))
            .end(w::flat_button(text("Share")).on_press(Msg::Noop))
            .tint(tint)
            .narrow(self.narrow)
            .metrics(m);
        column![
            Self::group(
                "Detail header",
                "Wide or narrow, with a wash of the cover's color fading down from the top.",
                m
            )
            .push(w::SwitchRow::new("Narrow arrangement", self.narrow, Msg::Narrow).metrics(m)),
            header,
        ]
        .spacing(m.group_spacing)
        .into()
    }

    pub fn icons_view(&self, m: Metrics) -> Element<'_, Msg> {
        let cells = Icon::ALL.iter().map(|i| {
            container(
                column![i.view(24.0), text(format!("{i:?}")).size(11)]
                    .spacing(m.space_xs)
                    .align_x(iced::Alignment::Center),
            )
            .width(Length::Fixed(110.0))
            .padding(m.space_s)
            .center_x(Length::Fixed(110.0))
            .into()
        });
        Self::group(
            "Icons",
            "The built-in symbolic icons. Anything that takes an icon takes an `Icon` or your own SVG (`IconSource`).",
            m,
        )
        .push(w::card(row(cells).spacing(m.space_xs).wrap()))
        .into()
    }
}
