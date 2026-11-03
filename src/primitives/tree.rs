//! Headless logic of a tree: the data, which branches are open, what is selected and checked,
//! search and filter, and keyboard navigation. After Mantine's
//! [Tree](https://mantine.dev/core/tree/) and its `useTree`, with the keys of GTK's
//! `GtkTreeExpander`.
//!
//! Pure and controlled: the app owns the [`TreeNode`] data and one [`TreeState`], and applies the
//! one message the widget speaks ([`Event`]) with [`TreeState::apply`].
//!
//! * **Search is external**, as in Mantine: the tree has no search box. [`filter`] keeps the
//!   nodes that match and their ancestors (open the result with [`TreeState::expand_all`]);
//!   [`matching_ancestors`] lists the nodes to open when *highlighting* instead of filtering.
//! * **Checking** follows Mantine's default (not `check_strictly`): checking a branch checks all
//!   below it, a branch whose children are all checked is checked, and one with only some is
//!   [indeterminate](TreeState::is_indeterminate).
//! * **Selection** is single, or multiple with an anchor for ranges
//!   ([`select_range`](TreeState::select_range)).
//! * **Async loading**: a [`lazy`](TreeNode::lazy) node has children that are not loaded yet.
//!   Opening it returns [`Effect::Load`]; the app fetches them and calls
//!   [`finish_load`](TreeState::finish_load), with the error shown (and Retry) if it failed.
//! * **Drag and drop**: [`Event::Drop`] moves a node before, inside or after another
//!   ([`move_node`]); [`can_move`] refuses a node into itself or its own branch, and the widget
//!   takes the app's `allow_drop` on top (Mantine's `allowDrop`).
//! * **Virtualization**: [`window`] (from [`virtualize`](super::virtualize)) says which rows to build
//!   for a scroll offset.
//! * **Keys** ([`navigate`]) are those of GTK's tree expander and the ARIA tree: up / down /
//!   Home / End move, Right opens a branch or enters it, Left closes it or goes to its parent,
//!   `+` `*` open, `-` `/` close, Ctrl+Space toggles.

use super::loading::Loads;
use std::collections::HashSet;

/// One node: a unique `value`, a `label`, and its `children`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub value: String,
    pub label: String,
    pub children: Vec<TreeNode>,
    /// It has children that are not loaded yet (Mantine's `hasChildren` without `children`).
    pub lazy: bool,
}

impl TreeNode {
    /// A leaf.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: Vec::new(),
            lazy: false,
        }
    }
    /// A branch whose children are loaded when it is first opened.
    pub fn lazy(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            lazy: true,
            ..Self::new(value, label)
        }
    }
    /// A branch with `children`.
    pub fn branch(
        value: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeNode>,
    ) -> Self {
        Self {
            children,
            ..Self::new(value, label)
        }
    }
    /// Whether it can be opened: it has children, or will once loaded.
    pub fn has_children(&self) -> bool {
        self.lazy || !self.children.is_empty()
    }
}

/// Whether `node` matches `query`: its label contains it, ignoring case. An empty query matches
/// everything (Mantine's `defaultTreeNodeFilter`).
pub fn matches(query: &str, node: &TreeNode) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty() || node.label.to_lowercase().contains(&q)
}

/// Whether `query` is a subsequence of the node's label, ignoring case: "bttn" finds "Button".
/// A blank query matches everything. A simple stand-in for Mantine's fuse.js filter.
pub fn fuzzy(query: &str, node: &TreeNode) -> bool {
    fuzzy_ranges(&node.label, query).len() == query.trim().chars().count()
        || query.trim().is_empty()
}

/// The characters of `label` that a fuzzy `query` uses, as single-character byte ranges, for
/// highlighting. Empty when the query is not a subsequence of the label.
pub fn fuzzy_ranges(label: &str, query: &str) -> Vec<std::ops::Range<usize>> {
    let q: Vec<char> = query.trim().to_lowercase().chars().collect();
    let mut out = Vec::new();
    let mut next = 0;
    for (i, c) in label.char_indices() {
        if next < q.len() && c.to_lowercase().next() == Some(q[next]) {
            out.push(i..i + c.len_utf8());
            next += 1;
        }
    }
    if next == q.len() { out } else { Vec::new() }
}

/// The tree with only the nodes that match `query` and their ancestors. A matching node keeps
/// the children that match; a node that only has a matching descendant stays as the way to it.
pub fn filter(data: &[TreeNode], query: &str) -> Vec<TreeNode> {
    filter_with(data, query, &matches)
}

/// [`filter`] with another matcher (a fuzzy one, say).
pub fn filter_with(
    data: &[TreeNode],
    query: &str,
    matcher: &dyn Fn(&str, &TreeNode) -> bool,
) -> Vec<TreeNode> {
    if query.trim().is_empty() {
        return data.to_vec();
    }
    data.iter()
        .filter_map(|n| {
            let children = filter_with(&n.children, query, matcher);
            (matcher(query, n) || !children.is_empty()).then(|| TreeNode {
                children,
                ..n.clone()
            })
        })
        .collect()
}

