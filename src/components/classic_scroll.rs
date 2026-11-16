//! `ClassicScroll`: a vertical scroll area with the classic scrollbar: arrow buttons at both ends,
//! a dithered track and a bevelled thumb.
//!
//! iced's `scrollable` draws a rail and a scroller with one flat color each, so it cannot draw
//! arrow buttons, a checkered track or a bevelled thumb. `ClassicScroll` keeps the `scrollable`
//! for the content and the mouse wheel (with its own bar hidden), takes the right `[scrollbar]
//! width` px of its bounds for a bar of its own, and drives the scrollable through iced's scroll
//! operation: it reads the scroll state, and moves it for an arrow press (a line), a track press
//! (a page) and a thumb drag.
//!
//! The theme asks for it with `[scrollbar] arrows` or `track-style=dither`; [`themed_scrolled`]
//! then builds it, and returns the toolkit's own scrolled window when the theme does not. Tokens:
//! `[scrollbar] width` (16), `arrows` (true), `track-style` (`plain` or `dither`), `track-light`
//! and `track-dark` (the two colors of a dithered track), and the frames `[frame.scrollbutton]`
//! and `[frame.scrollthumb]` (`border-style`, the bevel colors, `bg`).
//!
//! The dither is drawn as one image at one image pixel per logical pixel, so it is exact at a
//! display scale of 1.

use crate::primitives::{bevel, pixmap};
use crate::{Element, Theme};
use iced::advanced::image::{self as img, Renderer as _};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::widget::scrollable::{self, AbsoluteOffset, Direction, Scrollbar};
use iced::{Color, Event, Length, Point, Rectangle, Size, Vector, mouse};

/// Whether the installed theme draws scrollbars itself.
pub fn classic_scrollbar() -> bool {
    let cfg = crate::theme_config::current();
    cfg.bool("scrollbar", "arrows").unwrap_or(false)
        || cfg.text("scrollbar", "track-style") == Some("dither")
}

/// A vertical scroll area as the theme draws it: a [`ClassicScroll`] when the theme asks for a
/// classic scrollbar, else the toolkit's own scrolled window.
pub fn themed_scrolled<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    if classic_scrollbar() {
        ClassicScroll::new(content).into()
    } else {
        crate::layouts::scrolled_window(content).into()
    }
}

/// How far an arrow press scrolls, in px.
const LINE: f32 = 16.0;

/// See the [module](self).
pub struct ClassicScroll<'a, M> {
    content: Element<'a, M>,
    width: f32,
    arrows: bool,
    dither: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Info {
    viewport: f32,
    content: f32,
    offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Part {
    Up,
    Down,
}

struct State {
    info: Option<Info>,
    /// The distance from the thumb's top to the pointer while the thumb is dragged.
    grab: Option<f32>,
    held: Option<Part>,
    track: Option<((u32, u32), [[u8; 4]; 2], img::Handle)>,
}

/// Reads the first scrollable's geometry and offset.
struct Probe(Option<Info>);

impl Operation for Probe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn scrollable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        if self.0.is_none() {
            self.0 = Some(Info {
                viewport: bounds.height,
                content: content_bounds.height,
                offset: translation.y,
            });
        }
    }
}

/// Scrolls the first scrollable to an absolute vertical offset.
struct ScrollTo(f32, bool);

impl Operation for ScrollTo {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn scrollable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        if !self.1 {
            self.1 = true;
            state.scroll_to(AbsoluteOffset {
                x: None,
                y: Some(self.0.max(0.0)),
            });
        }
    }
}

