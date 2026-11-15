//! Demos for committed widgets not otherwise exposed as gallery details.
use crate::nav::Detail;
use iad::components::typography as ty;
use iad::primitives::sorting::SortState;
use iad::{Element, Metrics, components as cmp, widgets as w};
use iced::Length;
use iced::widget::{column, container, row};
use iced_adaptive_design as iad;

#[derive(Debug, Clone)]
pub enum Msg {
    Open(bool),
    Select(usize),
    Accent(iad::palette::AccentColor),
    Custom(String),
    Sort(u8),
    Narrow(bool),
}

pub struct State {
    open: bool,
    selected: usize,
    accent: iad::palette::AccentColor,
    custom: String,
    sort: SortState<u8>,
    narrow: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: true,
            selected: 0,
            accent: iad::palette::AccentColor::Blue,
            custom: "#3584e4".into(),
            sort: SortState::new(),
            narrow: false,
        }
    }
}

impl State {
    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Open(v) => self.open = v,
            Msg::Select(v) => self.selected = v,
            Msg::Accent(v) => self.accent = v,
            Msg::Custom(v) => {
                if let Some(accent) = iad::palette::AccentColor::parse(&v) {
                    self.accent = accent;
                }
                self.custom = v;
            }
            Msg::Sort(key) => self.sort.activate(key, false),
            Msg::Narrow(v) => self.narrow = v,
        }
    }

    pub fn view(&self, d: Detail, m: Metrics) -> Element<'_, Msg> {
        let button = |label: &'static str, msg| cmp::raised_button(ty::label(label)).on_press(msg);
        match d {
            Detail::AccentPicker => column![
                w::accent_picker(self.accent, Msg::Accent)
                    .custom(&self.custom, Msg::Custom)
                    .metrics(m),
                ty::body(format!(
                    "Selected accent: {:?} (local to this demo)",
                    self.accent
                )),
            ]
            .spacing(m.space_m)
            .into(),
            Detail::Collapsible => w::Collapsible::new(
                button(
                    if self.open {
                        "Hide details"
                    } else {
                        "Show details"
                    },
                    Msg::Open(!self.open),
                ),
                w::card(ty::body(
                    "The body remains mounted while the disclosure animates.",
                )),
                self.open,
            )
            .into(),
            Detail::ExpanderRow => {
                w::boxed_list([
                    w::ExpanderRow::new("Advanced options", self.open, Msg::Open)
                        .subtitle("Expand to reveal nested settings")
                        .push(w::ActionRow::new("Connection").subtitle("Automatic"))
                        .push(w::ActionRow::new("Security").subtitle("System defaults"))
                        .into(),
                ])
            }
            Detail::SectionDivider => column![
                w::SectionDivider::new("Documents")
                    .detail("3 items")
                    .open(self.open)
                    .on_toggle(Msg::Open(!self.open))
                    .metrics(m),
                Element::from(iad::layouts::reveal(
                    column![
                        ty::body("Notes.txt"),
                        ty::body("Plan.md"),
                        ty::body("Readme.md")
                    ]
                    .spacing(m.space_s),
                    self.open,
                )),
            ]
            .spacing(m.space_m)
            .into(),
            Detail::ProfileHeader => column![
                w::SwitchRow::new("Narrow layout", self.narrow, Msg::Narrow).metrics(m),
                w::ProfileHeader::new(
                    "Ada Lovelace",
                    iad::Icon::Home.view(64.0),
                    ty::body("Mathematician and writer · London")
                )
                .after_name(ty::caption("Contributor"))
                .controls(button(
                    if self.open { "Following" } else { "Follow" },
                    Msg::Open(!self.open)
                ))
                .narrow(self.narrow)
                .metrics(m),
            ]
            .spacing(m.space_m)
            .into(),
            Detail::FloatingIndicator => column![
                w::FloatingIndicator::new(
                    ["Preview", "Source", "Activity"]
                        .into_iter()
                        .enumerate()
                        .map(|(i, label)| cmp::flat_button(ty::label(label))
                            .on_press(Msg::Select(i))
                            .into())
                        .collect(),
                    self.selected.min(2)
                )
                .spacing(m.space_s)
                .target_shape(|theme, _, _| {
                    iad::styles::button::flat(theme, iced::widget::button::Status::Active)
                        .border
                        .radius
                }),
                ty::body(
                    ["Rendered preview", "Source code", "Recent activity"][self.selected.min(2)]
                ),
            ]
            .spacing(m.space_m)
            .into(),
            Detail::SlidePages => {
                let pages = vec![
                    column![
                        ty::title_3("Library"),
                        button("Open folder", Msg::Select(1))
                    ]
                    .spacing(m.space_m)
                    .into(),
                    column![
                        ty::title_3("Folder"),
                        button("Open document", Msg::Select(2)),
                        button("Back to library", Msg::Select(0))
                    ]
                    .spacing(m.space_m)
                    .into(),
                    column![
                        ty::title_3("Document"),
                        ty::body("Go back to see the reverse transition."),
                        button("Back to folder", Msg::Select(1))
                    ]
                    .spacing(m.space_m)
                    .into(),
                ];
                container(w::SlidePages::new(pages, self.selected.min(2)).depths(vec![0, 1, 2]))
                    .width(Length::Fill)
                    .height(240)
                    .into()
            }
            Detail::SortHeader => {
                let mut items = [("Notes", 12), ("Archive", 40), ("Photos", 8)];
                self.sort.apply(&mut items, |key, a, b| {
                    if *key == 0 {
                        a.0.cmp(b.0)
                    } else {
                        a.1.cmp(&b.1)
                    }
                });
                let mut list = column![
                    ty::caption("Click a header: ascending → descending → original order."),
                    w::SortHeader::new(&self.sort, Msg::Sort)
                        .column(0, "Name")
                        .column(1, "Count")
                        .padding(0)
                        .metrics(m)
                ]
                .spacing(m.space_s);
                for (name, count) in items {
                    list = list.push(
                        row![
                            container(ty::body(name)).width(Length::Fill),
                            container(ty::body(count.to_string())).width(Length::Fill)
                        ]
                        .spacing(m.space_m),
                    );
                }
                list.into()
            }
            Detail::Toolbar => container(
                w::ToolbarView::new(
                    container(ty::body(if self.open {
                        "Select an item to act on it."
                    } else {
                        "Selection cleared."
                    }))
                    .center(Length::Fill),
                )
                .top_bar(w::HeaderBar::new().title("Documents"))
                .bottom_bar(
                    w::ActionBar::new()
                        .pack_start(button("Select", Msg::Open(true)))
                        .pack_end(button("Clear selection", Msg::Open(false))),
                )
                .top_bar_style(w::BarStyle::Raised)
                .bottom_bar_style(w::BarStyle::Raised),
            )
            .height(260)
            .width(Length::Fill)
            .into(),
            Detail::OverlaySplitView => column![
                w::SwitchRow::new("Collapse to overlay", self.narrow, Msg::Narrow).metrics(m),
                button("Show sidebar", Msg::Open(true)),
                Element::from(
                    container(
                        w::OverlaySplitView::new(
                            w::card(
                                column![
                                    ty::title_3("Folders"),
                                    button("Close sidebar", Msg::Open(false))
                                ]
                                .spacing(m.space_m)
                            ),
                            w::navigation_view(
                                vec![w::NavigationPage::new(
                                    "Documents",
                                    ty::body("The sidebar overlays this page when collapsed.")
                                )],
                                Msg::Open(false)
                            ),
                        )
                        .sidebar_width(220.0)
                        .collapsed(self.narrow)
                        .show_sidebar(self.open)
                        .on_close(Msg::Open(false))
                    )
                    .width(Length::Fill)
                    .height(280)
                ),
            ]
            .spacing(m.space_m)
            .into(),
            _ => unreachable!("only stable demo details are routed here"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_details_are_discoverable_and_build_in_both_layouts() {
        let mut state = State::default();
        let details = [
            Detail::AccentPicker,
            Detail::Collapsible,
            Detail::ExpanderRow,
            Detail::SectionDivider,
            Detail::ProfileHeader,
            Detail::FloatingIndicator,
            Detail::SlidePages,
            Detail::SortHeader,
            Detail::Toolbar,
            Detail::OverlaySplitView,
        ];
        for detail in details {
            assert_eq!(Detail::all().iter().filter(|d| **d == detail).count(), 1);
            assert!(!detail.title().is_empty());
            assert!(!detail.description().is_empty());
            let _ = detail.page();
            for narrow in [false, true] {
                state.update(Msg::Narrow(narrow));
                for open in [false, true] {
                    state.update(Msg::Open(open));
                    for selected in 0..3 {
                        state.update(Msg::Select(selected));
                        let _ = state.view(detail, Metrics::desktop());
                    }
                }
            }
        }
    }

    #[test]
    fn controls_update_demo_state() {
        let mut s = State::default();
        s.update(Msg::Open(false));
        s.update(Msg::Select(2));
        s.update(Msg::Narrow(true));
        assert!(!s.open);
        assert_eq!(s.selected, 2);
        assert!(s.narrow);
        s.update(Msg::Custom("#ff0000".into()));
        let accent = s.accent;
        s.update(Msg::Custom("invalid".into()));
        assert_eq!(s.accent, accent);
        assert_eq!(s.custom, "invalid");
        s.update(Msg::Sort(0));
        assert_eq!(
            s.sort.direction(&0),
            Some(iad::primitives::sorting::Direction::Asc)
        );
    }
}
