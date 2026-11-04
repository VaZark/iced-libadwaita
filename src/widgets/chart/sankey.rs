//! `Sankey`: flows between nodes, in layers, each link as thick as its value.

use super::{Formatter, State, assemble, fmt_or_default, track};
use crate::Element;
use crate::Theme;
use crate::components::chart::path::to_path;
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{Anchor, color, ink, label, text_width};
use crate::primitives::chart::sankey::{Link, Sankey as Layout, sankey};
use crate::primitives::chart::shape::Pt;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Path, Stroke};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::rc::Rc;

const NODE_WIDTH: f32 = 12.0;
const NODE_PAD: f32 = 10.0;
const ITERATIONS: usize = 6;
const MARGIN: f32 = 8.0;

/// See the [module](self).
pub struct Sankey {
    names: Vec<String>,
    links: Vec<Link>,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    tooltip: bool,
    labels: bool,
}

impl Default for Sankey {
    fn default() -> Self {
        Self::new()
    }
}

impl Sankey {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            links: Vec::new(),
            width: Length::Fill,
            height: Length::Fixed(300.0),
            format: None,
            tooltip: true,
            labels: true,
        }
    }
    fn index(&mut self, name: &str) -> usize {
        match self.names.iter().position(|n| n == name) {
            Some(i) => i,
            None => {
                self.names.push(name.to_string());
                self.names.len() - 1
            }
        }
    }
    /// Declares a node, to fix its color and order before links name it.
    pub fn node(mut self, name: impl AsRef<str>) -> Self {
        self.index(name.as_ref());
        self
    }
    /// A flow of `value` from the node `from` to the node `to`. Nodes are made as they are named.
    pub fn link(mut self, from: impl AsRef<str>, to: impl AsRef<str>, value: f32) -> Self {
        let (s, t) = (self.index(from.as_ref()), self.index(to.as_ref()));
        self.links.push(Link::new(s, t, value));
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
    /// The tooltip that follows the pointer (on by default).
    pub fn tooltip(mut self, show: bool) -> Self {
        self.tooltip = show;
        self
    }
    /// The text on the marks (on by default).
    pub fn labels(mut self, show: bool) -> Self {
        self.labels = show;
        self
    }
    /// How values are written in the tooltip.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }

    /// The layout, with room at both ends for the labels of the first and last layers.
    fn layout(&self, size: Size) -> (Layout, f32) {
        let n = self.names.len();
        let probe = sankey(n, &self.links, 100.0, 100.0, NODE_WIDTH, NODE_PAD, 0);
        let last = probe.nodes.iter().map(|b| b.depth).max().unwrap_or(0);
        let widest = |depth: usize| {
            probe
                .nodes
                .iter()
                .filter(|b| b.depth == depth)
                .map(|b| text_width(&self.names[b.index]))
                .fold(0.0f32, f32::max)
        };
        let right = if last > 0 { widest(last) + 8.0 } else { 0.0 };
        let left = if last > 0 { widest(0) + 8.0 } else { 0.0 };
        let (w, h) = (
            size.width - 2.0 * MARGIN - right - left,
            size.height - 2.0 * MARGIN,
        );
        let mut l = sankey(
            n,
            &self.links,
            w.max(0.0),
            h.max(0.0),
            NODE_WIDTH,
            NODE_PAD,
            ITERATIONS,
        );
        for b in &mut l.nodes {
            b.x0 += MARGIN + left;
            b.x1 += MARGIN + left;
            b.y0 += MARGIN;
            b.y1 += MARGIN;
        }
        for b in &mut l.links {
            b.x0 += MARGIN + left;
            b.x1 += MARGIN + left;
            b.y0 += MARGIN;
            b.y1 += MARGIN;
        }
        (l, left + right)
    }

    /// What is at `p`: a node `i` is `i`, a link `j` is `names + j`. Nodes win.
    fn at(&self, size: Size, p: Pt) -> Option<usize> {
        let (l, _) = self.layout(size);
        l.nodes.iter().position(|b| b.contains(p)).or_else(|| {
            l.links
                .iter()
                .position(|b| b.contains(p))
                .map(|j| self.names.len() + j)
        })
    }
}

impl<M> canvas::Program<M, Theme> for Sankey {
    type State = State;

