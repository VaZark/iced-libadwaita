//! The Tree pages, after Mantine's Tree page: the playground (lines, level offset, checkboxes,
//! strict checking, multiple selection, expand on click, icons, the three kinds of search,
//! drag and drop with locked nodes), async loading, a 2,000-node windowed tree, and the two
//! examples (a files tree and a docs navigation editor).
//!
//! The playground's controls live in `playground.rs`; [`State::sync`] applies the ones that
//! belong to the tree's state (selection mode, strict checking, the search).

use iced_libadwaita as adw;

use crate::playground::Playground;
use adw::primitives::tree as t;
use adw::primitives::virtualize::Viewport;
use adw::widget::{
    self as w, DragMode, TREE_ROW_HEIGHT, Tree, TreeEffect, TreeEvent, TreeNode, TreeState,
    typography as ty,
};
use adw::{Element, Icon, Metrics};
use iced::widget::{column, row, scrollable};
use iced::{Length, Task};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Msg {
    Tree(TreeEvent),
    ExpandAll,
    CollapseAll,
    CheckAll,
    UncheckAll,
    Lazy(TreeEvent),
    Loaded(String, Result<Vec<TreeNode>, String>),
    Big(TreeEvent),
    Scroll(Viewport),
    Files(TreeEvent),
    Docs(TreeEvent),
    DocsSearch(String),
}

pub struct State {
    data: Vec<TreeNode>,
    tree: TreeState,
    /// What the playground shows: `data`, or `data` filtered by the search.
    shown: Vec<TreeNode>,
    query: String,
    mode: usize,
    lazy: Vec<TreeNode>,
    lazy_tree: TreeState,
    /// The branches whose load has already failed once (the demo fails each one once).
    failed: std::collections::HashSet<String>,
    big: Vec<TreeNode>,
    big_tree: TreeState,
    vp: Viewport,
    files: Vec<TreeNode>,
    files_tree: TreeState,
    docs: Vec<TreeNode>,
    docs_tree: TreeState,
    docs_search: String,
}

fn f(v: &str) -> TreeNode {
    TreeNode::new(v, v.rsplit('/').next().unwrap_or(v))
}
fn d(v: &str, kids: Vec<TreeNode>) -> TreeNode {
    TreeNode::branch(v, v.rsplit('/').next().unwrap_or(v), kids)
}

/// A small project, like the one on Mantine's page.
fn project() -> Vec<TreeNode> {
    vec![
        d(
            "src",
            vec![
                d(
                    "src/components",
                    vec![
                        d(
                            "src/components/Accordion",
                            vec![
                                f("src/components/Accordion/Accordion.tsx"),
                                f("src/components/Accordion/Accordion.css"),
                            ],
                        ),
                        d(
                            "src/components/Tree",
                            vec![
                                f("src/components/Tree/Tree.tsx"),
                                f("src/components/Tree/Tree.css"),
                            ],
                        ),
                        f("src/components/index.ts"),
                    ],
                ),
                d(
                    "src/theme",
                    vec![f("src/theme/colors.ts"), f("src/theme/spacing.ts")],
                ),
                f("src/main.rs"),
            ],
        ),
        d("locked", vec![f("locked/secret.txt")]),
        d("docs", vec![f("docs/README.md"), f("docs/CHANGELOG.md")]),
        f("Cargo.toml"),
        f("LICENSE"),
    ]
}

fn big() -> Vec<TreeNode> {
    (0..20)
        .map(|g| {
            let kids = (0..100).map(|i| f(&format!("g{g}/item {i:03}"))).collect();
            TreeNode::branch(format!("g{g}"), format!("Group {g}"), kids)
        })
        .collect()
}

fn docs() -> Vec<TreeNode> {
    vec![
        d(
            "Getting started",
            vec![f("Installation"), f("Usage"), f("Theming")],
        ),
        d(
            "Components",
            vec![
                f("Button"),
                f("Tree"),
                d("Forms", vec![f("Field"), f("Stepper")]),
            ],
        ),
        d("Guides", vec![f("Migration"), f("FAQ")]),
    ]
}

impl Default for State {
    fn default() -> Self {
        let data = project();
        let mut files_tree = TreeState::new();
        files_tree.set_expanded(["src".to_string(), "src/components".to_string()]);
        let mut big_tree = TreeState::new();
        let big = big();
        big_tree.expand_all(&big);
        let mut docs_tree = TreeState::new();
        let docs = docs();
        docs_tree.expand_all(&docs);
        Self {
            shown: data.clone(),
            files: data.clone(),
            data,
            tree: TreeState::new(),
            query: String::new(),
            mode: 0,
            lazy: vec![
                TreeNode::lazy("remote", "remote"),
                TreeNode::lazy("broken", "broken (fails once)"),
                f("local.txt"),
            ],
            lazy_tree: TreeState::new(),
            failed: Default::default(),
            big,
            big_tree,
            vp: Viewport::new(360.0),
            files_tree,
            docs,
            docs_tree,
            docs_search: String::new(),
        }
    }
}

