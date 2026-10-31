//! The layout and interaction of `AdwToggleGroup`, `AdwViewSwitcher` and the
//! path bar: an exclusive row of segments.
//!
//! Unlike a row of buttons, this widget owns its layout so it always respects
//! the space it is given:
//!
//! * **Fills, `flex: 1`.** With `Length::Fill`, or when its limits demand a
//!   minimum width (e.g. a stacked row asking for 100%), every segment gets an
//!   *equal* share of the width (flex-grow 1, basis 0) and its content is
//!   centered in it, so there is no dead space after the last one. A segment
//!   never shrinks below its own content (min-content): wide ones keep their
//!   size and the rest split what remains. `homogeneous` forces strictly equal
//!   widths regardless of content.
//! * **Never overshoots.** When the natural width exceeds the available width,
//!   segments shrink fairly (short ones keep their size, long ones share the
//!   rest, like CSS `flex-shrink`), and content is clipped to its segment.
//! * **Adapts.** A segment with an icon *and* a label switches to icon-only
//!   when its share is too small for both (`AdwViewSwitcher`'s narrow policy).
//!   The switch is **synchronized**: as soon as one segment needs its compact form,
//!   every segment that has one switches, so the control is never half icons, half
//!   labels.

use crate::style::button as sb;
use crate::style::button::Position;
use crate::{Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::widget::button::Status;
use iced::{mouse, Background, Border, Element, Event, Length, Point, Rectangle, Size};

const TROUGH_PADDING: f32 = 2.0;

/// One segment: its value, full content, and optional compact (icon-only) content.
pub type Segment<'a, T, M> = (T, Element<'a, M, Theme>, Option<Element<'a, M, Theme>>);

/// An exclusive segmented control. Build it with
/// [`toggle_group`](super::toggle_group) rather than directly.
pub struct Segmented<'a, T, M> {
    values: Vec<T>,
    /// `2 * n` elements: the full content of each segment, then its compact
    /// (icon-only) variant, or a placeholder when it has none.
    children: Vec<Element<'a, M, Theme>>,
    has_compact: Vec<bool>,
    active: Option<usize>,
    on_select: Box<dyn Fn(T) -> M + 'a>,
    width: Length,
    homogeneous: bool,
    compact: Option<bool>,
    metrics: Metrics,
    // Results of the last layout (relative to the widget's bounds).
    rects: Vec<Rectangle>,
    use_compact: Vec<bool>,
}

#[derive(Default)]
struct State {
    pressed: Option<usize>,
}

impl<'a, T: Clone, M: 'a> Segmented<'a, T, M> {
    pub fn new(
        segments: Vec<Segment<'a, T, M>>,
        active: Option<usize>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        let n = segments.len();
        let mut values = Vec::with_capacity(n);
        let mut full = Vec::with_capacity(n);
        let mut compact = Vec::with_capacity(n);
        let mut has_compact = Vec::with_capacity(n);
        for (v, f, c) in segments {
            values.push(v);
            full.push(f);
            has_compact.push(c.is_some());
            compact.push(c.unwrap_or_else(|| iced::widget::space().into()));
        }
        full.extend(compact);
        Self {
            values,
            children: full,
            has_compact,
            active,
            on_select: Box::new(on_select),
            width: Length::Shrink,
            homogeneous: false,
            compact: None,
            metrics: Metrics::desktop(),
            rects: Vec::new(),
            use_compact: vec![false; n],
        }
    }

    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    /// All segments the same width (`AdwToggleGroup:homogeneous`): the widest
    /// content sets it, or the equal share when filling, whichever is larger.
    pub fn homogeneous(mut self, h: bool) -> Self {
        self.homogeneous = h;
        self
    }

    /// Which form the segments take. `Some(true)` shows every segment that has an icon-only form
    /// as that icon; `Some(false)` always shows the full content. Pass the breakpoint here
    /// (`window::is_mobile(width)`) so the control follows the window's mode. `None` (the default)
    /// decides from the space: icons as soon as any segment would be squeezed. Either way the
    /// segments switch together, never half icons and half labels.
    pub fn compact(mut self, compact: Option<bool>) -> Self {
        self.compact = compact;
        self
    }

    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn position(i: usize, n: usize) -> Position {
        match (i, n) {
            (_, 1) => Position::Only,
            (0, _) => Position::Start,
            (i, n) if i + 1 == n => Position::End,
            _ => Position::Middle,
        }
    }

    fn hit(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<usize> {
        let p = cursor.position_in(bounds)?;
        self.rects.iter().position(|r| r.contains(p))
    }
}

