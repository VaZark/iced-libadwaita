//! A tree of values, partitioned into rings (d3-hierarchy's `partition`, for a sunburst).

/// A node: a leaf carries its own value, a branch is the sum of its children.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub value: f32,
    pub children: Vec<Node>,
}

impl Node {
    pub fn leaf(name: impl Into<String>, value: f32) -> Self {
        Self {
            name: name.into(),
            value,
            children: Vec::new(),
        }
    }
    pub fn branch(name: impl Into<String>, children: impl Into<Vec<Node>>) -> Self {
        Self {
            name: name.into(),
            value: 0.0,
            children: children.into(),
        }
    }
    /// The value of a leaf, or the sum of the children. Negative and non-finite values are zero.
    pub fn total(&self) -> f32 {
        if self.children.is_empty() {
            if self.value.is_finite() {
                self.value.max(0.0)
            } else {
                0.0
            }
        } else {
            self.children.iter().map(Node::total).sum()
        }
    }
    /// How many levels there are below this node (0 for a leaf).
    pub fn height(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.height() + 1)
            .max()
            .unwrap_or(0)
    }
}

/// One node as a ring segment: `x0..x1` is its share of the circle (`0..=1`) and `depth` its ring
/// (the root is 0).
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub depth: usize,
    pub x0: f32,
    pub x1: f32,
    pub value: f32,
    pub name: String,
    /// The index among the root's children of the branch this node is in (itself, at depth 1);
    /// 0 for the root. Color by this, so a branch keeps one hue.
    pub branch: usize,
    /// The names from the root's child down to this node.
    pub path: Vec<String>,
}

/// Every node of `root`, parents before children. Children share their parent's span in
/// proportion to their value; nodes without value take no space.
pub fn partition(root: &Node) -> Vec<Cell> {
    let mut out = Vec::new();
    walk(root, 0, 0.0, 1.0, 0, &mut Vec::new(), &mut out);
    out
}

fn walk(
    n: &Node,
    depth: usize,
    x0: f32,
    x1: f32,
    branch: usize,
    path: &mut Vec<String>,
    out: &mut Vec<Cell>,
) {
    let total = n.total();
    out.push(Cell {
        depth,
        x0,
        x1,
        value: total,
        name: n.name.clone(),
        branch,
        path: path.clone(),
    });
    if total <= 0.0 {
        return;
    }
    let mut x = x0;
    for (i, c) in n.children.iter().enumerate() {
        let w = (x1 - x0) * c.total() / total;
        path.push(c.name.clone());
        walk(
            c,
            depth + 1,
            x,
            x + w,
            if depth == 0 { i } else { branch },
            path,
            out,
        );
        path.pop();
        x += w;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Node {
        Node::branch(
            "root",
            [
                Node::branch("a", [Node::leaf("a1", 1.0), Node::leaf("a2", 3.0)]),
                Node::leaf("b", 4.0),
            ],
        )
    }

    #[test]
    fn a_branch_is_the_sum_of_its_children() {
        assert_eq!(tree().total(), 8.0);
        assert_eq!(tree().height(), 2);
        assert_eq!(Node::leaf("x", -3.0).total(), 0.0);
    }

    #[test]
    fn children_share_the_parent_span() {
        let cells = partition(&tree());
        let by = |n: &str| cells.iter().find(|c| c.name == n).unwrap().clone();
        assert_eq!((by("root").x0, by("root").x1), (0.0, 1.0));
        assert_eq!((by("a").x0, by("a").x1), (0.0, 0.5));
        assert_eq!((by("b").x0, by("b").x1), (0.5, 1.0));
        assert_eq!((by("a1").x0, by("a1").x1), (0.0, 0.125));
        assert_eq!(by("a2").depth, 2);
    }

    #[test]
    fn a_branch_keeps_its_hue() {
        let cells = partition(&tree());
        let by = |n: &str| cells.iter().find(|c| c.name == n).unwrap().clone();
        assert_eq!((by("a").branch, by("a2").branch, by("b").branch), (0, 0, 1));
        assert_eq!(by("a2").path, vec!["a", "a2"]);
    }

    #[test]
    fn an_empty_tree_is_safe() {
        let cells = partition(&Node::branch("root", []));
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].value, 0.0);
    }
}
