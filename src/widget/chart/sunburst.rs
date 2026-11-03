//! `Sunburst`: a tree as rings. The first ring is the root's children; each next ring splits
//! its parent's span among the children.

use super::{Formatter, State, assemble, fmt_or_default, track};
use crate::Theme;
use crate::components::chart::path::to_path;
use crate::components::chart::tooltip::{self, Row};
use crate::components::chart::{Anchor, color, label, text_width};
use crate::primitives::chart::hierarchy::{Cell, Node, partition};
use crate::primitives::chart::pie::arc;
use crate::primitives::chart::shape::Pt;
use crate::widget::Element;
use iced::mouse::Cursor;
use iced::widget::canvas::{self, Fill, Frame, Geometry, Stroke};
use iced::{Length, Point, Rectangle, Renderer, Size};
use std::f32::consts::TAU;
use std::rc::Rc;

/// See the [module](self).
pub struct Sunburst {
    root: Node,
    hole: f32,
    legend: bool,
    width: Length,
    height: Length,
    format: Option<Formatter>,
    cells: Vec<Cell>,
}

impl Sunburst {
    /// The tree to show. The root itself is the middle; its children make the first ring.
    pub fn new(root: Node) -> Self {
        let cells = partition(&root);
        Self {
            root,
            hole: 0.25,
            legend: true,
            width: Length::Fill,
            height: Length::Fixed(300.0),
            format: None,
            cells,
        }
    }
    /// The middle hole as a fraction of the radius (`0..1`).
    pub fn hole(mut self, hole: f32) -> Self {
        self.hole = hole.clamp(0.0, 0.9);
        self
    }
    /// The legend of the root's children (on by default).
    pub fn legend(mut self, legend: bool) -> Self {
        self.legend = legend;
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
    /// How values are written in the tooltip and the middle.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }
}

struct Geo {
    center: Pt,
    /// Where the first ring starts.
    r0: f32,
    /// The thickness of a ring.
    ring: f32,
}

impl Sunburst {
    fn geo(&self, size: Size) -> Geo {
        let outer = (size.width.min(size.height) / 2.0 - 8.0).max(0.0);
        let r0 = outer * self.hole;
        Geo {
            center: Pt::new(size.width / 2.0, size.height / 2.0),
            r0,
            ring: (outer - r0) / self.root.height().max(1) as f32,
        }
    }

    /// The cell under `p` (never the root).
    fn at(&self, size: Size, p: Pt) -> Option<usize> {
        let g = self.geo(size);
        let (dx, dy) = (p.x - g.center.x, p.y - g.center.y);
        let r = dx.hypot(dy);
        if g.ring <= 0.0 || r < g.r0 {
            return None;
        }
        let depth = ((r - g.r0) / g.ring).floor() as usize + 1;
        let x = dx.atan2(-dy).rem_euclid(TAU) / TAU;
        self.cells
            .iter()
            .position(|c| c.depth == depth && c.value > 0.0 && x >= c.x0 && x < c.x1)
    }