/// Distributes `target` among segments with natural widths `nat`: stretch evenly
/// when there is room, otherwise shrink fairly (segments below the fair share
/// keep their natural width, the rest split what remains).
fn allot(nat: &[f32], target: f32) -> Vec<f32> {
    let n = nat.len();
    if n == 0 {
        return Vec::new();
    }
    let total: f32 = nat.iter().sum();
    if total <= target {
        let extra = (target - total) / n as f32;
        return nat.iter().map(|w| w + extra).collect();
    }
    let mut out = nat.to_vec();
    let mut fixed = vec![false; n];
    let mut remaining = target.max(0.0);
    let mut free = n;
    loop {
        let fair = remaining / free as f32;
        let mut changed = false;
        for i in 0..n {
            if !fixed[i] && nat[i] <= fair {
                fixed[i] = true;
                out[i] = nat[i];
                remaining -= nat[i];
                free -= 1;
                changed = true;
            }
        }
        if !changed || free == 0 {
            break;
        }
    }
    if free > 0 {
        let share = remaining / free as f32;
        for i in 0..n {
            if !fixed[i] {
                out[i] = share;
            }
        }
    }
    out
}

/// `flex: 1` with `min-width: auto`: every segment gets an equal share of
/// `target`, except those whose content is wider, which keep their content width
/// (their share is then redistributed equally among the rest).
fn allot_flex(nat: &[f32], target: f32) -> Vec<f32> {
    let n = nat.len();
    if n == 0 {
        return Vec::new();
    }
    let mut out = vec![0.0; n];
    let mut fixed = vec![false; n];
    let mut remaining = target.max(0.0);
    let mut free = n;
    loop {
        let share = remaining / free as f32;
        let mut changed = false;
        for i in 0..n {
            if !fixed[i] && nat[i] > share {
                fixed[i] = true;
                out[i] = nat[i];
                remaining -= nat[i];
                free -= 1;
                changed = true;
            }
        }
        if !changed || free == 0 {
            break;
        }
    }
    if free > 0 {
        let share = remaining / free as f32;
        for i in 0..n {
            if !fixed[i] {
                out[i] = share;
            }
        }
    }
    out
}

/// Whether the control must switch to compact (icon-only) segments: true as soon as
/// any segment that *has* a compact form would be squeezed below its natural width.
/// The caller then switches all segments that have one (all icons or all content).
fn needs_compact(widths: &[f32], nat_full: &[f32], nat_compact: &[Option<f32>]) -> bool {
    (0..widths.len()).any(|i| widths[i] + 0.5 < nat_full[i] && nat_compact[i].is_some())
}

/// Whether the segments take their compact form: as told by the breakpoint (`forced`), else
/// because the space is short.
fn use_compact(forced: Option<bool>, widths: &[f32], nat_full: &[f32], nat_compact: &[Option<f32>]) -> bool {
    forced.unwrap_or_else(|| needs_compact(widths, nat_full, nat_compact))
}