/// The values of the nodes that match `query` or have a matching descendant: the branches to
/// open to show every match, when highlighting rather than filtering.
pub fn matching_ancestors(data: &[TreeNode], query: &str) -> Vec<String> {
    matching_ancestors_with(data, query, &matches)
}

/// [`matching_ancestors`] with another matcher.
pub fn matching_ancestors_with(
    data: &[TreeNode],
    query: &str,
    matcher: &dyn Fn(&str, &TreeNode) -> bool,
) -> Vec<String> {
    let mut out = Vec::new();
    for n in data {
        let below = matching_ancestors_with(&n.children, query, matcher);
        if matcher(query, n) || !below.is_empty() {
            out.push(n.value.clone());
            out.extend(below);
        }
    }
    out
}

/// Where `query` occurs in `label`, as byte ranges (case-insensitive), for highlighting. Empty
/// for an empty query.
pub fn match_ranges(label: &str, query: &str) -> Vec<std::ops::Range<usize>> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    // Lowercasing can change byte lengths, so search the lowercase text char by char and map back.
    let lower: Vec<(usize, char)> = label
        .char_indices()
        .flat_map(|(i, c)| c.to_lowercase().map(move |l| (i, l)))
        .collect();
    let needle: Vec<char> = q.chars().collect();
    let mut out: Vec<std::ops::Range<usize>> = Vec::new();
    let mut i = 0;
    while i + needle.len() <= lower.len() {
        if lower[i..i + needle.len()]
            .iter()
            .map(|(_, c)| c)
            .eq(needle.iter())
        {
            let start = lower[i].0;
            let last = lower[i + needle.len() - 1].0;
            let end = last + label[last..].chars().next().map_or(0, char::len_utf8);
            out.push(start..end);
            i += needle.len();
        } else {
            i += 1;
        }
    }
    out
}

/// A visible row: a node with its depth and whether it is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub value: String,
    pub label: String,
    pub level: usize,
    pub has_children: bool,
    pub expanded: bool,
    /// Its children are being loaded.
    pub loading: bool,
    /// Loading its children failed, with the reason.
    pub error: Option<String>,
}

/// The rows to show: every node whose ancestors are all open, in order.
pub fn visible(data: &[TreeNode], expanded: &HashSet<String>) -> Vec<Row> {
    fn walk(nodes: &[TreeNode], level: usize, expanded: &HashSet<String>, out: &mut Vec<Row>) {
        for n in nodes {
            let open = n.has_children() && expanded.contains(&n.value);
            out.push(Row {
                value: n.value.clone(),
                label: n.label.clone(),
                level,
                has_children: n.has_children(),
                expanded: open,
                loading: false,
                error: None,
            });
            if open {
                walk(&n.children, level + 1, expanded, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(data, 0, expanded, &mut out);
    out
}

fn find_mut<'a>(data: &'a mut [TreeNode], value: &str) -> Option<&'a mut TreeNode> {
    for n in data {
        if n.value == value {
            return Some(n);
        }
        if let Some(f) = find_mut(&mut n.children, value) {
            return Some(f);
        }
    }
    None
}

/// The node with this value, anywhere in the tree.
pub fn find<'a>(data: &'a [TreeNode], value: &str) -> Option<&'a TreeNode> {
    data.iter().find_map(|n| {
        (n.value == value)
            .then_some(n)
            .or_else(|| find(&n.children, value))
    })
}

/// The values of `node` and everything below it.
fn subtree(node: &TreeNode, out: &mut Vec<String>) {
    out.push(node.value.clone());
    for c in &node.children {
        subtree(c, out);
    }
}

/// The values on the way to `value` (its ancestors, outermost first), if it exists.
fn path(data: &[TreeNode], value: &str) -> Option<Vec<String>> {
    for n in data {
        if n.value == value {
            return Some(Vec::new());
        }
        if let Some(mut p) = path(&n.children, value) {
            p.insert(0, n.value.clone());
            return Some(p);
        }
    }
    None
}

/// Which rows to build for a scrolled list: the shared [`virtualize::window`](super::virtualize::window).
pub use super::virtualize::window;

/// Where a dragged node lands relative to the one it is dropped on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// As the previous sibling.
    Before,
    /// As the last child.
    Inside,
    /// As the next sibling.
    After,
}

/// The drop position for a pointer at `frac` (0 = top edge, 1 = bottom edge) of the target row:
/// the top quarter is `Before`, the bottom quarter `After`, and the middle `Inside`, unless
/// `can_inside` is false, when the middle goes to the nearer edge.
pub fn drop_position(frac: f32, can_inside: bool) -> Position {
    if frac < 0.25 || (!can_inside && frac < 0.5) {
        Position::Before
    } else if frac > 0.75 || !can_inside {
        Position::After
    } else {
        Position::Inside
    }
}