/// The children a lazy branch gets after a short wait.
async fn load(value: String, fail: bool) -> Result<Vec<TreeNode>, String> {
    tokio::time::sleep(Duration::from_millis(700)).await;
    if fail {
        return Err("Could not reach the server".into());
    }
    Ok(vec![
        TreeNode::lazy(format!("{value}/nested"), "nested (loads too)"),
        f(&format!("{value}/one.txt")),
        f(&format!("{value}/two.txt")),
    ])
}

fn icon_for(n: &TreeNode, r: &t::Row) -> Option<Icon> {
    Some(if r.has_children {
        Icon::Folder
    } else if n.value.ends_with(".md") {
        Icon::Edit
    } else {
        Icon::Document
    })
}

impl State {
    /// Applies the playground's controls that belong to the tree's state.
    pub fn sync(&mut self, p: &Playground) {
        self.tree.multiple = p.bool("multiple");
        self.tree.check_strictly = p.bool("strict");
        self.tree.expand_on_click = p.bool("expand_on_click");
        let (q, mode) = (p.text("search").to_string(), p.choice("search_mode"));
        if (q.as_str(), mode) != (self.query.as_str(), self.mode) {
            self.query = q;
            self.mode = mode;
            self.refresh();
            let blank = self.query.trim().is_empty() || self.mode == 0;
            if blank {
                self.tree.collapse_all();
            } else if self.mode == 1 {
                self.tree
                    .set_expanded(t::matching_ancestors(&self.data, &self.query));
            } else {
                self.tree.expand_all(&self.shown);
            }
        }
    }

    fn refresh(&mut self) {
        self.shown = match self.mode {
            2 => t::filter(&self.data, &self.query),
            3 => t::filter_with(&self.data, &self.query, &t::fuzzy),
            _ => self.data.clone(),
        };
    }

