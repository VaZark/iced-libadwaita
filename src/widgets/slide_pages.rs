//! Pages of which one shows, chosen by the app, sliding as it changes: the way a submenu slides in
//! place of its parent (see [`menu_pages`](super::menu_pages)), but with the page named from the
//! outside, so what is shown follows the app's own state (the folder you are in) and the slide
//! follows the change, whoever made it: a press on a row, a back button, a link elsewhere.
//!
//! Going to a page that is *deeper* slides the old one out towards the start and the new one in
//! from the end; going to a shallower one is the mirror image. `depths` says how deep each page is.

use crate::Theme;
use crate::layouts::animation::{Progress, Spec, Trigger};
use crate::layouts::motion::{advance, draw_translated};
use crate::widgets::menu_pages::slide_offsets;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{Element, Event, Length, Rectangle, Size, Vector, mouse};

/// See the [module](self).
pub struct SlidePages<'a, M> {
    pages: Vec<Element<'a, M, Theme>>,
    current: usize,
    depths: Vec<u32>,
}

impl<'a, M> SlidePages<'a, M> {
    /// `pages`, of which page `current` shows.
    pub fn new(pages: Vec<Element<'a, M, Theme>>, current: usize) -> Self {
        Self {
            current: current.min(pages.len().saturating_sub(1)),
            depths: Vec::new(),
            pages,
        }
    }

    /// How deep each page is (the root 0): a change to a deeper page slides forward, to a
    /// shallower one back. Without it, a later page is deeper.
    pub fn depths(mut self, depths: Vec<u32>) -> Self {
        self.depths = depths;
        self
    }

    fn deeper(&self, from: usize, to: usize) -> bool {
        match (self.depths.get(from), self.depths.get(to)) {
            (Some(a), Some(b)) => b > a,
            _ => to > from,
        }
    }
}

struct State {
    current: usize,
    from: usize,
    forward: bool,
    progress: Progress,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for SlidePages<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            current: self.current,
            from: self.current,
            forward: true,
            progress: Progress::new(1.0),
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
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let nodes: Vec<layout::Node> = self
            .pages
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(page, t)| page.as_widget_mut().layout(t, renderer, limits))
            .collect();
        let size = limits.max();
        layout::Node::with_children(size, nodes)
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
        // The app named another page: slide to it.
        {
            let (current, from) = {
                let s = tree.state.downcast_ref::<State>();
                (s.current, s.current)
            };
            if self.current != current {
                let forward = self.deeper(from, self.current);
                let state = tree.state.downcast_mut::<State>();
                state.from = from;
                state.forward = forward;
                state.current = self.current;
                state.progress = Progress::new(if Trigger::PagePush.animated() { 0.0 } else { 1.0 });
                shell.invalidate_layout();
                shell.request_redraw();
            }
        }
        {
            let state = tree.state.downcast_mut::<State>();
            advance(
                &mut state.progress,
                event,
                1.0,
                Trigger::PagePush.spec(Spec::STYLE),
                shell,
                false,
            );
        }
        let (current, settled) = {
            let s = tree.state.downcast_ref::<State>();
            (s.current, s.progress.shown >= 0.999)
        };
        if settled {
            if let Some(l) = layout.children().nth(current) {
                self.pages[current].as_widget_mut().update(
                    &mut tree.children[current],
                    event,
                    l,
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
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let current = tree.state.downcast_ref::<State>().current;
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
        let current = tree.state.downcast_ref::<State>().current;
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
        let state = tree.state.downcast_ref::<State>();
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

impl<'a, M: 'a> From<SlidePages<'a, M>> for Element<'a, M, Theme> {
    fn from(p: SlidePages<'a, M>) -> Self {
        Element::new(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deeper_page_is_forward_and_a_shallower_one_back() {
        let p: SlidePages<'_, ()> = SlidePages::new(Vec::new(), 0).depths(vec![0, 1, 2, 1]);
        assert!(p.deeper(0, 1));
        assert!(p.deeper(1, 2));
        assert!(!p.deeper(2, 3), "3 is a later page but not a deeper one");
        assert!(!p.deeper(1, 0));
        // Without depths a later page is deeper.
        let q: SlidePages<'_, ()> = SlidePages::new(Vec::new(), 0);
        assert!(q.deeper(0, 2) && !q.deeper(2, 0));
    }
}
