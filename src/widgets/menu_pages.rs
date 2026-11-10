//! The machinery behind sliding submenus: a stack of menu pages inside one popover, of which one
//! shows at a time. Choosing a submenu row slides the next page in from the end edge, and the back
//! row at the top of a subpage slides it away again, as GTK's `GtkPopoverMenu` does for a submenu
//! that is not cascaded beside its parent.
//!
//! * [`MenuPages`] holds every page and which one shows. The "which" is the widget's own state, so
//!   the app sees only the messages of the items that are actually chosen.
//! * [`NavRow`] is a row that opens another page. It looks like a menu item, and on a click it
//!   records which page it wants (a [`Request`]) and leaves the click uncaptured, so the popover
//!   (which closes after a captured click on an item) stays open.

use std::cell::Cell;
use std::rc::Rc;

use crate::Theme;
use crate::layouts::animation::{Progress, Spec, Trigger};
use crate::layouts::motion::{advance, draw_translated};
use crate::styles::button as sb;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::widget::button::Status;
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// The page a row asked for, passed from the row to the pages widget that holds it.
#[derive(Clone, Default)]
pub(crate) struct Request(Rc<Cell<Option<usize>>>);

impl Request {
    fn ask(&self, page: usize) {
        self.0.set(Some(page));
    }
    fn take(&self) -> Option<usize> {
        self.0.take()
    }
}

/// How the pages of a menu with submenus are numbered: the root is page 0 and every submenu
/// follows its parent, depth first. For each page, `(parent, children)`, where `children` are the
/// pages its submenu rows open, in the order the rows appear.
pub(crate) type Plan = Vec<(Option<usize>, Vec<usize>)>;

/// Numbers the pages of a tree of menus, given by how many submenus each has: `shape(menu)` lists
/// the menu's submenus in order.
pub(crate) fn plan<T>(root: &T, subs: &dyn Fn(&T) -> Vec<&T>) -> Plan {
    fn walk<T>(m: &T, parent: Option<usize>, subs: &dyn Fn(&T) -> Vec<&T>, out: &mut Plan) -> usize {
        let at = out.len();
        out.push((parent, Vec::new()));
        for child in subs(m) {
            let c = walk(child, Some(at), subs, out);
            out[at].1.push(c);
        }
        at
    }
    let mut out = Vec::new();
    walk(root, None, subs, &mut out);
    out
}

/// How far each page of a slide is from rest, in pixels, `progress` of the way through (0 to 1) on
/// a page `width` wide: `(leaving, arriving)`. Going deeper, the old page leaves to the start and
/// the new one arrives from the end; coming back is the reverse.
pub(crate) fn slide_offsets(progress: f32, forward: bool, width: f32) -> (f32, f32) {
    let p = progress.clamp(0.0, 1.0);
    let (leave, arrive) = (-width * p, width * (1.0 - p));
    if forward { (leave, arrive) } else { (-leave, -arrive) }
}

/// How tall the stack of pages is, `progress` of the way through a slide from page `from` to page
/// `to` (0 to 1): the height of the page that is showing, eased between the two heights while
/// they slide, so the menu is always as tall as its contents and never as tall as the tallest page.
pub(crate) fn blended_height(heights: &[f32], from: usize, to: usize, progress: f32) -> f32 {
    let at = |i: usize| heights.get(i).copied().unwrap_or(0.0);
    let p = progress.clamp(0.0, 1.0);
    at(from) + (at(to) - at(from)) * p
}

// ---- a row that opens a page ---------------------------------------------------------------

/// A menu row that opens page `target` when it is clicked. See the [module](self).
pub(crate) struct NavRow<'a, M> {
    content: Element<'a, M, Theme>,
    target: usize,
    request: Request,
}

impl<'a, M> NavRow<'a, M> {
    pub(crate) fn new(
        content: impl Into<Element<'a, M, Theme>>,
        target: usize,
        request: &Request,
    ) -> Self {
        Self { content: content.into(), target, request: request.clone() }
    }
}