    pub fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Tree(e) => {
                self.tree.apply(&mut self.data, e);
                self.refresh();
            }
            Msg::ExpandAll => self.tree.expand_all(&self.shown),
            Msg::CollapseAll => self.tree.collapse_all(),
            Msg::CheckAll => self.tree.check_all(&self.data),
            Msg::UncheckAll => self.tree.uncheck_all(),
            Msg::Lazy(e) => {
                if let Some(TreeEffect::Load(v)) = self.lazy_tree.apply(&mut self.lazy, e) {
                    let fail = v.ends_with("broken") && self.failed.insert(v.clone());
                    return Task::perform(load(v.clone(), fail), move |r| {
                        Msg::Loaded(v.clone(), r)
                    });
                }
            }
            Msg::Loaded(v, r) => self.lazy_tree.finish_load(&mut self.lazy, &v, r),
            Msg::Big(e) => {
                self.big_tree.apply(&mut self.big, e);
            }
            Msg::Scroll(v) => self.vp = v,
            Msg::Files(e) => {
                self.files_tree.apply(&mut self.files, e);
            }
            Msg::Docs(e) => {
                self.docs_tree.apply(&mut self.docs, e);
            }
            Msg::DocsSearch(q) => {
                if q.trim().is_empty() {
                    self.docs_tree.expand_all(&self.docs);
                } else {
                    self.docs_tree
                        .set_expanded(t::matching_ancestors(&self.docs, &q));
                }
                self.docs_search = q;
            }
        }
        Task::none()
    }

    /// The playground: one tree and the buttons Mantine's `useTree` demos have.
    pub fn play<'a>(&'a self, p: &Playground, m: Metrics) -> (Element<'a, Msg>, String) {
        let drag = [DragMode::Off, DragMode::Row, DragMode::Handle][p.choice("drag")];
        let mut tree = Tree::new(&self.shown, &self.tree)
            .lines(p.bool("lines"))
            .checkboxes(p.bool("checkboxes"))
            .draggable(drag)
            .metrics(m)
            .on_event(Msg::Tree);
        if p.changed("offset") {
            tree = tree.level_offset(p.number("offset"));
        }
        if p.bool("icons") {
            tree = tree.icons(icon_for);
        }
        if p.bool("locked") {
            // A locked node cannot be dragged, and nothing can be dropped on the locked folder.
            tree = tree.allow_drop(|from, to, _| {
                !from.value.starts_with("locked") && to.value != "locked"
            });
        }
        if p.choice("search_mode") == 1 || p.choice("search_mode") == 3 {
            tree = tree
                .highlight(p.text("search"))
                .fuzzy(p.choice("search_mode") == 3);
        }
        if p.choice("search_mode") == 2 {
            tree = tree.highlight(p.text("search"));
        }
        let buttons = row![
            w::raised_button(ty::label("Expand all")).on_press(Msg::ExpandAll),
            w::raised_button(ty::label("Collapse all")).on_press(Msg::CollapseAll),
            w::raised_button(ty::label("Check all")).on_press(Msg::CheckAll),
            w::raised_button(ty::label("Uncheck all")).on_press(Msg::UncheckAll),
        ]
        .spacing(m.space_s);
        let checked = self.tree.checked_nodes(&self.data).len();
        let preview: Element<'a, Msg> = column![
            buttons,
            Element::from(tree),
            ty::caption(format!(
                "selected: {} · checked: {}",
                self.tree.selected().join(", "),
                checked
            ))
        ]
        .spacing(m.space_m)
        .into();

        let mut code = String::from("Tree::new(&self.data, &self.tree)\n");
        let mut line = |s: String| {
            code.push_str("    ");
            code.push_str(&s);
            code.push('\n');
        };
        if p.bool("lines") {
            line(".lines(true)".into());
        }
        if p.changed("offset") {
            line(format!(".level_offset({:.0})", p.number("offset")));
        }
        if p.bool("checkboxes") {
            line(".checkboxes(true)".into());
        }
        if p.bool("icons") {
            line(".icons(|node, row| Some(if row.has_children { Icon::Folder } else { Icon::Document }))".into());
        }
        match p.choice("search_mode") {
            1 => line(".highlight(&self.search)          // data unfiltered; matches marked".into()),
            2 => line(".highlight(&self.search)          // data = tree::filter(&data, &search)".into()),
            3 => line(".highlight(&self.search).fuzzy(true) // data = filter_with(&data, &search, &fuzzy)".into()),
            _ => {}
        }
        if drag != DragMode::Off {
            line(format!(
                ".draggable(DragMode::{})",
                ["Off", "Row", "Handle"][p.choice("drag")]
            ));
        }
        if p.bool("locked") {
            line(".allow_drop(|from, to, _| !from.value.starts_with(\"locked\") && to.value != \"locked\")".into());
        }
        line(".on_event(Msg::Tree)".into());
        code.push_str("\n// update: self.tree.apply(&mut self.data, event);");
        if p.bool("multiple") || p.bool("strict") || p.changed("expand_on_click") {
            code.push_str("\n// state: ");
            let mut opts = Vec::new();
            if p.bool("multiple") {
                opts.push("tree.multiple = true");
            }
            if p.bool("strict") {
                opts.push("tree.check_strictly = true");
            }
            if !p.bool("expand_on_click") {
                opts.push("tree.expand_on_click = false");
            }
            code.push_str(&opts.join("; "));
        }
        (preview, code)
    }

    /// Async loading: a spinner while a branch loads, the error and Retry when it fails.
    pub fn async_view(&self, m: Metrics) -> Element<'_, Msg> {
        column![
            ty::caption(
                "Open a branch: its children load after a moment. “broken” fails once; press Retry."
            ),
            Tree::new(&self.lazy, &self.lazy_tree)
                .icons(icon_for)
                .lines(true)
                .metrics(m)
                .on_event(Msg::Lazy)
        ]
        .spacing(m.space_m)
        .into()
    }

    /// 2,000 nodes, of which only the rows in view are built.
    pub fn virtual_view(&self, m: Metrics) -> Element<'_, Msg> {
        let total = self.big_tree.rows(&self.big).len();
        let range = self.vp.window(total, TREE_ROW_HEIGHT, 4);
        column![
            ty::caption(format!("{total} rows, {} built", range.len())),
            scrollable(
                Tree::new(&self.big, &self.big_tree)
                    .window(range)
                    .icons(icon_for)
                    .metrics(m)
                    .on_event(Msg::Big)
            )
            .on_scroll(|vp| {
                let mut v = Viewport::default();
                v.set(vp.absolute_offset().y, vp.bounds().height);
                Msg::Scroll(v)
            })
            .height(360)
        ]
        .spacing(m.space_m)
        .into()
    }

    /// Mantine's files tree: lines, icons by type, selection.
    pub fn files_view(&self, m: Metrics) -> Element<'_, Msg> {
        Tree::new(&self.files, &self.files_tree)
            .lines(true)
            .icons(icon_for)
            .metrics(m)
            .on_event(Msg::Files)
            .into()
    }

    /// Mantine's docs navigation editor: search, drag and drop, selection, a count per branch.
    pub fn docs_view(&self, m: Metrics) -> Element<'_, Msg> {
        let tree = Tree::new(&self.docs, &self.docs_tree)
            .draggable(DragMode::Row)
            .highlight(&self.docs_search)
            .label(|p| {
                let n = p.node.children.len();
                if n == 0 {
                    return p.text;
                }
                row![p.text, w::Badge::count(n as u32, 99)]
                    .spacing(8)
                    .into()
            })
            .metrics(m)
            .on_event(Msg::Docs);
        column![
            w::entry("Search pages…", &self.docs_search)
                .on_input(Msg::DocsSearch)
                .metrics(m),
            Element::from(tree)
        ]
        .spacing(m.space_m)
        .width(Length::Fill)
        .into()
    }
}