impl<'a, M: 'a> ClassicScroll<'a, M> {
    /// `content`, scrolled vertically, with the classic bar on its right.
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        let cfg = crate::theme_config::current();
        let hidden = scrollable::Scrollable::new(content.into())
            .direction(Direction::Vertical(Scrollbar::hidden()))
            .width(Length::Fill)
            .height(Length::Fill);
        Self {
            content: hidden.into(),
            width: cfg.number("scrollbar", "width").unwrap_or(16.0).max(8.0),
            arrows: cfg.bool("scrollbar", "arrows").unwrap_or(true),
            dither: cfg.text("scrollbar", "track-style") == Some("dither"),
        }
    }
    /// The bar's width in px (default `[scrollbar] width`, 16).
    pub fn width(mut self, px: f32) -> Self {
        self.width = px.max(8.0);
        self
    }
    /// Arrow buttons at both ends (default `[scrollbar] arrows`).
    pub fn arrows(mut self, on: bool) -> Self {
        self.arrows = on;
        self
    }
    /// A dithered track (default `[scrollbar] track-style=dither`).
    pub fn dither(mut self, on: bool) -> Self {
        self.dither = on;
        self
    }

    fn probe(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
    ) -> Option<Info> {
        let mut probe = Probe(None);
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut probe);
        probe.0
    }

    fn scroll_to(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        y: f32,
    ) {
        let mut op = ScrollTo(y, false);
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut op);
    }
}

/// The parts of the bar for a bar rectangle `bar` and scroll `info`.
struct Geometry {
    up: Rectangle,
    down: Rectangle,
    track: Rectangle,
    thumb: Option<Rectangle>,
}