    fn update(
        &self,
        state: &mut State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<M>> {
        track(&mut state.hover, event, || {
            let p = cursor.position_in(bounds)?;
            self.at(bounds.size(), Pt::new(p.x, p.y))
        })
    }

    fn draw(
        &self,
        state: &State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let (l, _) = self.layout(bounds.size());
        let n = self.names.len();
        let mut frame = Frame::new(renderer, bounds.size());
        let (text, _) = ink(theme);
        let focus = |i: usize, b: &crate::primitives::chart::sankey::Band| match state.hover {
            None => 0.35,
            Some(h) if h == n + i => 0.7,
            Some(h) if h < n && (b.source == h || b.target == h) => 0.7,
            Some(_) => 0.1,
        };
        for (i, b) in l.links.iter().enumerate() {
            frame.stroke(
                &to_path(&b.path()),
                Stroke::default()
                    .with_width(b.width)
                    .with_color(crate::color::alpha(
                        color::series(theme, b.source),
                        focus(i, b),
                    )),
            );
        }
        for b in &l.nodes {
            let col = color::series(theme, b.index);
            frame.fill(
                &Path::rectangle(
                    Point::new(b.x0, b.y0),
                    Size::new(b.x1 - b.x0, (b.y1 - b.y0).max(1.0)),
                ),
                Fill::from(col),
            );
            let mid = Point::new(0.0, (b.y0 + b.y1) / 2.0);
            // The first layer's labels sit to its left and every other's to its right, in the
            // room `layout` keeps at both ends.
            let (x, anchor) = if b.depth == 0 && l.nodes.iter().any(|o| o.depth > 0) {
                (b.x0 - 6.0, Anchor::End)
            } else {
                (b.x1 + 6.0, Anchor::Start)
            };
            if self.labels {
                label(
                    &mut frame,
                    self.names[b.index].as_str(),
                    Point::new(x, mid.y),
                    anchor,
                    text,
                );
            }
        }
        if let (Some(h), Some(at), true) = (state.hover, cursor.position_in(bounds), self.tooltip) {
            if h < n {
                let b = l.nodes[h];
                let rows = [Row::new(
                    Some(color::series(theme, h)),
                    "Total",
                    fmt_or_default(&self.format, b.value),
                )];
                tooltip::draw(&mut frame, theme, at, Some(self.names[h].as_str()), &rows);
            } else if let Some(b) = l.links.get(h - n) {
                let title = format!("{} \u{2192} {}", self.names[b.source], self.names[b.target]);
                let rows = [Row::new(
                    Some(color::series(theme, b.source)),
                    "Flow",
                    fmt_or_default(&self.format, b.value),
                )];
                tooltip::draw(&mut frame, theme, at, Some(&title), &rows);
            }
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<Sankey> for Element<'a, M> {
    fn from(c: Sankey) -> Self {
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Sankey {
        Sankey::new()
            .link("A", "B", 5.0)
            .link("A", "C", 3.0)
            .link("B", "D", 5.0)
            .link("C", "D", 3.0)
    }

    #[test]
    fn nodes_are_made_as_they_are_named() {
        assert_eq!(s().names, vec!["A", "B", "C", "D"]);
        assert_eq!(s().links.len(), 4);
        let s2 = Sankey::new().node("Z").link("A", "Z", 1.0);
        assert_eq!(s2.names, vec!["Z", "A"]);
    }

    #[test]
    fn the_layout_stays_inside_the_chart() {
        let (l, right) = s().layout(Size::new(400.0, 300.0));
        assert!(right > 0.0);
        for b in &l.nodes {
            assert!(b.x0 >= MARGIN - 1e-3 && b.x1 <= 400.0 - MARGIN + 1e-3);
            assert!(b.y0 >= MARGIN - 1e-3 && b.y1 <= 300.0 - MARGIN + 1e-3);
        }
    }

    #[test]
    fn a_node_is_found_before_a_link() {
        let c = s();
        let (l, _) = c.layout(Size::new(400.0, 300.0));
        let b = l.nodes[0];
        assert_eq!(
            c.at(
                Size::new(400.0, 300.0),
                Pt::new((b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0)
            ),
            Some(0)
        );
        assert_eq!(c.at(Size::new(400.0, 300.0), Pt::new(-5.0, -5.0)), None);
    }
}