#[derive(Default)]
struct NavRowState {
    pressed: bool,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for NavRow<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<NavRowState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(NavRowState::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // A transparent wrapper: the row is its content's layout.
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<NavRowState>();
        let over = cursor.is_over(layout.bounds());
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over => {
                state.pressed = true;
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.pressed && over {
                    self.request.ask(self.target);
                    // Not captured: the popover must not take this click for a choice and close.
                    shell.request_redraw();
                }
                state.pressed = false;
            }
            _ => {}
        }
    }
    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let over = cursor.is_over(bounds);
        let status = match (over, tree.state.downcast_ref::<NavRowState>().pressed) {
            (true, true) => Status::Pressed,
            (true, false) => Status::Hovered,
            _ => Status::Active,
        };
        // The same look as the other rows of the menu.
        let look = sb::menu_item(false)(theme, status);
        if let Some(background) = look.background {
            renderer.fill_quad(
                renderer::Quad { bounds, border: look.border, shadow: Default::default(), snap: false },
                background,
            );
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style { text_color: look.text_color },
            layout,
            cursor,
            viewport,
        );
    }
}

impl<'a, M: 'a> From<NavRow<'a, M>> for Element<'a, M, Theme> {
    fn from(r: NavRow<'a, M>) -> Self {
        Element::new(r)
    }
}

// ---- the stack of pages --------------------------------------------------------------------

/// Pages of a menu, one showing at a time, sliding as rows ask for another. See the
/// [module](self).
pub(crate) struct MenuPages<'a, M> {
    pages: Vec<Element<'a, M, Theme>>,
    request: Request,
    width: f32,
    /// The page to show on first draw. Anything but the root slides in, as if it had been asked for.
    initial: usize,
}

impl<'a, M> MenuPages<'a, M> {
    pub(crate) fn new(
        pages: Vec<Element<'a, M, Theme>>,
        request: &Request,
        width: f32,
        initial: usize,
    ) -> Self {
        Self { initial: initial.min(pages.len().saturating_sub(1)), pages, request: request.clone(), width }
    }
}