fn geometry(bar: Rectangle, arrows: bool, info: Option<Info>) -> Geometry {
    let end = if arrows { bar.width } else { 0.0 };
    let up = Rectangle::new(bar.position(), Size::new(bar.width, end));
    let down = Rectangle::new(
        Point::new(bar.x, bar.y + bar.height - end),
        Size::new(bar.width, end),
    );
    let track = Rectangle::new(
        Point::new(bar.x, bar.y + end),
        Size::new(bar.width, (bar.height - 2.0 * end).max(0.0)),
    );
    let thumb = info.filter(|i| i.content > i.viewport + 0.5).map(|i| {
        let ratio = (i.viewport / i.content).clamp(0.0, 1.0);
        let h = (track.height * ratio).max(bar.width).min(track.height);
        let range = (i.content - i.viewport).max(1.0);
        let y = track.y + (i.offset / range).clamp(0.0, 1.0) * (track.height - h);
        Rectangle::new(Point::new(bar.x, y), Size::new(bar.width, h))
    });
    Geometry {
        up,
        down,
        track,
        thumb,
    }
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for ClassicScroll<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            info: None,
            grab: None,
            held: None,
            track: None,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        let s = self.content.as_widget().size();
        Size::new(s.width, s.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let bar = self.width;
        let inner = limits.shrink(Size::new(bar, 0.0));
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &inner);
        let size = Size::new(node.size().width + bar, node.size().height);
        let node = layout::Node::with_children(
            limits.resolve(Length::Fill, Length::Fill, size),
            vec![node],
        );
        let child = node.children()[0].clone();
        let info = self.probe(tree, Layout::new(&child), renderer);
        let size = node.size();
        let geo = geometry(
            Rectangle::new(Point::ORIGIN, Size::new(bar, size.height)),
            self.arrows,
            info,
        );
        let state = tree.state.downcast_mut::<State>();
        state.info = info;
        if self.dither {
            ensure_track(state, geo.track.width, geo.track.height);
        }
        node
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let child = layout.children().next().expect("a content layout");
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        let info = self.probe(tree, child, renderer);
        tree.state.downcast_mut::<State>().info = info;
        let bounds = layout.bounds();
        let bar = Rectangle::new(
            Point::new(bounds.x + bounds.width - self.width, bounds.y),
            Size::new(self.width, bounds.height),
        );
        let geo = geometry(bar, self.arrows, info);
        if self.dither {
            ensure_track(
                tree.state.downcast_mut::<State>(),
                geo.track.width,
                geo.track.height,
            );
        }
        let Some(i) = info else { return };
        let range = (i.content - i.viewport).max(0.0);
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(p) = cursor.position_over(bar) else {
                    return;
                };
                let target = if self.arrows && geo.up.contains(p) {
                    tree.state.downcast_mut::<State>().held = Some(Part::Up);
                    Some(i.offset - LINE)
                } else if self.arrows && geo.down.contains(p) {
                    tree.state.downcast_mut::<State>().held = Some(Part::Down);
                    Some(i.offset + LINE)
                } else if let Some(t) = geo.thumb.filter(|t| t.contains(p)) {
                    tree.state.downcast_mut::<State>().grab = Some(p.y - t.y);
                    None
                } else if let Some(t) = geo.thumb {
                    // A press on the track pages towards it.
                    Some(if p.y < t.y {
                        i.offset - i.viewport
                    } else {
                        i.offset + i.viewport
                    })
                } else {
                    None
                };
                if let Some(y) = target {
                    self.scroll_to(tree, child, renderer, y.clamp(0.0, range));
                }
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let state = tree.state.downcast_ref::<State>();
                if let (Some(grab), Some(t)) = (state.grab, geo.thumb)
                    && let Some(p) = cursor.position()
                {
                    let free = (geo.track.height - t.height).max(1.0);
                    let top = (p.y - grab - geo.track.y).clamp(0.0, free);
                    self.scroll_to(tree, child, renderer, top / free * range);
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let state = tree.state.downcast_mut::<State>();
                if state.grab.take().is_some() || state.held.take().is_some() {
                    shell.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().expect("a content layout"),
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().expect("a content layout"),
            renderer,
            operation,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let child = layout.children().next().expect("a content layout");
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            child,
            cursor,
            viewport,
        );
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let bar = Rectangle::new(
            Point::new(bounds.x + bounds.width - self.width, bounds.y),
            Size::new(self.width, bounds.height),
        );
        let geo = geometry(bar, self.arrows, state.info);
        let fill_rect = |renderer: &mut iced::Renderer, r: Rectangle, color: Color| {
            renderer.fill_quad(
                Quad {
                    bounds: r,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                color,
            );
        };
        // The track: the dither image made at layout time, else the flat dark color.
        let dark = track_colors()[1];
        match state.track.as_ref().filter(|_| self.dither) {
            Some((_, _, handle)) => renderer.draw_image(
                img::Image::new(handle.clone())
                    .filter_method(img::FilterMethod::Nearest)
                    .snap(true),
                geo.track,
                geo.track,
            ),
            None => fill_rect(renderer, geo.track, dark),
        }
        // The buttons and the thumb: bevelled boxes.
        let boxed = |renderer: &mut iced::Renderer, r: Rectangle, section: &str, pressed: bool| {
            let (style, width) =
                crate::components::bevel_of(section).unwrap_or((bevel::BevelStyle::Raised, 2.0));
            let style = if pressed { style.pressed() } else { style };
            let face = theme
                .token_color(section, "bg")
                .unwrap_or(theme.semantic.window.bg);
            fill_rect(renderer, r, face);
            let colors = theme.bevel_colors(section);
            for e in bevel::edges(style, bevel::rings(width), r.size()) {
                fill_rect(
                    renderer,
                    Rectangle::new(Point::new(r.x + e.rect.x, r.y + e.rect.y), e.rect.size()),
                    colors.tone(e.tone),
                );
            }
        };
        let ink = |_: ()| style.text_color;
        if self.arrows {
            for (rect, dir, part) in [(geo.up, "up", Part::Up), (geo.down, "down", Part::Down)] {
                let pressed = state.held == Some(part);
                boxed(renderer, rect, "frame.scrollbutton", pressed);
                if let Some(glyph) = pixmap::arrow(dir) {
                    let off = if pressed { 1.0 } else { 0.0 };
                    let (gw, gh) = (glyph.width() as f32, glyph.height() as f32);
                    let (ox, oy) = (
                        rect.x + ((rect.width - gw) / 2.0).floor() + off,
                        rect.y + ((rect.height - gh) / 2.0).floor() + off,
                    );
                    for run in glyph.runs() {
                        fill_rect(
                            renderer,
                            Rectangle::new(
                                Point::new(ox + run.x as f32, oy + run.y as f32),
                                Size::new(run.width as f32, 1.0),
                            ),
                            ink(()),
                        );
                    }
                }
            }
        }
        if let Some(t) = geo.thumb {
            boxed(renderer, t, "frame.scrollthumb", false);
        }
    }
}

/// The two colors of a dithered track: `[scrollbar] track-light` (default white) and
/// `track-dark` (default the classic face gray), read when the widget is laid out.
fn track_colors() -> [Color; 2] {
    let cfg = crate::theme_config::current();
    [
        cfg.color("scrollbar", "track-light")
            .unwrap_or(Color::WHITE),
        cfg.color("scrollbar", "track-dark")
            .unwrap_or_else(|| crate::color::hex(0xc0c0c0)),
    ]
}

/// Makes the dither image for a track of `width` x `height` when there is none for that size and
/// those colors yet, so drawing never has to upload one.
fn ensure_track(state: &mut State, width: f32, height: f32) {
    let (w, h) = (width.max(0.0) as u32, height.max(0.0) as u32);
    if w == 0 || h == 0 {
        state.track = None;
        return;
    }
    let colors = track_colors().map(rgba8);
    if state
        .track
        .as_ref()
        .is_some_and(|(size, c, _)| *size == (w, h) && *c == colors)
    {
        return;
    }
    let handle = img::Handle::from_rgba(w, h, pixmap::dither_rgba(w, h, colors[0], colors[1]));
    state.track = Some(((w, h), colors, handle));
}

fn rgba8(c: Color) -> [u8; 4] {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [b(c.r), b(c.g), b(c.b), b(c.a)]
}

impl<'a, M: 'a> From<ClassicScroll<'a, M>> for Element<'a, M> {
    fn from(s: ClassicScroll<'a, M>) -> Self {
        iced::Element::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAR: Rectangle = Rectangle {
        x: 100.0,
        y: 10.0,
        width: 16.0,
        height: 200.0,
    };

    #[test]
    fn the_bar_is_arrow_track_arrow() {
        let g = geometry(BAR, true, None);
        assert_eq!((g.up.y, g.up.height), (10.0, 16.0));
        assert_eq!((g.down.y, g.down.height), (194.0, 16.0));
        assert_eq!((g.track.y, g.track.height), (26.0, 168.0));
        assert!(g.thumb.is_none());
        let plain = geometry(BAR, false, None);
        assert_eq!((plain.track.y, plain.track.height), (10.0, 200.0));
    }

    #[test]
    fn the_thumb_is_the_visible_share_of_the_track_and_moves_with_the_offset() {
        let info = |offset| {
            Some(Info {
                viewport: 100.0,
                content: 400.0,
                offset,
            })
        };
        let top = geometry(BAR, true, info(0.0)).thumb.unwrap();
        assert_eq!((top.y, top.height), (26.0, 42.0));
        let bottom = geometry(BAR, true, info(300.0)).thumb.unwrap();
        assert!((bottom.y + bottom.height - 194.0).abs() < 1e-3);
        let mid = geometry(BAR, true, info(150.0)).thumb.unwrap();
        assert!(mid.y > top.y && mid.y < bottom.y);
        // Content that fits has no thumb.
        assert!(
            geometry(
                BAR,
                true,
                Some(Info {
                    viewport: 100.0,
                    content: 100.0,
                    offset: 0.0
                })
            )
            .thumb
            .is_none()
        );
    }

    #[test]
    fn the_thumb_is_never_smaller_than_the_bar_is_wide() {
        let g = geometry(
            BAR,
            true,
            Some(Info {
                viewport: 10.0,
                content: 100000.0,
                offset: 0.0,
            }),
        );
        assert_eq!(g.thumb.unwrap().height, 16.0);
    }

    #[test]
    fn an_unset_theme_gets_the_stock_scrolled_window() {
        assert!(!classic_scrollbar());
        let _: Element<'_, ()> = themed_scrolled(iced::widget::space());
        let _: Element<'_, ()> = ClassicScroll::new(iced::widget::space())
            .width(16.0)
            .arrows(true)
            .dither(true)
            .into();
    }
}
