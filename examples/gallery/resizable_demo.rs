//! The Resizable page: frames you can drag to any size (bottom-right corner, like a textarea),
//! to try responsive components without resizing the window. The width snaps to the
//! breakpoints (`[breakpoints]`) when it comes close, and the readout names the width class.
//!
//! * a `MetaComponent` that picks a layout by the width of its container (desktop, tablet,
//!   phone);
//! * a `ResponsiveTable`, a table until its columns no longer fit, cards after;
//! * the same frame with each of the other grips (an edge, another corner).
//!
//! `GALLERY_RESIZABLE=WxH` starts the first frame at that size (screenshots).

use iced_libadwaita as adw;

use adw::breakpoints::{Breakpoints, WidthClass};
use adw::primitives::resize::{Constraints, Grip};
use adw::widget::{self as w, MetaComponent, Resizable, When, typography as ty};
use adw::{Element, Metrics, Role};
use iced::widget::{column, container, row};
use iced::{Length, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Desktop,
    Tablet,
    Phone,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Meta(Size),
    Table(Size),
    Edge(Size),
}

pub struct State {
    meta: Size,
    table: Size,
    edge: Size,
}

impl Default for State {
    fn default() -> Self {
        let meta = std::env::var("GALLERY_RESIZABLE")
            .ok()
            .and_then(|v| {
                let (w, h) = v.split_once('x')?;
                Some(Size::new(w.parse().ok()?, h.parse().ok()?))
            })
            .unwrap_or(Size::new(820.0, 190.0));
        Self {
            meta,
            table: Size::new(560.0, 300.0),
            edge: Size::new(420.0, 120.0),
        }
    }
}

const PEOPLE: [(&str, &str, &str); 4] = [
    ("Ada Lovelace", "Engineer", "ada@example.org"),
    ("Grace Hopper", "Admiral", "grace@example.org"),
    ("Linus T.", "Maintainer", "linus@example.org"),
    ("Margaret H.", "Director", "margaret@example.org"),
];

/// The breakpoints, for the width to snap to.
fn snaps() -> Vec<f32> {
    let b = Breakpoints::current();
    vec![b.sm, b.md, b.lg, b.xl, b.xxl]
}

fn constraints(max: Size) -> Constraints {
    Constraints::new(Size::new(200.0, 90.0), max).snap_to(snaps(), 8.0)
}

/// "640 x 190 px - Lg": the size and the width class it is in.
fn readout<'a>(size: Size) -> Element<'a, Msg> {
    ty::caption(format!(
        "{:.0} x {:.0} px - {:?}",
        size.width,
        size.height,
        WidthClass::of(size.width)
    ))
    .style(adw::style::text::dim)
    .into()
}

fn group<'a>(
    m: Metrics,
    t: &'static str,
    d: &'static str,
    size: Size,
    body: Element<'a, Msg>,
) -> Element<'a, Msg> {
    column![
        ty::heading(t),
        ty::body(d).style(adw::style::text::dim),
        readout(size),
        body,
    ]
    .spacing(m.space_s)
    .into()
}

impl State {
    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Meta(s) => self.meta = s,
            Msg::Table(s) => self.table = s,
            Msg::Edge(s) => self.edge = s,
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        // -- a meta component choosing a layout by its container's width
        let tile = move |title: &'static str, note: String, role: Role| -> Element<'static, Msg> {
            container(
                column![
                    ty::heading(title),
                    ty::caption(note).style(adw::style::text::dim)
                ]
                .spacing(4),
            )
            .padding(sp)
            .width(Length::Fill)
            .style(move |t: &adw::Theme| container::Style {
                background: Some(iced::Background::Color(adw::color::alpha(
                    t.semantic.swatch(role).color,
                    0.18,
                ))),
                border: iced::Border::default().rounded(8),
                ..Default::default()
            })
            .into()
        };
        let layout = move |kind: Layout, cols: usize, role: Role| {
            move |size: Size| -> Element<'static, Msg> {
                let (title, tag) = match kind {
                    Layout::Desktop => ("Desktop layout", "Lg and up"),
                    Layout::Tablet => ("Tablet layout", "Md up to Lg"),
                    Layout::Phone => ("Phone layout", "below Md"),
                };
                let note = format!("{tag}: {:.0} px wide", size.width);
                row((0..cols).map(|i| {
                    tile(
                        title,
                        if i == 0 {
                            note.clone()
                        } else {
                            String::from("another column")
                        },
                        role,
                    )
                }))
                .spacing(sp)
                .padding(sp)
                .into()
            }
        };
        let meta = MetaComponent::new()
            .variant(
                Layout::Desktop,
                When::from_class(WidthClass::Lg),
                layout(Layout::Desktop, 3, Role::Success),
            )
            .variant(
                Layout::Tablet,
                When::from_class(WidthClass::Md),
                layout(Layout::Tablet, 2, Role::Accent),
            )
            .variant(
                Layout::Phone,
                When::always(),
                layout(Layout::Phone, 1, Role::Warning),
            );
        let meta = Resizable::new(Element::from(meta), self.meta)
            .on_resize(Msg::Meta)
            .constraints(constraints(Size::new(1500.0, 320.0)));

        // -- a table that becomes cards
        let table = w::ResponsiveTable::<(&str, &str, &str), Msg>::new(&PEOPLE)
            .column(
                w::Column::new("Name", |p: &(&str, &str, &str)| ty::heading(p.0).into())
                    .min_width(120.0),
            )
            .column(
                w::Column::new("Role", |p: &(&str, &str, &str)| ty::body(p.1).into())
                    .min_width(90.0),
            )
            .column(
                w::Column::new("Email", |p: &(&str, &str, &str)| ty::body(p.2).into())
                    .min_width(170.0),
            );
        let table = Resizable::new(Element::from(table), self.table)
            .on_resize(Msg::Table)
            .constraints(constraints(Size::new(1100.0, 420.0)));

        // -- other grips
        let edge = Resizable::new(
            Element::from(
                container(ty::body("Drag the right edge: only the width changes.")).padding(sp),
            ),
            self.edge,
        )
        .grip(Grip::Right)
        .on_resize(Msg::Edge)
        .constraints(Constraints::new(
            Size::new(160.0, 120.0),
            Size::new(900.0, 120.0),
        ));

        column![
            group(
                m,
                "Meta component",
                "Drag the corner: the layout is chosen by the frame's width, and the width snaps to the breakpoints.",
                self.meta,
                meta.into(),
            ),
            group(
                m,
                "Responsive table",
                "A table while the columns fit, cards below that.",
                self.table,
                table.into(),
            ),
            group(m, "Edge handle", "A handle on the right edge changes one axis.", self.edge, edge.into()),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
