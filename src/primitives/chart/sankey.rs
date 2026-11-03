//! Sankey layout: nodes in layers, links as bands whose thickness is their value (after
//! d3-sankey, with justified alignment and iterative relaxation).

use super::shape::{Cmd, Pt};

/// A flow from node `source` to node `target`. Nodes are numbered `0..n`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Link {
    pub source: usize,
    pub target: usize,
    pub value: f32,
}

impl Link {
    pub fn new(source: usize, target: usize, value: f32) -> Self {
        Self {
            source,
            target,
            value,
        }
    }
}

/// A node's rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeBox {
    pub index: usize,
    /// The layer, counted from the sources.
    pub depth: usize,
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    /// The larger of what flows in and what flows out.
    pub value: f32,
}

impl NodeBox {
    pub fn contains(&self, p: Pt) -> bool {
        p.x >= self.x0 && p.x <= self.x1 && p.y >= self.y0 && p.y <= self.y1
    }
}

/// A link as a band: a curve from the source's right edge to the target's left edge, drawn
/// with a stroke `width` wide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub source: usize,
    pub target: usize,
    pub value: f32,
    pub x0: f32,
    pub x1: f32,
    /// The center of the band at the source.
    pub y0: f32,
    /// The center of the band at the target.
    pub y1: f32,
    pub width: f32,
}

impl Band {
    /// The center line as path commands.
    pub fn path(&self) -> [Cmd; 2] {
        let xi = (self.x0 + self.x1) / 2.0;
        [
            Cmd::Move(Pt::new(self.x0, self.y0)),
            Cmd::Cubic(
                Pt::new(xi, self.y0),
                Pt::new(xi, self.y1),
                Pt::new(self.x1, self.y1),
            ),
        ]
    }
    /// The center line's y at `x` (`None` outside the band's span).
    pub fn y_at(&self, x: f32) -> Option<f32> {
        if x < self.x0 || x > self.x1 {
            return None;
        }
        let xi = (self.x0 + self.x1) / 2.0;
        let bx = |t: f32| {
            let u = 1.0 - t;
            u * u * u * self.x0 + 3.0 * u * u * t * xi + 3.0 * u * t * t * xi + t * t * t * self.x1
        };
        // x(t) rises with t, so bisect.
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if bx(mid) < x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t = (lo + hi) / 2.0;
        let u = 1.0 - t;
        let s = u * u * u + 3.0 * u * u * t;
        Some(s * self.y0 + (1.0 - s) * self.y1)
    }
    pub fn contains(&self, p: Pt) -> bool {
        self.y_at(p.x)
            .is_some_and(|y| (p.y - y).abs() <= self.width / 2.0)
    }
}

/// The laid out diagram.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sankey {
    pub nodes: Vec<NodeBox>,
    pub links: Vec<Band>,
}