/// Whether `from` may be moved onto `to`: both exist, they differ, and `to` is not inside the
/// branch of `from` (a node cannot go into itself).
pub fn can_move(data: &[TreeNode], from: &str, to: &str) -> bool {
    from != to
        && find(data, to).is_some()
        && find(data, from).is_some_and(|n| {
            let mut below = Vec::new();
            subtree(n, &mut below);
            !below.iter().any(|v| v == to)
        })
}

fn remove(data: &mut Vec<TreeNode>, value: &str) -> Option<TreeNode> {
    if let Some(i) = data.iter().position(|n| n.value == value) {
        return Some(data.remove(i));
    }
    data.iter_mut().find_map(|n| remove(&mut n.children, value))
}

fn insert(
    data: &mut Vec<TreeNode>,
    to: &str,
    pos: Position,
    node: TreeNode,
) -> Result<(), TreeNode> {
    if let Some(i) = data.iter().position(|n| n.value == to) {
        match pos {
            Position::Before => data.insert(i, node),
            Position::After => data.insert(i + 1, node),
            Position::Inside => {
                data[i].children.push(node);
                data[i].lazy = false;
            }
        }
        return Ok(());
    }
    let mut node = node;
    for n in data.iter_mut() {
        match insert(&mut n.children, to, pos, node) {
            Ok(()) => return Ok(()),
            Err(back) => node = back,
        }
    }
    Err(node)
}

/// Moves `from` to `pos` of `to` (Mantine's `moveTreeNode`). Returns whether it moved.
pub fn move_node(data: &mut Vec<TreeNode>, from: &str, to: &str, pos: Position) -> bool {
    if !can_move(data, from, to) {
        return false;
    }
    let Some(node) = remove(data, from) else {
        return false;
    };
    insert(data, to, pos, node).is_ok()
}

/// What the widget reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// The arrow of a branch was pressed.
    Toggle(String),
    /// A row's label was pressed: it is selected (and opened, with `expand_on_click`).
    Click(String),
    /// A row's checkbox was pressed: check or uncheck it.
    Check(String, bool),
    /// A node was dropped on another.
    Drop {
        from: String,
        to: String,
        position: Position,
    },
    /// Retry loading a branch whose load failed.
    Retry(String),
}

/// What the app must do after an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Fetch the children of this lazy branch, then call [`TreeState::finish_load`].
    Load(String),
}

/// Which branches are open, and what is selected and checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeState {
    expanded: HashSet<String>,
    selected: Vec<String>,
    checked: HashSet<String>,
    anchor: Option<String>,
    /// Which lazy branches are loading or failed (the shared [`Loads`] tracker).
    loads: Loads<String>,
    /// More than one row can be selected.
    pub multiple: bool,
    /// Checking a node affects only that node, not its branch (Mantine's `checkStrictly`).
    pub check_strictly: bool,
    /// Pressing a branch's label also opens or closes it (Mantine's `expandOnClick`, on by default).
    pub expand_on_click: bool,
}

impl Default for TreeState {
    fn default() -> Self {
        Self {
            expanded: HashSet::new(),
            selected: Vec::new(),
            checked: HashSet::new(),
            anchor: None,
            loads: Loads::new(),
            multiple: false,
            check_strictly: false,
            expand_on_click: true,
        }
    }
}

impl TreeState {
    pub fn new() -> Self {
        Self::default()
    }

    // -- expansion