/// Chooses the distribution: equal `flex: 1` shares when there is room to
/// stretch, natural widths when exactly fitting, fair shrinking when short.
fn distribute(nat: &[f32], target: f32, homogeneous: bool) -> Vec<f32> {
    let total: f32 = nat.iter().sum();
    if homogeneous {
        let each = (target / nat.len().max(1) as f32).max(0.0);
        // Equal widths; never wider than the container, content clips if needed.
        return vec![each; nat.len()];
    }
    if target > total + 0.5 {
        allot_flex(nat, target)
    } else {
        allot(nat, target)
    }
}

impl<'a, T: Clone, M: 'a> Widget<M, Theme, iced::Renderer> for Segmented<'a, T, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children.iter().map(|c| c.as_widget()).collect::<Vec<_>>());
    }

    fn size(&self) -> Size<Length> {
        let m = &self.metrics;
        Size::new(self.width, Length::Fixed(m.button_min_height))
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let m = self.metrics;
        let n = self.values.len();
        let seg_h = m.button_min_height - 2.0 * TROUGH_PADDING;
        let hpad = m.button_padding.x() / 2.0;
        let max_w = {
            let w = limits.max().width;
            if w.is_finite() { w } else { f32::MAX / 4.0 }
        };
        let min_w = limits.min().width;

        // Natural width of each segment's full and compact content.
        let natural = layout::Limits::new(Size::ZERO, Size::new(max_w, seg_h));
        let mut nat_full = Vec::with_capacity(n);
        let mut nat_compact = vec![None; n];
        // Content widths without padding, to decide below whether a segment must be
        // re-laid-out narrower than it was measured.
        let mut content_full = Vec::with_capacity(n);
        let mut content_compact = vec![0.0f32; n];
        for i in 0..n {
            let w = self.children[i].as_widget_mut().layout(&mut tree.children[i], renderer, &natural);
            content_full.push(w.size().width);
            nat_full.push(w.size().width + 2.0 * hpad);
            if self.has_compact[i] {
                let c = self.children[n + i]
                    .as_widget_mut()
                    .layout(&mut tree.children[n + i], renderer, &natural);
                content_compact[i] = c.size().width;
                nat_compact[i] = Some(c.size().width + 2.0 * hpad);
            }
        }

        // Homogeneous: every segment is as wide as the widest content.
        if self.homogeneous {
            let widest = nat_full.iter().copied().fold(0.0, f32::max);
            nat_full.iter_mut().for_each(|w| *w = widest);
            nat_compact.iter_mut().flatten().for_each(|w| *w = widest.min(*w));
        }
        let avail = (max_w - 2.0 * TROUGH_PADDING).max(0.0);
        let total: f32 = nat_full.iter().sum();
        let fills = matches!(self.width, Length::Fill | Length::FillPortion(_));
        let target = if fills && max_w < f32::MAX / 8.0 {
            avail
        } else {
            total.max((min_w - 2.0 * TROUGH_PADDING).max(0.0)).min(avail)
        };

        // First pass with full content. If *any* segment would be squeezed below its
        // natural width and has a compact (icon-only) form, switch every segment
        // that has one, so the control reads uniformly: all icons or all full content.
        let mut nat = nat_full.clone();
        let mut widths = distribute(&nat, target, self.homogeneous);
        self.use_compact = vec![false; n];
        if use_compact(self.compact, &widths, &nat_full, &nat_compact) {
            for i in 0..n {
                if let Some(c) = nat_compact[i] {
                    self.use_compact[i] = true;
                    nat[i] = c;
                }
            }
            widths = distribute(&nat, target, self.homogeneous);
        }

        // Lay the chosen content out inside its allotted width, centered.
        self.rects.clear();
        let mut nodes = Vec::with_capacity(n);
        let mut x = TROUGH_PADDING;
        for i in 0..n {
            let w = widths[i];
            let idx = if self.use_compact[i] { n + i } else { i };
            let room = (w - 2.0 * hpad).max(0.0);
            let content = if self.use_compact[i] { content_compact[i] } else { content_full[i] };
            // Lay out at the *same* bounds used for measuring unless the content has to
            // be squeezed: re-laying text out at different bounds mutates its paragraph
            // every frame (and invalidates the renderer's references to it).
            let limits = if content <= room + 0.5 {
                natural
            } else {
                layout::Limits::new(Size::ZERO, Size::new(room, seg_h))
            };
            let node = self.children[idx].as_widget_mut().layout(&mut tree.children[idx], renderer, &limits);
            let size = node.size();
            let pos = Point::new(x + (w - size.width) / 2.0, TROUGH_PADDING + (seg_h - size.height) / 2.0);
            nodes.push(node.move_to(pos));
            self.rects.push(Rectangle::new(Point::new(x, TROUGH_PADDING), Size::new(w, seg_h)));
            x += w;
        }

        let width = if fills && max_w < f32::MAX / 8.0 { max_w } else { x + TROUGH_PADDING };
        layout::Node::with_children(Size::new(width, m.button_min_height), nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let hit = self.hit(bounds, cursor);
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if hit.is_some() => {
                state.pressed = hit;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.pressed.is_some() => {
                if hit == state.pressed
                    && let Some(i) = hit
                {
                    shell.publish((self.on_select)(self.values[i].clone()));
                }
                state.pressed = None;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => shell.request_redraw(),
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.hit(layout.bounds(), cursor).is_some() {
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
        let n = self.values.len();
        let radius = if theme.metrics.radius_m > 0.0 { theme.metrics.radius_m + TROUGH_PADDING } else { 0.0 };

        // Trough.
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border::default().rounded(radius),
                shadow: Default::default(),
                snap: false,
            },
            Background::Color(crate::color::alpha(theme.semantic.window.fg, theme.semantic.opacity.control)),
        );

        let hover = self.hit(bounds, cursor);
        let pressed = tree.state.downcast_ref::<State>().pressed;
        for (i, node) in layout.children().enumerate() {
            let rel = self.rects[i];
            let seg = Rectangle { x: rel.x + bounds.x, y: rel.y + bounds.y, ..rel };
            let status = if pressed == Some(i) {
                Status::Pressed
            } else if hover == Some(i) {
                Status::Hovered
            } else {
                Status::Active
            };
            let st = sb::toggle(self.active == Some(i), Self::position(i, n))(theme, status);
            if let Some(bg) = st.background {
                renderer.fill_quad(
                    Quad { bounds: seg, border: st.border, shadow: st.shadow, snap: false },
                    bg,
                );
            }
            let idx = if self.use_compact[i] { n + i } else { i };
            // Clip to the segment so squeezed content can never spill over a neighbor.
            renderer.with_layer(seg, |r| {
                self.children[idx].as_widget().draw(
                    &tree.children[idx],
                    r,
                    theme,
                    &renderer::Style { text_color: st.text_color },
                    node,
                    cursor,
                    viewport,
                );
            });
        }
    }
}

impl<'a, T: Clone + 'a, M: 'a> From<Segmented<'a, T, M>> for Element<'a, M, Theme> {
    fn from(s: Segmented<'a, T, M>) -> Self {
        Element::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::allot;

    #[test]
    fn allot_adds_equal_extra_when_there_is_room() {
        let w = allot(&[40.0, 60.0], 140.0);
        assert_eq!(w, vec![60.0, 80.0]);
        assert!((w.iter().sum::<f32>() - 140.0).abs() < 1e-4);
    }

    #[test]
    fn shrinks_fairly_and_never_exceeds_the_target() {
        // fair share 50: the 30 stays, the long ones split the remaining 70.
        let w = allot(&[30.0, 100.0, 100.0], 100.0);
        assert!((w[0] - 30.0).abs() < 1e-4);
        assert!((w[1] - 35.0).abs() < 1e-4 && (w[2] - 35.0).abs() < 1e-4);
        assert!((w.iter().sum::<f32>() - 100.0).abs() < 1e-3);
    }

    #[test]
    fn flex_one_gives_equal_shares_with_min_content() {
        // Equal shares when everything fits the share: 3 x 40 in 150.
        let w = super::allot_flex(&[30.0, 30.0, 30.0], 150.0);
        assert!(w.iter().all(|x| (x - 50.0).abs() < 1e-4));
        // A wide segment keeps its content width; the others split the rest equally.
        let w = super::allot_flex(&[20.0, 100.0, 20.0], 150.0);
        assert!((w[1] - 100.0).abs() < 1e-4);
        assert!((w[0] - 25.0).abs() < 1e-4 && (w[2] - 25.0).abs() < 1e-4);
        assert!((w.iter().sum::<f32>() - 150.0).abs() < 1e-3);
    }

    #[test]
    fn distribute_picks_the_right_mode() {
        // Room to stretch -> equal (flex: 1).
        let w = super::distribute(&[30.0, 30.0], 100.0, false);
        assert_eq!(w, vec![50.0, 50.0]);
        // Exactly natural -> unchanged (no stretching when not asked to).
        assert_eq!(super::distribute(&[30.0, 50.0], 80.0, false), vec![30.0, 50.0]);
        // Homogeneous -> strictly equal regardless of content.
        assert_eq!(super::distribute(&[30.0, 90.0], 100.0, true), vec![50.0, 50.0]);
    }

    #[test]
    fn compact_is_all_or_nothing() {
        let nat = [100.0, 100.0, 100.0];
        let comp = [Some(36.0), Some(36.0), Some(36.0)];
        // Everyone fits: stay full.
        assert!(!super::needs_compact(&[100.0, 100.0, 100.0], &nat, &comp));
        // One squeezed segment triggers compact for the whole control.
        assert!(super::needs_compact(&[100.0, 100.0, 60.0], &nat, &comp));
        // A squeezed segment with no compact form cannot trigger it.
        assert!(!super::needs_compact(&[100.0, 100.0, 60.0], &nat, &[Some(36.0), Some(36.0), None]));
    }

    /// Five segments with icons, crowded: one squeezed segment turns *all five* into icons, so
    /// the control is never half labels, half icons.
    #[test]
    fn five_crowded_segments_collapse_uniformly_to_icons() {
        let nat = [70.0, 70.0, 80.0, 70.0, 70.0];
        let comp = [Some(36.0); 5];
        // 300 for 360 of content: the fair shrink squeezes the long ones.
        let widths = super::distribute(&nat, 300.0, false);
        assert!(widths.iter().any(|w| *w + 0.5 < 80.0));
        assert!(super::use_compact(None, &widths, &nat, &comp));
        // Compact, all five fit with room to spare and share it equally.
        let compact = super::distribute(&[36.0; 5], 300.0, false);
        assert!(compact.iter().all(|w| (w - 60.0).abs() < 1e-3), "{compact:?}");
        // Room for the labels: nothing collapses.
        let roomy = super::distribute(&nat, 400.0, false);
        assert!(!super::use_compact(None, &roomy, &nat, &comp));
    }

    /// The breakpoint decides when it is given, whatever the space.
    #[test]
    fn the_breakpoint_overrides_the_space() {
        let nat = [70.0; 5];
        let comp = [Some(36.0); 5];
        let roomy = super::distribute(&nat, 600.0, false);
        assert!(super::use_compact(Some(true), &roomy, &nat, &comp), "mobile: icons even with room");
        let tight = super::distribute(&nat, 200.0, false);
        assert!(!super::use_compact(Some(false), &tight, &nat, &comp), "desktop: never icons");
    }

    #[test]
    fn mobile_mode_is_a_width_rule() {
        assert!(crate::window::is_mobile(400.0));
        assert!(!crate::window::is_mobile(900.0));
    }

    #[test]
    fn exact_fit_is_unchanged() {
        assert_eq!(allot(&[50.0, 50.0], 100.0), vec![50.0, 50.0]);
    }
}