struct PagesState {
    current: usize,
    /// The page that is leaving while one slides in.
    from: usize,
    /// Whether the slide is towards a deeper page.
    forward: bool,
    progress: Progress,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for MenuPages<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PagesState>()
    }
    fn state(&self) -> tree::State {
        let slides_in = self.initial != 0 && Trigger::PageChange.animated();
        tree::State::new(PagesState {
            current: self.initial,
            from: 0,
            forward: true,
            progress: Progress::new(if slides_in { 0.0 } else { 1.0 }),
        })
    }
    fn children(&self) -> Vec<Tree> {
        self.pages.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        let pages: Vec<_> = self.pages.iter().map(|p| p.as_widget()).collect();
        tree.diff_children(&pages);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.width), Length::Shrink)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fixed(self.width));
        let nodes: Vec<layout::Node> = self
            .pages
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(page, t)| page.as_widget_mut().layout(t, renderer, &limits))
            .collect();
        // As tall as the page that is showing, easing between two heights while pages slide.
        let heights: Vec<f32> = nodes.iter().map(|n| n.size().height).collect();
        let state = tree.state.downcast_ref::<PagesState>();
        let height = blended_height(&heights, state.from, state.current, state.progress.shown);
        layout::Node::with_children(Size::new(self.width, height), nodes)
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
        let animated = Trigger::PageChange.animated();
        {
            let state = tree.state.downcast_mut::<PagesState>();
            // The height changes with the slide, so each frame of it lays the menu out again.
            advance(&mut state.progress, event, 1.0, Trigger::PageChange.spec(Spec::STYLE), shell, true);
        }
        let (current, settled) = {
            let s = tree.state.downcast_ref::<PagesState>();
            (s.current, s.progress.shown >= 0.999)
        };
        if settled {
            if let Some(page_layout) = layout.children().nth(current) {
                self.pages[current].as_widget_mut().update(
                    &mut tree.children[current],
                    event,
                    page_layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        } else if matches!(event, Event::Mouse(_) | Event::Touch(_)) && cursor.is_over(layout.bounds()) {
            shell.capture_event(); // no clicks while a page slides
        }
        // A row asked for another page: slide to it.
        if let Some(target) = self.request.take()
            && target < self.pages.len()
        {
            let state = tree.state.downcast_mut::<PagesState>();
            if target != state.current {
                state.from = state.current;
                state.forward = target > state.current;
                state.current = target;
                state.progress = Progress::new(if animated { 0.0 } else { 1.0 });
                shell.invalidate_layout();
                shell.request_redraw();
            }
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
        let current = tree.state.downcast_ref::<PagesState>().current;
        match layout.children().nth(current) {
            Some(l) => self.pages[current].as_widget().mouse_interaction(
                &tree.children[current],
                l,
                cursor,
                viewport,
                renderer,
            ),
            None => mouse::Interaction::None,
        }
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        let current = tree.state.downcast_ref::<PagesState>().current;
        if let Some(l) = layout.children().nth(current) {
            self.pages[current].as_widget_mut().operate(&mut tree.children[current], l, renderer, operation);
        }
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
        let state = tree.state.downcast_ref::<PagesState>();
        let bounds = layout.bounds();
        let draw_page = |r: &mut iced::Renderer, page: usize, dx: f32| {
            if let Some(l) = layout.children().nth(page) {
                draw_translated(r, bounds, viewport, Vector::new(dx, 0.0), |r, view| {
                    self.pages[page].as_widget().draw(&tree.children[page], r, theme, style, l, cursor, view);
                });
            }
        };
        let p = state.progress.shown;
        if p >= 0.999 {
            draw_page(renderer, state.current, 0.0);
            return;
        }
        let (leaving, arriving) = slide_offsets(p, state.forward, bounds.width);
        renderer.with_layer(bounds, |r| {
            draw_page(r, state.from, leaving);
            draw_page(r, state.current, arriving);
        });
    }
}

impl<'a, M: 'a> From<MenuPages<'a, M>> for Element<'a, M, Theme> {
    fn from(p: MenuPages<'a, M>) -> Self {
        Element::new(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A little tree of menus to number: (name, submenus).
    #[allow(dead_code)]
    struct T(&'static str, Vec<T>);

    fn subs(t: &T) -> Vec<&T> {
        t.1.iter().collect()
    }

    #[test]
    fn pages_are_numbered_depth_first_with_parents_and_children() {
        let root = T("root", vec![T("a", vec![T("a1", vec![])]), T("b", vec![])]);
        let p = plan(&root, &subs);
        // root=0, a=1, a1=2, b=3
        assert_eq!(p.len(), 4);
        assert_eq!(p[0], (None, vec![1, 3]));
        assert_eq!(p[1], (Some(0), vec![2]));
        assert_eq!(p[2], (Some(1), vec![]));
        assert_eq!(p[3], (Some(0), vec![]));
    }

    #[test]
    fn the_menu_is_as_tall_as_the_page_it_shows_not_the_tallest() {
        let heights = [90.0, 170.0, 130.0];
        // At rest each page has its own height.
        assert_eq!(blended_height(&heights, 0, 0, 1.0), 90.0);
        assert_eq!(blended_height(&heights, 0, 1, 1.0), 170.0);
        assert_eq!(blended_height(&heights, 1, 2, 1.0), 130.0);
        // Sliding from the short page to the tall one eases the height between them.
        assert_eq!(blended_height(&heights, 0, 1, 0.0), 90.0);
        assert_eq!(blended_height(&heights, 0, 1, 0.5), 130.0);
        // And back again, from the tall page to the short one.
        assert_eq!(blended_height(&heights, 1, 0, 0.5), 130.0);
        // An overshooting spring never takes it outside the two heights.
        assert_eq!(blended_height(&heights, 0, 1, 1.2), 170.0);
        // A page that is not there is no height at all, not a panic.
        assert_eq!(blended_height(&heights, 0, 9, 1.0), 0.0);
    }

    #[test]
    fn a_menu_without_submenus_is_one_page() {
        let p = plan(&T("root", vec![]), &subs);
        assert_eq!(p, vec![(None, vec![])]);
    }

    #[test]
    fn going_deeper_slides_the_old_page_out_and_the_new_one_in() {
        // At rest: the old page is gone to the start, the new one is home.
        assert_eq!(slide_offsets(1.0, true, 260.0), (-260.0, 0.0));
        // Not started: the old page is home, the new one waits at the end.
        assert_eq!(slide_offsets(0.0, true, 260.0), (0.0, 260.0));
        // Halfway they are each half a page along.
        assert_eq!(slide_offsets(0.5, true, 260.0), (-130.0, 130.0));
    }

    #[test]
    fn coming_back_is_the_mirror_image() {
        assert_eq!(slide_offsets(0.5, false, 260.0), (130.0, -130.0));
        assert_eq!(slide_offsets(1.0, false, 260.0), (260.0, 0.0));
        // A spring can overshoot; the offsets never leave the page.
        assert_eq!(slide_offsets(1.2, true, 260.0), (-260.0, 0.0));
    }
}