    pub fn is_expanded(&self, value: &str) -> bool {
        self.expanded.contains(value)
    }
    pub fn expanded(&self) -> &HashSet<String> {
        &self.expanded
    }
    pub fn expand(&mut self, value: &str) {
        self.expanded.insert(value.to_string());
    }
    pub fn collapse(&mut self, value: &str) {
        self.expanded.remove(value);
    }
    pub fn toggle_expanded(&mut self, value: &str) {
        if !self.expanded.remove(value) {
            self.expanded.insert(value.to_string());
        }
    }
    /// Opens every branch (`getTreeExpandedState(data, '*')`).
    pub fn expand_all(&mut self, data: &[TreeNode]) {
        fn walk(nodes: &[TreeNode], out: &mut HashSet<String>) {
            for n in nodes.iter().filter(|n| n.has_children()) {
                out.insert(n.value.clone());
                walk(&n.children, out);
            }
        }
        self.expanded.clear();
        walk(data, &mut self.expanded);
    }
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
    }
    /// Opens exactly these branches (the result of [`matching_ancestors`], say).
    pub fn set_expanded(&mut self, values: impl IntoIterator<Item = String>) {
        self.expanded = values.into_iter().collect();
    }
    /// Opens the branches on the way to `value`, so its row is visible.
    pub fn reveal(&mut self, data: &[TreeNode], value: &str) {
        if let Some(p) = path(data, value) {
            self.expanded.extend(p);
        }
    }

    // -- selection

    pub fn selected(&self) -> &[String] {
        &self.selected
    }
    pub fn is_selected(&self, value: &str) -> bool {
        self.selected.iter().any(|v| v == value)
    }
    /// Selects `value`: alone, or added to the selection when `multiple`. It becomes the anchor.
    pub fn select(&mut self, value: &str) {
        if self.multiple {
            if !self.is_selected(value) {
                self.selected.push(value.to_string());
            }
        } else {
            self.selected = vec![value.to_string()];
        }
        self.anchor = Some(value.to_string());
    }
    /// Selects `value`, or deselects it when it already is.
    pub fn toggle_selected(&mut self, value: &str) {
        if self.is_selected(value) {
            self.selected.retain(|v| v != value);
        } else {
            self.select(value);
        }
    }
    pub fn clear_selected(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }
    /// Selects the visible rows from the anchor to `value` (Shift+click). Without an anchor it
    /// selects `value` alone.
    pub fn select_range(&mut self, data: &[TreeNode], value: &str) {
        let rows = visible(data, &self.expanded);
        let to = rows.iter().position(|r| r.value == value);
        let from = self
            .anchor
            .as_deref()
            .and_then(|a| rows.iter().position(|r| r.value == a));
        match (from, to) {
            (Some(a), Some(b)) if self.multiple => {
                let (lo, hi) = (a.min(b), a.max(b));
                self.selected = rows[lo..=hi].iter().map(|r| r.value.clone()).collect();
            }
            _ => self.select(value),
        }
    }

    // -- checking

    pub fn is_checked(&self, value: &str) -> bool {
        self.checked.contains(value)
    }
    /// A branch with some, but not all, of what is below it checked.
    pub fn is_indeterminate(&self, data: &[TreeNode], value: &str) -> bool {
        if self.check_strictly || self.is_checked(value) {
            return false;
        }
        find(data, value).is_some_and(|n| {
            let mut below = Vec::new();
            subtree(n, &mut below);
            below[1..].iter().any(|v| self.checked.contains(v))
        })
    }
    /// Checks `value` (and, unless `check_strictly`, everything below it, and the branches above
    /// whose children are now all checked).
    pub fn check(&mut self, data: &[TreeNode], value: &str) {
        let Some(node) = find(data, value) else {
            return;
        };
        if self.check_strictly {
            self.checked.insert(value.to_string());
            return;
        }
        let mut all = Vec::new();
        subtree(node, &mut all);
        self.checked.extend(all);
        for a in path(data, value).unwrap_or_default().iter().rev() {
            let Some(parent) = find(data, a) else { break };
            if parent
                .children
                .iter()
                .all(|c| self.checked.contains(&c.value))
            {
                self.checked.insert(a.clone());
            } else {
                break;
            }
        }
    }
    /// Unchecks `value` (and, unless `check_strictly`, everything below it and the branches
    /// above it, which are no longer fully checked).
    pub fn uncheck(&mut self, data: &[TreeNode], value: &str) {
        let Some(node) = find(data, value) else {
            return;
        };
        if self.check_strictly {
            self.checked.remove(value);
            return;
        }
        let mut all = Vec::new();
        subtree(node, &mut all);
        for v in all {
            self.checked.remove(&v);
        }
        for a in path(data, value).unwrap_or_default() {
            self.checked.remove(&a);
        }
    }
    pub fn check_all(&mut self, data: &[TreeNode]) {
        for n in data {
            self.check(data, &n.value);
        }
    }
    pub fn uncheck_all(&mut self) {
        self.checked.clear();
    }
    /// The checked nodes, in tree order.
    pub fn checked_nodes<'a>(&self, data: &'a [TreeNode]) -> Vec<&'a TreeNode> {
        fn walk<'a>(nodes: &'a [TreeNode], on: &HashSet<String>, out: &mut Vec<&'a TreeNode>) {
            for n in nodes {
                if on.contains(&n.value) {
                    out.push(n);
                }
                walk(&n.children, on, out);
            }
        }
        let mut out = Vec::new();
        walk(data, &self.checked, &mut out);
        out
    }

    // -- async loading

    pub fn is_loading(&self, value: &str) -> bool {
        self.loads.is_loading(&value.to_string())
    }
    pub fn load_error(&self, value: &str) -> Option<&str> {
        self.loads.error(&value.to_string())
    }
    /// Starts loading `value` when it is a lazy branch with nothing loaded and not already
    /// loading. Returns whether the app should fetch.
    fn begin_load(&mut self, data: &[TreeNode], value: &str) -> bool {
        let lazy = find(data, value).is_some_and(|n| n.lazy && n.children.is_empty());
        lazy && self.loads.begin(value.to_string())
    }
    /// Puts the loaded `children` into the lazy branch `value` (Mantine's `mergeAsyncChildren`),
    /// or records why it failed. A failed branch stays open and shows the error with Retry.
    pub fn finish_load(
        &mut self,
        data: &mut [TreeNode],
        value: &str,
        result: Result<Vec<TreeNode>, String>,
    ) {
        match result {
            Ok(children) => {
                self.loads.finish(&value.to_string(), Ok(()));
                if let Some(n) = find_mut(data, value) {
                    n.children = children;
                    n.lazy = false;
                }
            }
            Err(e) => self.loads.finish(&value.to_string(), Err(e)),
        }
    }

    /// The rows to show, with what is loading and what failed.
    pub fn rows(&self, data: &[TreeNode]) -> Vec<Row> {
        let mut rows = visible(data, &self.expanded);
        for r in &mut rows {
            r.loading = self.loads.is_loading(&r.value);
            r.error = self.loads.error(&r.value).map(str::to_string);
        }
        rows
    }

    /// Opens or closes a branch, asking for its children first when they are not loaded.
    fn toggle_loading(&mut self, data: &[TreeNode], value: &str) -> Option<Effect> {
        let opening = !self.is_expanded(value);
        self.toggle_expanded(value);
        (opening && self.begin_load(data, value)).then(|| Effect::Load(value.to_string()))
    }

    /// Applies what the widget reported. Returns what the app has to do next, if anything.
    pub fn apply(&mut self, data: &mut Vec<TreeNode>, event: Event) -> Option<Effect> {
        match event {
            Event::Toggle(v) => return self.toggle_loading(data, &v),
            Event::Click(v) => {
                if self.multiple && self.is_selected(&v) {
                    self.toggle_selected(&v);
                } else {
                    self.select(&v);
                }
                if self.expand_on_click && find(data, &v).is_some_and(TreeNode::has_children) {
                    return self.toggle_loading(data, &v);
                }
            }
            Event::Check(v, true) => self.check(data, &v),
            Event::Check(v, false) => self.uncheck(data, &v),
            Event::Drop { from, to, position } => {
                if move_node(data, &from, &to, position) && position == Position::Inside {
                    self.expand(&to);
                }
            }
            Event::Retry(v) => {
                self.expand(&v);
                return self.begin_load(data, &v).then_some(Effect::Load(v));
            }
        }
        None
    }
}