/// Lays out `n` nodes and their `links` in a `width` × `height` area. `node_width` and
/// `node_pad` (the gap between nodes of a layer) are in the same units; `iterations` of
/// relaxation move nodes towards what they connect to (6 is enough for most diagrams).
/// Links with a bad index, no value or the same ends are dropped, and cycles do not loop: a
/// node never lands deeper than `n - 1`.
pub fn sankey(
    n: usize,
    links: &[Link],
    width: f32,
    height: f32,
    node_width: f32,
    node_pad: f32,
    iterations: usize,
) -> Sankey {
    let links: Vec<Link> = links
        .iter()
        .copied()
        .filter(|l| {
            l.source < n
                && l.target < n
                && l.source != l.target
                && l.value > 0.0
                && l.value.is_finite()
        })
        .collect();
    let (mut inn, mut out) = (vec![0.0f32; n], vec![0.0f32; n]);
    for l in &links {
        out[l.source] += l.value;
        inn[l.target] += l.value;
    }
    let value: Vec<f32> = (0..n).map(|i| inn[i].max(out[i])).collect();

    // Layers: as deep as the longest path to the node; sinks line up at the end.
    let mut depth = vec![0usize; n];
    for _ in 0..n {
        let mut changed = false;
        for l in &links {
            let d = (depth[l.source] + 1).min(n.saturating_sub(1));
            if depth[l.target] < d {
                depth[l.target] = d;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let last = depth.iter().copied().max().unwrap_or(0);
    for i in 0..n {
        if out[i] == 0.0 && inn[i] > 0.0 {
            depth[i] = last;
        }
    }
    let mut layers: Vec<Vec<usize>> = vec![Vec::new(); last + 1];
    for i in 0..n {
        layers[depth[i]].push(i);
    }

    let span = (width - node_width).max(0.0) / last.max(1) as f32;
    let ky = layers
        .iter()
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let sum: f32 = l.iter().map(|i| value[*i]).sum();
            (sum > 0.0).then(|| ((height - node_pad * (l.len() as f32 - 1.0)) / sum).max(0.0))
        })
        .fold(f32::INFINITY, f32::min);
    let ky = if ky.is_finite() { ky } else { 0.0 };

    let mut y0 = vec![0.0f32; n];
    for layer in &layers {
        let mut y = 0.0;
        for i in layer {
            y0[*i] = y;
            y += value[*i] * ky + node_pad;
        }
    }

    let center = |y0: &[f32], i: usize| y0[i] + value[i] * ky / 2.0;
    let pull = |y0: &mut Vec<f32>, layer: &[usize], forward: bool, alpha: f32| {
        for &i in layer {
            let (mut sum, mut weight) = (0.0, 0.0);
            for l in &links {
                let other = match (forward, l.target == i, l.source == i) {
                    (true, true, _) => l.source,
                    (false, _, true) => l.target,
                    _ => continue,
                };
                sum += l.value * center(y0, other);
                weight += l.value;
            }
            if weight > 0.0 {
                y0[i] += (sum / weight - center(y0, i)) * alpha;
            }
        }
        // Resolve overlaps: push down from the top, then back up from the bottom.
        let mut order = layer.to_vec();
        order.sort_by(|a, b| y0[*a].total_cmp(&y0[*b]));
        let mut y = 0.0;
        for &i in &order {
            y0[i] = y0[i].max(y);
            y = y0[i] + value[i] * ky + node_pad;
        }
        let mut bottom = height;
        for &i in order.iter().rev() {
            let h = value[i] * ky;
            if y0[i] + h > bottom {
                y0[i] = (bottom - h).max(0.0);
            }
            bottom = y0[i] - node_pad;
        }
    };
    for it in 0..iterations {
        let alpha = 0.99f32.powi(it as i32);
        for layer in layers.iter().skip(1) {
            pull(&mut y0, layer, true, alpha);
        }
        for layer in layers.iter().rev().skip(1) {
            pull(&mut y0, layer, false, alpha);
        }
    }

    let nodes: Vec<NodeBox> = (0..n)
        .map(|i| NodeBox {
            index: i,
            depth: depth[i],
            x0: depth[i] as f32 * span,
            x1: depth[i] as f32 * span + node_width,
            y0: y0[i],
            y1: y0[i] + value[i] * ky,
            value: value[i],
        })
        .collect();

    // Each node's links take their share of its edge, ordered by where they go.
    let mut bands: Vec<Band> = links
        .iter()
        .map(|l| Band {
            source: l.source,
            target: l.target,
            value: l.value,
            x0: nodes[l.source].x1,
            x1: nodes[l.target].x0,
            y0: 0.0,
            y1: 0.0,
            width: l.value * ky,
        })
        .collect();
    for (i, node) in nodes.iter().enumerate() {
        let mut outgoing: Vec<usize> = (0..bands.len()).filter(|b| bands[*b].source == i).collect();
        outgoing.sort_by(|a, b| {
            nodes[bands[*a].target]
                .y0
                .total_cmp(&nodes[bands[*b].target].y0)
        });
        let mut y = node.y0;
        for b in outgoing {
            bands[b].y0 = y + bands[b].width / 2.0;
            y += bands[b].width;
        }
        let mut incoming: Vec<usize> = (0..bands.len()).filter(|b| bands[*b].target == i).collect();
        incoming.sort_by(|a, b| {
            nodes[bands[*a].source]
                .y0
                .total_cmp(&nodes[bands[*b].source].y0)
        });
        let mut y = node.y0;
        for b in incoming {
            bands[b].y1 = y + bands[b].width / 2.0;
            y += bands[b].width;
        }
    }
    Sankey {
        nodes,
        links: bands,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow() -> Sankey {
        // 0 -> 1 -> 3 and 0 -> 2 -> 3
        sankey(
            4,
            &[
                Link::new(0, 1, 6.0),
                Link::new(0, 2, 4.0),
                Link::new(1, 3, 6.0),
                Link::new(2, 3, 4.0),
            ],
            300.0,
            100.0,
            10.0,
            10.0,
            6,
        )
    }

    #[test]
    fn nodes_sit_in_layers() {
        let s = flow();
        let d: Vec<usize> = s.nodes.iter().map(|n| n.depth).collect();
        assert_eq!(d, vec![0, 1, 1, 2]);
        assert_eq!(s.nodes[0].x0, 0.0);
        assert_eq!(s.nodes[3].x1, 300.0);
    }

    #[test]
    fn thickness_is_proportional_and_fits() {
        let s = flow();
        assert!((s.nodes[0].y1 - s.nodes[0].y0 - s.nodes[3].y1 + s.nodes[3].y0).abs() < 1e-3);
        assert!((s.links[0].width / s.links[1].width - 1.5).abs() < 1e-4);
        for n in &s.nodes {
            assert!(n.y0 >= -1e-3 && n.y1 <= 100.0 + 1e-3);
        }
        // Nodes of one layer do not overlap.
        let (a, b) = (s.nodes[1], s.nodes[2]);
        assert!(a.y1 <= b.y0 + 1e-3 || b.y1 <= a.y0 + 1e-3);
    }

    #[test]
    fn links_leave_inside_their_node() {
        let s = flow();
        for l in &s.links {
            let (a, b) = (s.nodes[l.source], s.nodes[l.target]);
            assert!(l.y0 - l.width / 2.0 >= a.y0 - 1e-3 && l.y0 + l.width / 2.0 <= a.y1 + 1e-3);
            assert!(l.y1 - l.width / 2.0 >= b.y0 - 1e-3 && l.y1 + l.width / 2.0 <= b.y1 + 1e-3);
            assert_eq!((l.x0, l.x1), (a.x1, b.x0));
        }
    }

    #[test]
    fn sinks_line_up_at_the_end() {
        // 0 -> 1 -> 2 and 0 -> 3: node 3 is a sink, so it joins the last layer.
        let s = sankey(
            4,
            &[
                Link::new(0, 1, 1.0),
                Link::new(1, 2, 1.0),
                Link::new(0, 3, 1.0),
            ],
            100.0,
            50.0,
            5.0,
            5.0,
            4,
        );
        assert_eq!(s.nodes[3].depth, s.nodes[2].depth);
    }

    #[test]
    fn bad_input_is_dropped_and_cycles_end() {
        let s = sankey(
            3,
            &[
                Link::new(0, 1, 1.0),
                Link::new(1, 0, 1.0),
                Link::new(0, 9, 1.0),
                Link::new(2, 2, 1.0),
                Link::new(1, 2, 0.0),
            ],
            100.0,
            50.0,
            5.0,
            5.0,
            3,
        );
        assert_eq!(s.links.len(), 2);
        assert!(sankey(0, &[], 10.0, 10.0, 1.0, 1.0, 1).nodes.is_empty());
    }

    #[test]
    fn a_band_is_hit_along_its_curve() {
        let s = flow();
        let b = s.links[0];
        let mid = Pt::new((b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0);
        assert!(b.contains(mid));
        assert!(!b.contains(Pt::new(mid.x, mid.y + b.width)));
        assert_eq!(b.y_at(b.x0 - 1.0), None);
    }
}