    /// The color of a cell: its branch's hue, paler the deeper it is.
    fn fill(&self, theme: &Theme, c: &Cell) -> iced::Color {
        let base = color::series(theme, c.branch);
        crate::color::mix(
            base,
            theme.semantic.view.bg,
            (c.depth.saturating_sub(1)) as f32 * 0.16,
        )
    }
}

impl<M> canvas::Program<M, Theme> for Sunburst {
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
        let g = self.geo(bounds.size());
        let mut frame = Frame::new(renderer, bounds.size());
        let fg = theme.semantic.window.fg;
        for (i, c) in self.cells.iter().enumerate() {
            if c.depth == 0 || c.x1 <= c.x0 {
                continue;
            }
            let (r_in, r_out) = (
                g.r0 + (c.depth - 1) as f32 * g.ring,
                g.r0 + c.depth as f32 * g.ring,
            );
            let mut col = self.fill(theme, c);
            if state.hover.is_some_and(|h| {
                h != i
                    && !self.cells[h].path.starts_with(&c.path)
                    && !c.path.starts_with(&self.cells[h].path)
            }) {
                col = crate::color::alpha(col, 0.4);
            }
            let shape = to_path(&arc(g.center, r_in, r_out, c.x0 * TAU, c.x1 * TAU));
            frame.fill(&shape, Fill::from(col));
            frame.stroke(
                &shape,
                Stroke::default()
                    .with_width(1.0)
                    .with_color(theme.semantic.view.bg),
            );
            // The name, when the arc is long enough to hold it.
            let mid_r = (r_in + r_out) / 2.0;
            let chord = (c.x1 - c.x0) * TAU * mid_r;
            if chord > text_width(&c.name) + 6.0 && g.ring >= 16.0 {
                let p = crate::primitives::chart::pie::polar(
                    g.center,
                    mid_r,
                    (c.x0 + c.x1) * TAU / 2.0,
                );
                label(
                    &mut frame,
                    c.name.as_str(),
                    Point::new(p.x, p.y),
                    Anchor::Middle,
                    crate::color::readable_on(col),
                );
            }
        }
        // The middle: what the pointer is over, else the whole.
        let (name, value) = match state.hover {
            Some(i) => (self.cells[i].name.as_str(), self.cells[i].value),
            None => (self.root.name.as_str(), self.cells[0].value),
        };
        if g.r0 > 24.0 {
            label(
                &mut frame,
                name,
                Point::new(g.center.x, g.center.y - 8.0),
                Anchor::Middle,
                fg,
            );
            label(
                &mut frame,
                fmt_or_default(&self.format, value),
                Point::new(g.center.x, g.center.y + 8.0),
                Anchor::Middle,
                theme.semantic.dim(),
            );
        }
        if let (Some(i), Some(at)) = (state.hover, cursor.position_in(bounds)) {
            let c = &self.cells[i];
            let total = self.cells[0].value;
            let rows = [
                Row::new(
                    Some(self.fill(theme, c)),
                    "Value",
                    fmt_or_default(&self.format, c.value),
                ),
                Row::new(None, "Of total", format!("{:.1}%", c.value / total * 100.0)),
            ];
            tooltip::draw(
                &mut frame,
                theme,
                at,
                Some(&c.path.join(" \u{203a} ")),
                &rows,
            );
        }
        vec![frame.into_geometry()]
    }
}

impl<'a, M: 'a> From<Sunburst> for Element<'a, M> {
    fn from(c: Sunburst) -> Self {
        let names = c
            .legend
            .then(|| c.root.children.iter().map(|n| n.name.clone()).collect());
        let (w, h) = (c.width, c.height);
        assemble(c, w, h, names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> Sunburst {
        Sunburst::new(Node::branch(
            "all",
            [
                Node::branch("a", [Node::leaf("a1", 1.0), Node::leaf("a2", 1.0)]),
                Node::leaf("b", 2.0),
            ],
        ))
    }

    #[test]
    fn a_point_finds_its_ring_and_arc() {
        let c = chart();
        let size = Size::new(216.0, 216.0);
        let g = c.geo(size);
        // Just right of twelve o'clock, in the first ring: branch "a" (the first half).
        let p = Pt::new(g.center.x + 2.0, g.center.y - g.r0 - g.ring / 2.0);
        assert_eq!(c.cells[c.at(size, p).unwrap()].name, "a");
        // The same angle one ring out: its first child.
        let p = Pt::new(g.center.x + 2.0, g.center.y - g.r0 - g.ring * 1.5);
        assert_eq!(c.cells[c.at(size, p).unwrap()].name, "a1");
        // The hole and the outside are nothing.
        assert_eq!(c.at(size, g.center), None);
        assert_eq!(c.at(size, Pt::new(0.0, 0.0)), None);
    }

    #[test]
    fn rings_share_the_radius() {
        let g = chart().geo(Size::new(216.0, 216.0));
        assert_eq!(g.r0 + 2.0 * g.ring, 100.0);
    }
}