/// A key the tree answers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavKey {
    Up,
    Down,
    Home,
    End,
    /// Opens a closed branch, else enters it (the first child).
    Right,
    /// Closes an open branch, else goes to its parent.
    Left,
    /// `+` and `*`.
    Expand,
    /// `-` and `/`.
    Collapse,
    /// Ctrl+Space.
    Toggle,
}

/// What a key does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nav {
    Focus(String),
    Expand(String),
    Collapse(String),
}

/// What `key` does with `focused` focused among the visible `rows`; `None` when nothing happens
/// (an end, a leaf, nothing focused).
pub fn navigate(rows: &[Row], focused: Option<&str>, key: NavKey) -> Option<Nav> {
    let at = focused.and_then(|f| rows.iter().position(|r| r.value == f));
    let focus = |i: usize| rows.get(i).map(|r| Nav::Focus(r.value.clone()));
    match key {
        NavKey::Home => focus(0),
        NavKey::End => rows.len().checked_sub(1).and_then(focus),
        NavKey::Down => focus(at.map_or(0, |i| i + 1)),
        NavKey::Up => match at {
            Some(i) => i.checked_sub(1).and_then(focus),
            None => rows.len().checked_sub(1).and_then(focus),
        },
        _ => {
            let i = at?;
            let row = &rows[i];
            match key {
                NavKey::Right | NavKey::Expand if row.has_children && !row.expanded => {
                    Some(Nav::Expand(row.value.clone()))
                }
                NavKey::Right if row.has_children => focus(i + 1),
                NavKey::Left | NavKey::Collapse if row.expanded => {
                    Some(Nav::Collapse(row.value.clone()))
                }
                NavKey::Left => rows[..i]
                    .iter()
                    .rposition(|r| r.level + 1 == row.level)
                    .and_then(focus),
                NavKey::Toggle if row.has_children => Some(if row.expanded {
                    Nav::Collapse(row.value.clone())
                } else {
                    Nav::Expand(row.value.clone())
                }),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(v: &str) -> TreeNode {
        TreeNode::new(v, v)
    }
    fn b(v: &str, c: Vec<TreeNode>) -> TreeNode {
        TreeNode::branch(v, v, c)
    }
    /// src { components { Button, Card }, theme }, docs { README }
    fn data() -> Vec<TreeNode> {
        vec![
            b(
                "src",
                vec![b("components", vec![n("Button"), n("Card")]), n("theme")],
            ),
            b("docs", vec![n("README")]),
        ]
    }
    fn values(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.value.as_str()).collect()
    }

    #[test]
    fn only_rows_under_open_branches_are_visible() {
        let mut s = TreeState::new();
        assert_eq!(values(&visible(&data(), s.expanded())), ["src", "docs"]);
        s.expand("src");
        assert_eq!(
            values(&visible(&data(), s.expanded())),
            ["src", "components", "theme", "docs"]
        );
        s.expand("components");
        let rows = visible(&data(), s.expanded());
        assert_eq!(rows[2].level, 2);
        assert!(rows[1].expanded && rows[1].has_children);
    }

    #[test]
    fn a_leaf_is_never_open() {
        let mut s = TreeState::new();
        s.expand("theme");
        let rows = visible(&data(), s.expanded());
        assert!(rows.iter().all(|r| !r.expanded));
    }

    #[test]
    fn expand_all_and_collapse_all() {
        let mut s = TreeState::new();
        s.expand_all(&data());
        assert_eq!(visible(&data(), s.expanded()).len(), 7);
        s.collapse_all();
        assert_eq!(visible(&data(), s.expanded()).len(), 2);
    }

    #[test]
    fn reveal_opens_the_way_to_a_node() {
        let mut s = TreeState::new();
        s.reveal(&data(), "Card");
        assert!(s.is_expanded("src") && s.is_expanded("components"));
        assert!(!s.is_expanded("docs"));
    }

    #[test]
    fn filter_keeps_matches_and_their_ancestors() {
        let f = filter(&data(), "card");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].value, "src");
        assert_eq!(f[0].children.len(), 1);
        assert_eq!(f[0].children[0].children[0].value, "Card");
        assert!(filter(&data(), "zzz").is_empty());
        assert_eq!(
            filter(&data(), "  ").len(),
            2,
            "a blank query keeps everything"
        );
    }

    #[test]
    fn a_matching_branch_keeps_only_matching_children() {
        let f = filter(&data(), "src");
        assert_eq!(f.len(), 1);
        assert!(f[0].children.is_empty());
    }

    #[test]
    fn matching_ancestors_are_the_nodes_on_the_way_to_a_match() {
        let m = matching_ancestors(&data(), "button");
        assert_eq!(m, ["src", "components", "Button"]);
        assert!(matching_ancestors(&data(), "zzz").is_empty());
    }

    #[test]
    fn match_ranges_find_each_occurrence_ignoring_case() {
        assert_eq!(match_ranges("Banana", "an"), vec![1..3, 3..5]);
        assert_eq!(match_ranges("Button", "BUT"), vec![0..3]);
        assert!(match_ranges("Button", "").is_empty());
        assert!(match_ranges("Button", "x").is_empty());
        assert_eq!(match_ranges("Ünïcode", "ï"), vec![3..5]);
    }

    #[test]
    fn single_selection_replaces_and_multiple_adds() {
        let mut s = TreeState::new();
        s.select("a");
        s.select("b");
        assert_eq!(s.selected(), ["b"]);
        s.multiple = true;
        s.select("c");
        assert_eq!(s.selected(), ["b", "c"]);
        s.toggle_selected("b");
        assert_eq!(s.selected(), ["c"]);
        s.clear_selected();
        assert!(s.selected().is_empty());
    }

    #[test]
    fn a_range_runs_over_the_visible_rows_from_the_anchor() {
        let mut s = TreeState::new();
        s.multiple = true;
        s.expand("src");
        s.select("src");
        s.select_range(&data(), "theme");
        assert_eq!(s.selected(), ["src", "components", "theme"]);
        // "Button" is hidden inside a closed branch: not part of the range.
        assert!(!s.is_selected("Button"));
    }

    #[test]
    fn checking_a_branch_checks_everything_below() {
        let mut s = TreeState::new();
        s.check(&data(), "components");
        assert!(s.is_checked("components") && s.is_checked("Button") && s.is_checked("Card"));
        assert!(!s.is_checked("src"), "theme is not checked yet");
        assert!(s.is_indeterminate(&data(), "src"));
        s.check(&data(), "theme");
        assert!(
            s.is_checked("src"),
            "all children checked: the parent is too"
        );
        assert!(!s.is_indeterminate(&data(), "src"));
    }

    #[test]
    fn unchecking_a_leaf_unchecks_the_branches_above() {
        let mut s = TreeState::new();
        s.check(&data(), "src");
        s.uncheck(&data(), "Card");
        assert!(!s.is_checked("Card") && !s.is_checked("components") && !s.is_checked("src"));
        assert!(s.is_checked("Button") && s.is_checked("theme"));
        assert!(s.is_indeterminate(&data(), "components"));
    }

    #[test]
    fn strict_checking_touches_only_the_node() {
        let mut s = TreeState::new();
        s.check_strictly = true;
        s.check(&data(), "components");
        assert!(s.is_checked("components") && !s.is_checked("Button"));
        assert!(!s.is_indeterminate(&data(), "src"));
        s.uncheck(&data(), "components");
        assert!(!s.is_checked("components"));
    }

    #[test]
    fn checked_nodes_come_in_tree_order() {
        let d = data();
        let mut s = TreeState::new();
        s.check(&d, "docs");
        s.check(&d, "Button");
        let v: Vec<&str> = s
            .checked_nodes(&d)
            .iter()
            .map(|n| n.value.as_str())
            .collect();
        assert_eq!(v, ["Button", "docs", "README"]);
    }

    #[test]
    fn a_click_selects_and_opens_a_branch() {
        let mut d = data();
        let mut s = TreeState::new();
        s.apply(&mut d, Event::Click("src".into()));
        assert_eq!(s.selected(), ["src"]);
        assert!(s.is_expanded("src"));
        s.apply(&mut d, Event::Click("src".into()));
        assert!(!s.is_expanded("src"), "a second click closes it");
        s.expand_on_click = false;
        s.apply(&mut d, Event::Click("docs".into()));
        assert!(!s.is_expanded("docs"));
    }

    #[test]
    fn the_arrow_only_toggles_and_a_checkbox_checks() {
        let mut d = data();
        let mut s = TreeState::new();
        s.apply(&mut d, Event::Toggle("src".into()));
        assert!(s.is_expanded("src") && s.selected().is_empty());
        s.apply(&mut d, Event::Check("docs".into(), true));
        assert!(s.is_checked("README"));
        s.apply(&mut d, Event::Check("docs".into(), false));
        assert!(!s.is_checked("README"));
    }

    fn rows() -> Vec<Row> {
        let mut s = TreeState::new();
        s.expand("src");
        visible(&data(), s.expanded()) // src, components, theme, docs
    }
    fn go(focused: &str, key: NavKey) -> Option<Nav> {
        navigate(&rows(), Some(focused), key)
    }
    fn focus(v: &str) -> Option<Nav> {
        Some(Nav::Focus(v.into()))
    }

    #[test]
    fn up_down_home_end_move_through_the_visible_rows() {
        assert_eq!(go("src", NavKey::Down), focus("components"));
        assert_eq!(go("theme", NavKey::Up), focus("components"));
        assert_eq!(go("docs", NavKey::Down), None);
        assert_eq!(go("src", NavKey::Up), None);
        assert_eq!(go("theme", NavKey::Home), focus("src"));
        assert_eq!(go("theme", NavKey::End), focus("docs"));
        assert_eq!(navigate(&rows(), None, NavKey::Down), focus("src"));
        assert_eq!(navigate(&rows(), None, NavKey::Up), focus("docs"));
    }

    #[test]
    fn right_opens_then_enters_and_left_closes_then_leaves() {
        assert_eq!(
            go("components", NavKey::Right),
            Some(Nav::Expand("components".into()))
        );
        assert_eq!(
            go("src", NavKey::Right),
            focus("components"),
            "open: go to the first child"
        );
        assert_eq!(go("theme", NavKey::Right), None, "a leaf");
        assert_eq!(go("src", NavKey::Left), Some(Nav::Collapse("src".into())));
        assert_eq!(
            go("theme", NavKey::Left),
            focus("src"),
            "a leaf goes to its parent"
        );
        assert_eq!(
            go("docs", NavKey::Left),
            None,
            "a closed root has no parent"
        );
    }

    #[test]
    fn gtk_keys_expand_collapse_and_toggle() {
        assert_eq!(go("docs", NavKey::Expand), Some(Nav::Expand("docs".into())));
        assert_eq!(go("src", NavKey::Expand), None, "already open");
        assert_eq!(
            go("src", NavKey::Collapse),
            Some(Nav::Collapse("src".into()))
        );
        assert_eq!(go("docs", NavKey::Collapse), None, "already closed");
        assert_eq!(go("src", NavKey::Toggle), Some(Nav::Collapse("src".into())));
        assert_eq!(go("docs", NavKey::Toggle), Some(Nav::Expand("docs".into())));
        assert_eq!(go("theme", NavKey::Toggle), None);
    }

    fn tops(d: &[TreeNode]) -> Vec<&str> {
        d.iter().map(|n| n.value.as_str()).collect()
    }

    #[test]
    fn a_node_moves_before_after_or_inside_another() {
        let mut d = data();
        assert!(move_node(&mut d, "docs", "src", Position::Before));
        assert_eq!(tops(&d), ["docs", "src"]);
        assert!(move_node(&mut d, "docs", "src", Position::After));
        assert_eq!(tops(&d), ["src", "docs"]);
        assert!(move_node(&mut d, "theme", "docs", Position::Inside));
        assert_eq!(
            d[1].children
                .iter()
                .map(|n| n.value.as_str())
                .collect::<Vec<_>>(),
            ["README", "theme"]
        );
        assert_eq!(d[0].children.len(), 1, "it left its old place");
        assert!(move_node(&mut d, "Button", "README", Position::After));
        assert_eq!(
            d[1].children[1].value, "Button",
            "a sibling of the target, at any depth"
        );
    }

    #[test]
    fn a_node_cannot_go_into_itself_or_its_own_branch() {
        let mut d = data();
        let before = d.clone();
        assert!(!move_node(&mut d, "src", "src", Position::Inside));
        assert!(!move_node(&mut d, "src", "Button", Position::Before));
        assert!(!move_node(&mut d, "src", "nope", Position::After));
        assert!(!move_node(&mut d, "nope", "src", Position::After));
        assert_eq!(d, before);
        assert!(!can_move(&d, "components", "Card"));
        assert!(can_move(&d, "Card", "components"));
    }

    #[test]
    fn dropping_inside_opens_the_target() {
        let mut d = data();
        let mut s = TreeState::new();
        s.apply(
            &mut d,
            Event::Drop {
                from: "theme".into(),
                to: "docs".into(),
                position: Position::Inside,
            },
        );
        assert!(s.is_expanded("docs"));
        s.apply(
            &mut d,
            Event::Drop {
                from: "docs".into(),
                to: "docs".into(),
                position: Position::Inside,
            },
        );
        assert_eq!(d.len(), 2, "refused: unchanged");
    }

    #[test]
    fn the_pointer_picks_before_inside_or_after() {
        assert_eq!(drop_position(0.1, true), Position::Before);
        assert_eq!(drop_position(0.5, true), Position::Inside);
        assert_eq!(drop_position(0.9, true), Position::After);
        assert_eq!(
            drop_position(0.4, false),
            Position::Before,
            "no inside: the nearer edge"
        );
        assert_eq!(drop_position(0.6, false), Position::After);
    }

    #[test]
    fn a_lazy_branch_asks_for_its_children_once_and_merges_them() {
        let mut d = vec![TreeNode::lazy("a", "A"), n("b")];
        let mut s = TreeState::new();
        assert_eq!(
            s.apply(&mut d, Event::Toggle("a".into())),
            Some(Effect::Load("a".into()))
        );
        assert!(s.is_loading("a") && s.is_expanded("a"));
        assert!(s.rows(&d)[0].loading);
        // Closed and opened again while it loads: no second request.
        s.apply(&mut d, Event::Toggle("a".into()));
        assert_eq!(s.apply(&mut d, Event::Toggle("a".into())), None);
        s.finish_load(&mut d, "a", Ok(vec![n("a1"), n("a2")]));
        assert!(!s.is_loading("a"));
        assert_eq!(values(&s.rows(&d)), ["a", "a1", "a2", "b"]);
        // Loaded: opening it again does not ask.
        s.apply(&mut d, Event::Toggle("a".into()));
        assert_eq!(s.apply(&mut d, Event::Toggle("a".into())), None);
    }

    #[test]
    fn a_failed_load_keeps_the_error_and_retry_asks_again() {
        let mut d = vec![TreeNode::lazy("a", "A")];
        let mut s = TreeState::new();
        s.apply(&mut d, Event::Toggle("a".into()));
        s.finish_load(&mut d, "a", Err("offline".into()));
        assert_eq!(s.load_error("a"), Some("offline"));
        assert_eq!(s.rows(&d)[0].error.as_deref(), Some("offline"));
        assert_eq!(
            s.apply(&mut d, Event::Retry("a".into())),
            Some(Effect::Load("a".into()))
        );
        assert_eq!(s.load_error("a"), None);
        assert!(d[0].lazy, "still not loaded");
    }

    #[test]
    fn a_lazy_branch_counts_as_having_children() {
        let d = vec![TreeNode::lazy("a", "A")];
        assert!(visible(&d, &HashSet::new())[0].has_children);
    }

    #[test]
    fn fuzzy_matching_is_a_case_insensitive_subsequence() {
        assert!(fuzzy("bttn", &n("Button")));
        assert!(fuzzy("", &n("Button")));
        assert!(!fuzzy("nb", &n("Button")));
        assert_eq!(fuzzy_ranges("Button", "bn"), vec![0..1, 5..6]);
        assert!(fuzzy_ranges("Button", "nb").is_empty());
        let f = filter_with(&data(), "cd", &fuzzy);
        assert_eq!(f[0].children[0].children[0].value, "Card");
    }

    #[test]
    fn the_window_covers_the_viewport_plus_overscan() {
        assert_eq!(window(1000, 0.0, 100.0, 20.0, 2), 0..7);
        assert_eq!(window(1000, 400.0, 100.0, 20.0, 2), 18..27);
        assert_eq!(
            window(10, 1000.0, 100.0, 20.0, 2),
            10..10,
            "scrolled past the end"
        );
        assert_eq!(window(0, 0.0, 100.0, 20.0, 2), 0..0);
    }
}
