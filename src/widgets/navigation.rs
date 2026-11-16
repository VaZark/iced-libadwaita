//! `AdwNavigationSplitView`, `AdwOverlaySplitView`, `AdwNavigationView`,
//! `.navigation-sidebar`, and `AdwBreakpoint`.

use crate::Element;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::primitives::drag::{DropHint, Place, Targets};
use crate::styles::{button as sb, container as sc};
use crate::widgets::{drag_source, drop_zone_at, header_bar, toolbar_view};
use crate::{Icon, IconSource, Metrics, Theme};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, mouse_area, opaque, row, space, stack};
use iced::{Length, Size};

fn geometry(number: impl Fn(&str) -> Option<f32>, key: &str, fallback: f32) -> f32 {
    number(key)
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(fallback)
}

fn navigation_geometry(key: &str, fallback: f32) -> f32 {
    geometry(
        |key| {
            crate::theme_config::current()
                .tokens()
                .variant_number("navigation", "default", key)
        },
        key,
        fallback,
    )
}

#[derive(Clone)]
struct NavigationDensity {
    metrics: Metrics,
    tokens: std::sync::Arc<crate::theme_config::Tokens>,
    variant: String,
}

impl NavigationDensity {
    fn current(padding: SidebarPadding) -> Self {
        Self {
            metrics: Metrics::desktop(),
            tokens: std::sync::Arc::new(crate::theme_config::current().tokens()),
            variant: padding.variant_name().into(),
        }
    }

    fn geometry(&self) -> SidebarGeometry {
        navigation_geometry_with_tokens(&self.metrics, &self.tokens, &self.variant)
    }

    fn number(&self, key: &str, fallback: f32) -> f32 {
        geometry(
            |key| self.tokens.variant_number("navigation", &self.variant, key),
            key,
            fallback,
        )
    }

    fn for_item(&self, padding: Option<SidebarPadding>) -> Self {
        let mut density = self.clone();
        if let Some(padding) = padding {
            density.variant = padding.variant_name().into();
        }
        density
    }
}

/// `AdwBreakpoint` condition on window width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Breakpoint {
    max_width: Option<f32>,
    min_width: Option<f32>,
}

impl Breakpoint {
    /// `max-width: <px>`.
    pub fn max_width(px: f32) -> Self {
        Self {
            max_width: Some(px),
            min_width: None,
        }
    }
    /// `min-width: <px>`.
    pub fn min_width(px: f32) -> Self {
        Self {
            max_width: None,
            min_width: Some(px),
        }
    }
    /// Whether the breakpoint applies at `size`. Typical use inside
    /// `iced::widget::responsive`.
    pub fn matches(&self, size: Size) -> bool {
        self.max_width.is_none_or(|m| size.width <= m)
            && self.min_width.is_none_or(|m| size.width >= m)
    }
}

/// Compatibility IDs for the `sm`, `md`, and `lg` navigation config variants.
/// The enum defines no geometry: each key inherits from the variant, base navigation
/// tokens, then metrics, exactly like a custom [`SidebarList::variant`] name.
/// The shipped libadwaita config supplies the named density defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SidebarPadding {
    Sm,
    #[default]
    Md,
    Lg,
}

/// Resolved logical geometry for a navigation density variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebarGeometry {
    pub row_height: f32,
    pub row_padding: f32,
    pub gap: f32,
    pub inset: f32,
}

/// Resolve each geometry key from a declared variant, base navigation tokens, then
/// metrics. Familiar, custom, missing and undeclared names use the same path.
///
/// [`SidebarList::theme_config`] and [`SidebarList::available_width`] apply these
/// values to the actual rows, including pinned rows and pages.
pub fn navigation_geometry_with_tokens(
    m: &Metrics,
    tokens: &crate::theme_config::Tokens,
    variant: &str,
) -> SidebarGeometry {
    let resolve = |key, fallback| {
        geometry(
            |key| tokens.variant_number("navigation", variant, key),
            key,
            fallback,
        )
    };
    SidebarGeometry {
        row_height: resolve("row-height", m.sidebar_row_height),
        row_padding: resolve("row-padding", m.sidebar_row_padding),
        gap: resolve("gap", m.sidebar_spacing),
        inset: resolve("inset", m.sidebar_padding),
    }
}

/// Resolve local navigation breakpoint rules from the actual available container width.
pub fn navigation_variant_with_tokens(
    tokens: &crate::theme_config::Tokens,
    width: f32,
    fallback: &str,
) -> String {
    crate::breakpoints::variant_for_tokens(
        tokens,
        "navigation",
        &crate::breakpoints::Breakpoints::default(),
        width,
        fallback,
    )
}

impl SidebarPadding {
    /// Resolve any declared local navigation variant; unknown names inherit base geometry.
    pub fn variant_geometry_with_tokens(
        m: &Metrics,
        tokens: &crate::theme_config::Tokens,
        variant: &str,
    ) -> SidebarGeometry {
        navigation_geometry_with_tokens(m, tokens, variant)
    }

    /// Select a local navigation variant from container width and logical breakpoint rules.
    pub fn variant_for_width_with_tokens(
        tokens: &crate::theme_config::Tokens,
        width: f32,
        fallback: &str,
    ) -> String {
        navigation_variant_with_tokens(tokens, width, fallback)
    }

    pub const ALL: [SidebarPadding; 3] = [Self::Sm, Self::Md, Self::Lg];

    /// The config variant ID represented by this compatibility enum.
    pub const fn variant_name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// Resolve this config variant using local tokens and generic per-key inheritance.
    pub fn geometry_with_tokens(
        self,
        m: &Metrics,
        tokens: &crate::theme_config::Tokens,
    ) -> SidebarGeometry {
        navigation_geometry_with_tokens(m, tokens, self.variant_name())
    }

    /// Minimum row height resolved from installed tokens, then the supplied metrics.
    pub fn row_height(self, m: &Metrics) -> f32 {
        self.geometry_with_tokens(m, &crate::theme_config::current().tokens())
            .row_height
    }
    pub fn row_padding(self, m: &Metrics) -> f32 {
        self.geometry_with_tokens(m, &crate::theme_config::current().tokens())
            .row_padding
    }
    pub fn gap(self, m: &Metrics) -> f32 {
        self.geometry_with_tokens(m, &crate::theme_config::current().tokens())
            .gap
    }
    pub fn inset(self, m: &Metrics) -> f32 {
        self.geometry_with_tokens(m, &crate::theme_config::current().tokens())
            .inset
    }
}

impl std::fmt::Display for SidebarPadding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Sm => "Small",
            Self::Md => "Medium",
            Self::Lg => "Large",
        })
    }
}

/// A sidebar entry (`.navigation-sidebar` row) at the default [`SidebarPadding::Md`].
pub fn sidebar_row<'a, M: Clone + 'a>(
    icon: Option<Icon>,
    label: impl Into<String>,
    selected: bool,
    on_press: M,
) -> Element<'a, M> {
    sidebar_row_padded(icon, label, selected, on_press, SidebarPadding::Md)
}

/// A sidebar entry at the given [`SidebarPadding`].
pub fn sidebar_row_padded<'a, M: Clone + 'a>(
    icon: Option<Icon>,
    label: impl Into<String>,
    selected: bool,
    on_press: M,
    padding: SidebarPadding,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    sidebar_row_with(
        icon.map(|i| i.view(m.icon_size)),
        label,
        selected,
        on_press,
        padding,
    )
}

/// A sidebar entry with its icon already built: any [`IconSource`] (an app's own SVG), tinted
/// or not.
pub fn sidebar_row_with<'a, M: Clone + 'a>(
    icon: Option<crate::IconView<'a>>,
    label: impl Into<String>,
    selected: bool,
    on_press: M,
    padding: SidebarPadding,
) -> Element<'a, M> {
    sidebar_row_opening(icon, label, selected, on_press, padding, false)
}

/// As [`sidebar_row_with`], and with a chevron at the end when `opens`: the row leads to another
/// page of the sidebar, as a submenu row does.
pub fn sidebar_row_opening<'a, M: Clone + 'a>(
    icon: Option<crate::IconView<'a>>,
    label: impl Into<String>,
    selected: bool,
    on_press: M,
    padding: SidebarPadding,
    opens: bool,
) -> Element<'a, M> {
    sidebar_row_opening_with_props(
        icon,
        label,
        selected,
        on_press,
        padding,
        opens,
        SidebarRowProps::default(),
    )
}

/// Presentation of the concrete widgets inside a sidebar row.
#[derive(Clone, Default)]
pub struct SidebarRowProps {
    pub label_props: TextProps,
    pub label_container_props: ContainerProps,
    pub content_props: ContainerProps,
    pub button_props: ButtonProps,
}
impl SidebarRowProps {
    pub fn label_props(mut self, props: TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn label_container_props(mut self, props: ContainerProps) -> Self {
        self.label_container_props = props;
        self
    }
    pub fn content_props(mut self, props: ContainerProps) -> Self {
        self.content_props = props;
        self
    }
    pub fn button_props(mut self, props: ButtonProps) -> Self {
        self.button_props = props;
        self
    }
}

/// Sidebar row with owned typed presentation overrides, without changing selection or actions.
pub fn sidebar_row_opening_with_props<'a, M: Clone + 'a>(
    icon: Option<crate::IconView<'a>>,
    label: impl Into<String>,
    selected: bool,
    on_press: M,
    padding: SidebarPadding,
    opens: bool,
    props: SidebarRowProps,
) -> Element<'a, M> {
    sidebar_row_resolved(
        icon,
        label.into(),
        selected,
        on_press,
        opens,
        props,
        &NavigationDensity::current(padding),
    )
}

fn sidebar_row_resolved<'a, M: Clone + 'a>(
    icon: Option<crate::IconView<'a>>,
    label: String,
    selected: bool,
    on_press: M,
    opens: bool,
    props: SidebarRowProps,
    density: &NavigationDensity,
) -> Element<'a, M> {
    let m = density.metrics;
    let g = density.geometry();
    let mut r = row![].spacing(m.space_m).align_y(Vertical::Center);
    if let Some(i) = icon {
        r = r.push(i);
    }
    // libadwaita ellipsizes; we clip at the row edge so text never paints
    // outside the selection highlight.
    r = r.push(
        props.label_container_props.apply(
            container(
                props
                    .label_props
                    .apply(crate::components::typography::label(label)),
            )
            .width(Length::Fill)
            .clip(true),
        ),
    );
    if opens {
        r = r.push(Icon::GoNext.view(m.icon_size));
    }
    // Every variant uses a minimum-height strut so larger fonts can grow the row.
    // Shrink occupies zero width without being discarded by iced's Row::push.
    let content = container(
        row![r, space().width(Length::Shrink).height(g.row_height)].align_y(Vertical::Center),
    )
    .padding([0.0, g.row_padding])
    .align_y(Vertical::Center);
    // Resolve the final target style (including child overrides) inside the native
    // button's callback. Selection changes its fill, never its shape or input state.
    let target_style = props.button_props.style.clone();
    let btn = props.button_props.apply(
        button(props.content_props.apply(content.width(Length::Fill)))
            .padding(0)
            .width(Length::Fill)
            .on_press(on_press),
    );
    let opacity = density
        .tokens
        .variant_number("navigation", &density.variant, "selection-opacity")
        .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
        .unwrap_or(0.10);
    crate::layouts::Highlight::button(btn, selected, move |theme, status, progress| {
        let target = target_style.as_ref().map_or_else(
            || sb::sidebar_row(false)(theme, status),
            |style| style(theme, status),
        );
        sb::with_underlay(
            target,
            crate::color::alpha(theme.semantic.sidebar.fg, opacity * progress),
        )
    })
    .into()
}

/// Like [`navigation_sidebar`] but rounds the bottom corners to match a
/// rounded window (pass [`window::State::radius`](crate::windows::State::radius)).
pub fn navigation_sidebar_rounded<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
    window_radius: f32,
) -> Element<'a, M> {
    navigation_sidebar_rounded_padded(rows, window_radius, SidebarPadding::Md)
}

/// [`navigation_sidebar_rounded`] with the list's gap and horizontal inset at the given [`SidebarPadding`].
pub fn navigation_sidebar_rounded_padded<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
    window_radius: f32,
    padding: SidebarPadding,
) -> Element<'a, M> {
    let g = NavigationDensity::current(padding).geometry();
    sidebar_surface(rows, window_radius, g.gap, g.inset)
}

/// The rounded sidebar surface with the rows `gap` apart and `inset` around them.
fn sidebar_surface<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
    window_radius: f32,
    gap: f32,
    inset: f32,
) -> Element<'a, M> {
    // CSS has top/bottom list padding as well as horizontal row margins. The
    // last row's 2px bottom margin compensates the list's bottom padding - 2px.
    container(column(rows).spacing(gap).padding(inset))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &crate::Theme| iced::widget::container::Style {
            // Transparent: the split view's pane supplies the color (sidebar or, as a first
            // page, content). Both bottom corners: beside content the right one reveals the pane behind
            // (same color); when the sidebar is the whole window it is a real window corner.
            border: iced::Border::default().rounded(iced::border::bottom(window_radius)),
            ..Default::default()
        })
        .into()
}

/// Stacks sidebar rows with `.navigation-sidebar` spacing and horizontal inset.
pub fn navigation_sidebar<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let g = NavigationDensity::current(SidebarPadding::Md).geometry();
    container(column(rows).spacing(g.gap).padding(g.inset))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(sc::sidebar)
        .into()
}

/// One entry of a [`SidebarList`]: one description with two forms, a compact row
/// ([`SidebarStyle::List`]) and a card row ([`SidebarStyle::Cards`]).
#[derive(Clone)]
pub struct SidebarItem<M> {
    icon: Option<IconSource>,
    /// Recolors the icon (a colored folder), instead of the text color.
    tint: Option<iced::Color>,
    title: String,
    subtitle: Option<String>,
    selected: bool,
    on_press: M,
    padding: Option<SidebarPadding>,
    divider_above: bool,
    reorder: Option<Reorder<M>>,
    /// Nesting depth: how far the row is indented, in steps of [`INDENT`].
    indent: u8,
    /// Ends in a chevron: the row opens another page.
    opens: bool,
    /// A chevron at the row's end that folds what is under it: whether it is open, and what
    /// pressing it sends.
    expander: Option<(bool, M)>,
    /// A row that takes drops but cannot be picked up: what the copy says, and what is sent.
    drop_only: Option<(String, std::rc::Rc<dyn Fn(Place) -> M>)>,
    row_props: SidebarRowProps,
    card_props: super::rows::ActionRowProps,
}

/// How far a nested [`SidebarItem`] is indented per level.
pub const INDENT: f32 = 14.0;

/// What makes a row of a [`SidebarList`] movable among the others.
#[derive(Clone)]
struct Reorder<M> {
    on_start: M,
    label: String,
    on_drop: std::rc::Rc<dyn Fn(Place) -> M>,
    /// What the copy says over the middle of the row, which then takes the drop *inside* it
    /// ([`SidebarItem::nest`]); without it the row only has edges, before and after.
    inside: Option<String>,
}

impl<M> SidebarItem<M> {
    pub fn row_props(mut self, props: SidebarRowProps) -> Self {
        self.row_props = props;
        self
    }
    pub fn card_props(mut self, props: super::rows::ActionRowProps) -> Self {
        self.card_props = props;
        self
    }
    pub fn new(title: impl Into<String>, on_press: M) -> Self {
        Self {
            row_props: SidebarRowProps::default(),
            card_props: super::rows::ActionRowProps::default(),
            icon: None,
            tint: None,
            title: title.into(),
            subtitle: None,
            selected: false,
            on_press,
            padding: None,
            divider_above: false,
            reorder: None,
            indent: 0,
            opens: false,
            expander: None,
            drop_only: None,
        }
    }
    /// A chevron at the end of the row: it leads to another page of a paged list
    /// ([`SidebarList::pages`]), as a submenu row does.
    pub fn opens(mut self, opens: bool) -> Self {
        self.opens = opens;
        self
    }
    /// Indents the row `level` steps (a folder inside a folder), so a tree reads as one.
    pub fn indent(mut self, level: u8) -> Self {
        self.indent = level;
        self
    }
    /// A chevron at the end of the row, pointing down when `open` and right when not: pressing
    /// it sends `on_toggle`, and the row itself still selects.
    pub fn expander(mut self, open: bool, on_toggle: M) -> Self {
        self.expander = Some((open, on_toggle));
        self
    }
    /// Makes the middle of a [`reorder`](Self::reorder)able row a place to drop *into* it: the
    /// edges still put the dragged row before or after, the middle gets [`Place::Inside`], and
    /// `label` is what the copy says there ("Move into Work").
    pub fn nest(mut self, label: impl Into<String>) -> Self {
        if let Some(r) = &mut self.reorder {
            r.inside = Some(label.into());
        }
        self
    }
    /// A row that cannot be moved but takes what is dropped on it (always [`Place::Inside`]):
    /// `label` is what the copy says over it.
    pub fn drop_target(
        mut self,
        label: impl Into<String>,
        on_drop: impl Fn(Place) -> M + 'static,
    ) -> Self {
        self.drop_only = Some((label.into(), std::rc::Rc::new(on_drop)));
        self
    }
    /// Lets the row be moved among the others: press and hold it to pick it up (`on_start` is
    /// sent), and while a drag is going on ([`SidebarList::dragging`]) the other rows take it
    /// before or after themselves, drawing an insertion line on the edge, as in GTK. `label` is
    /// said beside the copy over an edge ("Move here"); `on_drop` gets the row it was dropped on
    /// the [`Place`] to put it.
    pub fn reorder(
        mut self,
        on_start: M,
        label: impl Into<String>,
        on_drop: impl Fn(Place) -> M + 'static,
    ) -> Self {
        self.reorder = Some(Reorder {
            on_start,
            label: label.into(),
            on_drop: std::rc::Rc::new(on_drop),
            inside: None,
        });
        self
    }
    /// Draws a [`divider`](crate::components::divider) above this row, to start a new section.
    pub fn divider_above(mut self) -> Self {
        self.divider_above = true;
        self
    }
    /// This row's [`SidebarPadding`], overriding the list's.
    pub fn padding(mut self, padding: SidebarPadding) -> Self {
        self.padding = Some(padding);
        self
    }
    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Draws the icon in `color` rather than the text color: a single-color icon, such as a
    /// folder that wears its epic's color.
    pub fn icon_color(mut self, color: iced::Color) -> Self {
        self.tint = Some(color);
        self
    }
    fn icon_view<'a>(&self, size: f32) -> Option<crate::IconView<'a>> {
        let view = self.icon.as_ref()?.view(size);
        Some(match self.tint {
            Some(color) => {
                view.style(move |_: &Theme, _| iced::widget::svg::Style { color: Some(color) })
            }
            None => view,
        })
    }
    /// A short description. Shown only in [`SidebarStyle::Cards`].
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// How a [`SidebarList`] presents its items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarStyle {
    /// `.navigation-sidebar`: compact selectable rows on the sidebar color, for a sidebar
    /// beside the content or in a drawer. Subtitles are hidden.
    #[default]
    List,
    /// A page of cards: a padded, scrolling boxed list on the content color, each row with
    /// its icon, title, **subtitle** and a chevron, as in a mobile settings app. Use it for
    /// the sidebar as the parent page ([`CollapsedSidebar::ParentPage`]) when collapsed.
    Cards,
}

/// The pages of a paged [`SidebarList`].
#[derive(Clone)]
struct Pages<M> {
    pages: Vec<Vec<SidebarItem<M>>>,
    current: usize,
    depths: Vec<u32>,
}

/// The navigation entries of a sidebar, in either [`SidebarStyle`]. One component, so an
/// app describes its destinations once and the presentation follows the layout.
#[derive(Clone)]
pub struct SidebarList<'a, M> {
    items: Vec<SidebarItem<M>>,
    style: SidebarStyle,
    window_radius: f32,
    padding: SidebarPadding,
    variant: Option<String>,
    tokens: Option<crate::theme_config::Tokens>,
    metrics: Metrics,
    available_width: Option<f32>,
    spacing: Option<f32>,
    inset: Option<f32>,
    cards_below: Option<crate::breakpoints::WidthClass>,
    fixed_top: usize,
    dragging: bool,
    pages: Option<Pages<M>>,
    _p: std::marker::PhantomData<&'a ()>,
}

impl<'a, M: Clone + 'a> SidebarList<'a, M> {
    pub fn new(items: impl IntoIterator<Item = SidebarItem<M>>) -> Self {
        Self {
            items: items.into_iter().collect(),
            style: SidebarStyle::List,
            window_radius: 0.0,
            padding: SidebarPadding::Md,
            variant: None,
            tokens: None,
            metrics: Metrics::desktop(),
            available_width: None,
            spacing: None,
            inset: None,
            cards_below: None,
            fixed_top: 0,
            dragging: false,
            pages: None,
            _p: Default::default(),
        }
    }
    /// Use local tokens without changing the installed configuration or inherited metrics.
    pub fn tokens(mut self, tokens: crate::theme_config::Tokens) -> Self {
        self.tokens = Some(tokens);
        self
    }

    /// Use this configuration's tokens locally; draw-time colors still come from the theme.
    pub fn theme_config(self, config: &crate::theme_config::ThemeConfig) -> Self {
        self.tokens(config.tokens())
    }

    /// Inherited layout metrics; unset variant keys keep these values.
    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// A named density variant, overriding breakpoint rules. This and `padding` are
    /// equivalent explicit selections: the last call wins. Item padding wins over both.
    pub fn variant(mut self, name: impl Into<String>) -> Self {
        self.variant = Some(name.into());
        self
    }

    /// Resolve rules using this container width, without adding a responsive wrapper.
    /// Stored until rendering so builder order with `tokens` does not matter.
    pub fn available_width(mut self, width: f32) -> Self {
        self.available_width = Some(width);
        self
    }

    fn density(&self, tokens: crate::theme_config::Tokens) -> NavigationDensity {
        let variant = self.variant.clone().unwrap_or_else(|| {
            self.available_width.map_or_else(
                || self.padding.variant_name().into(),
                |width| navigation_variant_with_tokens(&tokens, width, self.padding.variant_name()),
            )
        });
        NavigationDensity {
            metrics: self.metrics,
            tokens: std::sync::Arc::new(tokens),
            variant,
        }
    }

    pub fn style(mut self, style: SidebarStyle) -> Self {
        self.style = style;
        self
    }
    /// Select the config density ID represented by [`SidebarPadding`]. Geometry is
    /// config-loaded, not defined by the enum. An item's [`SidebarItem::padding`] wins.
    pub fn padding(mut self, padding: SidebarPadding) -> Self {
        self.padding = padding;
        self.variant = Some(padding.variant_name().into());
        self
    }
    /// The gap between rows in px, over the one the [`padding`](Self::padding) level gives.
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The inset between the list's edges and its rows in px, over the level's.
    pub fn inset(mut self, px: f32) -> Self {
        self.inset = Some(px);
        self
    }
    /// Chooses the style by the width of the list's own container, with the same rule as
    /// [`MetaComponent`](crate::metas::MetaComponent) (as [`ResponsiveTable`](crate::widgets::ResponsiveTable)
    /// does): cards below this width class, the compact list from it. Overrides
    /// [`style`](Self::style) and [`follow`](Self::follow). The list then fills its container.
    pub fn cards_below(mut self, class: crate::breakpoints::WidthClass) -> Self {
        self.cards_below = Some(class);
        self
    }
    /// Keeps the first `n` items where they are and scrolls the rest beneath them, so a long list
    /// of items (folders, say) scrolls while the entries that matter most stay in view
    /// ([`SidebarStyle::List`] only; the dividers count as part of the item below them).
    pub fn fixed_top(mut self, n: usize) -> Self {
        self.fixed_top = n;
        self
    }
    /// Replaces what scrolls (everything after [`fixed_top`](Self::fixed_top)) with pages, one
    /// showing, the way a menu's submenu takes the place of its parent: `pages` are lists of
    /// items, `current` is the one to show, and `depths` says how deep each is (so a change to a
    /// deeper page slides forward and to a shallower one back). The page shown is the app's to
    /// name, so it follows whatever the app does: a row that opens a page, a back button, a link
    /// elsewhere. The items given to [`new`](Self::new) after `fixed_top` are then not used.
    /// [`SidebarStyle::List`] only.
    pub fn pages(
        mut self,
        pages: Vec<Vec<SidebarItem<M>>>,
        current: usize,
        depths: Vec<u32>,
    ) -> Self {
        self.pages = Some(Pages {
            pages,
            current,
            depths,
        });
        self
    }
    /// Whether a drag of one of the [`reorder`](SidebarItem::reorder)able rows is going on: they
    /// are only drop targets then.
    pub fn dragging(mut self, dragging: bool) -> Self {
        self.dragging = dragging;
        self
    }
    /// Window radius for the [`SidebarStyle::List`] bottom corners.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
    /// Makes the presentation follow the layout an [`Adaptive`](crate::adaptive::Adaptive) shows:
    /// `present` maps the shown layout to a style and a padding. The items stay the same in
    /// every layout (describe the destinations once); only how they are drawn changes, and it
    /// changes together with the layout, never ahead of it. Until a layout is shown, the list
    /// keeps its own [`style`](Self::style) and [`padding`](Self::padding).
    ///
    /// ```ignore
    /// SidebarList::new(items).follow(&self.adaptive, |layout| match layout {
    ///     Layout::Mobile => (SidebarStyle::List, SidebarPadding::Lg),
    ///     Layout::Desktop => (SidebarStyle::List, SidebarPadding::Sm),
    /// })
    /// ```
    pub fn follow<K: Clone + PartialEq>(
        mut self,
        state: &crate::adaptive::Adaptive<K>,
        present: impl Fn(&K) -> (SidebarStyle, SidebarPadding),
    ) -> Self {
        if let Some(layout) = state.shown().last() {
            (self.style, self.padding) = present(layout);
            self.variant = Some(self.padding.variant_name().into());
        }
        self
    }
}

/// One item as its row (with its divider above, if it has one).
fn item_rows<'a, M: Clone + 'a>(
    i: SidebarItem<M>,
    density: &NavigationDensity,
    gap: f32,
    inset: f32,
    dragging: bool,
) -> Vec<Element<'a, M>> {
    let m = density.metrics;
    let item_density = density.for_item(i.padding);
    let line = i
        .divider_above
        .then(|| Element::from(crate::components::divider().spacing((inset - gap).max(0.0))));
    let row = sidebar_row_resolved(
        i.icon_view(m.icon_size),
        i.title,
        i.selected,
        i.on_press,
        i.opens,
        i.row_props,
        &item_density,
    );
    // A chevron to fold what is under the row, and the indent of a nested one.
    let row: Element<'a, M> = match i.expander {
        Some((open, on_toggle)) => iced::widget::row![
            container(row).width(Length::Fill),
            crate::components::button::icon_button(if open { Icon::GoDown } else { Icon::GoNext })
                .on_press(on_toggle)
        ]
        .align_y(Vertical::Center)
        .into(),
        None => row,
    };
    let row: Element<'a, M> = if i.indent > 0 {
        container(row)
            .padding(iced::Padding {
                left: item_density.number("indent", INDENT) * f32::from(i.indent),
                ..iced::Padding::ZERO
            })
            .into()
    } else {
        row
    };
    let row: Element<'a, M> = match (i.reorder, i.drop_only) {
        (Some(re), _) => {
            let on_drop = re.on_drop;
            let nest = re.inside.is_some();
            let mut zone =
                drop_zone_at(drag_source(row, re.on_start), dragging, move |p| on_drop(p))
                    .targets(if nest {
                        Targets::Edges
                    } else {
                        Targets::Between
                    })
                    .reach(gap / 2.0)
                    .edge_hint(DropHint::new(re.label));
            if let Some(inside) = re.inside {
                zone = zone.hint(DropHint::new(inside).shrink());
            }
            zone.into()
        }
        (None, Some((label, on_drop))) => drop_zone_at(row, dragging, move |p| on_drop(p))
            .hint(DropHint::new(label))
            .into(),
        (None, None) => row,
    };
    line.into_iter().chain(std::iter::once(row)).collect()
}

fn list_view<'a, M: Clone + 'a>(
    items: Vec<SidebarItem<M>>,
    density: &NavigationDensity,
    radius: f32,
    fixed_top: usize,
    dragging: bool,
    pages: Option<Pages<M>>,
    (gap_override, inset_override): (Option<f32>, Option<f32>),
) -> Element<'a, M> {
    let g = density.geometry();
    let inset = inset_override.unwrap_or(g.inset);
    let gap = gap_override.unwrap_or(g.gap);
    let rows_of = |items: Vec<SidebarItem<M>>| -> Vec<Vec<Element<'a, M>>> {
        items
            .into_iter()
            .map(|i| item_rows(i, density, gap, inset, dragging))
            .collect()
    };
    let stack = |rows: Vec<Vec<Element<'a, M>>>, padding: iced::Padding| {
        column(rows.into_iter().flatten())
            .spacing(gap)
            .padding(padding)
    };
    // Pages: the items stay, and the pages slide beneath them.
    if let Some(Pages {
        pages,
        current,
        depths,
    }) = pages
    {
        let top_rows = rows_of(items.into_iter().take(fixed_top).collect());
        let top = stack(
            top_rows,
            iced::Padding {
                top: inset,
                right: inset,
                bottom: 0.0,
                left: inset,
            },
        );
        let slides: Vec<Element<'a, M>> = pages
            .into_iter()
            .map(|page| {
                crate::layouts::scrolled_window(stack(
                    rows_of(page),
                    iced::Padding {
                        top: gap,
                        right: inset,
                        bottom: inset,
                        left: inset,
                    },
                ))
                .height(Length::Fill)
                .into()
            })
            .collect();
        let slid = crate::widgets::SlidePages::new(slides, current).depths(depths);
        return container(column![top, container(slid).height(Length::Fill)])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_: &crate::Theme| iced::widget::container::Style {
                border: iced::Border::default().rounded(iced::border::bottom(radius)),
                ..Default::default()
            })
            .into();
    }
    // Each item as its row, with its divider above if it has one.
    let rows = rows_of(items);
    if fixed_top == 0 || fixed_top >= rows.len() {
        return sidebar_surface(rows.into_iter().flatten(), radius, gap, inset);
    }
    // The first items stay; the rest scroll beneath them.
    let mut rest = rows;
    let top: Vec<_> = rest.drain(..fixed_top).collect();
    let top = stack(
        top,
        iced::Padding {
            top: inset,
            right: inset,
            bottom: 0.0,
            left: inset,
        },
    );
    let rest = stack(
        rest,
        iced::Padding {
            top: gap,
            right: inset,
            bottom: inset,
            left: inset,
        },
    );
    container(column![
        top,
        crate::layouts::scrolled_window(rest).height(Length::Fill)
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_: &crate::Theme| iced::widget::container::Style {
        border: iced::Border::default().rounded(iced::border::bottom(radius)),
        ..Default::default()
    })
    .into()
}

fn cards_view<'a, M: Clone + 'a>(
    items: Vec<SidebarItem<M>>,
    density: &NavigationDensity,
    (gap_override, inset_override): (Option<f32>, Option<f32>),
) -> Element<'a, M> {
    let m = density.metrics;
    let g = density.geometry();
    let gap = gap_override.unwrap_or(g.gap);
    let inset = inset_override.unwrap_or(g.inset);
    let rows = items.into_iter().map(|i| {
        let item_geometry = density.for_item(i.padding).geometry();
        let mut row_metrics = m;
        row_metrics.row_min_height = item_geometry.row_height;
        row_metrics.row_padding.left = item_geometry.row_padding;
        row_metrics.row_padding.right = item_geometry.row_padding;
        let icon = i.icon_view(m.icon_size);
        let mut r = crate::widgets::ActionRow::new(i.title)
            .chevron(true)
            .on_activate(i.on_press)
            .metrics(row_metrics);
        if let Some(icon) = icon {
            r = r.prefix(icon);
        }
        if let Some(s) = i.subtitle {
            r = r.subtitle(s);
        }
        // Keep a single boxed surface and its separators; the density gap sits
        // outside the row's hit target, not between independent card surfaces.
        Element::from(container(i.card_props.apply(r)).padding([gap / 2.0, 0.0]))
    });
    crate::widgets::page(crate::widgets::boxed_list(rows))
        .metrics(m)
        .padding(inset)
        .into()
}

impl<'a, M: Clone + 'a> From<SidebarList<'a, M>> for Element<'a, M> {
    fn from(l: SidebarList<'a, M>) -> Self {
        let tokens = l
            .tokens
            .clone()
            .unwrap_or_else(|| crate::theme_config::current().tokens());
        let needs_width = l.cards_below.is_some()
            || (l.variant.is_none() && tokens.text("navigation", "breakpoint-rules").is_some());
        if l.available_width.is_none() && needs_width {
            let mut l = l;
            l.tokens = Some(tokens);
            return crate::layouts::responsive(move |size| {
                Element::from(l.clone().available_width(size.width))
            })
            .into();
        }
        let density = l.density(tokens);
        let style = match (l.cards_below, l.available_width) {
            (Some(class), Some(width)) => {
                let scale = crate::breakpoints::Breakpoints::default().with_tokens(&density.tokens);
                if width < class.min_width_with(&scale) {
                    SidebarStyle::Cards
                } else {
                    SidebarStyle::List
                }
            }
            _ => l.style,
        };
        match style {
            SidebarStyle::List => list_view(
                l.items,
                &density,
                l.window_radius,
                l.fixed_top,
                l.dragging,
                l.pages,
                (l.spacing, l.inset),
            ),
            SidebarStyle::Cards => cards_view(l.items, &density, (l.spacing, l.inset)),
        }
    }
}

/// What the sidebar becomes when a [`NavigationSplitView`] collapses (narrow / mobile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CollapsedSidebar {
    /// The sidebar slides over the content from the start edge with a scrim; the
    /// content stays visible behind it and the drawer never takes the whole screen
    /// (`AdwOverlaySplitView`'s collapsed overlay). Opened/closed by the app
    /// ([`NavigationSplitView::show_sidebar`]). (default)
    #[default]
    Drawer,
    /// The sidebar is the *parent page* of a navigation stack, as in a settings app: it
    /// is the landing page, painted in the **content** color (not the sidebar color),
    /// and choosing an item pushes the content page (`show_content`) with a back button.
    /// Beside the content the sidebar keeps its own color.
    ParentPage,
}

impl CollapsedSidebar {
    pub const ALL: [Self; 2] = [Self::Drawer, Self::ParentPage];
}

impl std::fmt::Display for CollapsedSidebar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Drawer => "Drawer",
            Self::ParentPage => "Parent page",
        })
    }
}

/// `AdwNavigationSplitView`. Collapsed, it shows either a drawer over the content
/// or one pane at a time; see [`CollapsedSidebar`].
pub struct NavigationSplitView<'a, M> {
    sidebar: Element<'a, M>,
    content: Element<'a, M>,
    sidebar_width: f32,
    fraction: f32,
    min_width: f32,
    max_width: f32,
    collapsed: bool,
    show_content: bool,
    window_radius: f32,
    collapsed_sidebar: CollapsedSidebar,
    drawer_open: bool,
    on_close_drawer: Option<M>,
    min_content: f32,
    drawer_width: f32,
    sidebar_props: ContainerProps,
    content_props: ContainerProps,
}

/// libadwaita's width for a split-view sidebar: `fraction` of the total width
/// clamped to `[min, max]` (defaults 25%, 180sp, 280sp).
pub fn sidebar_width_for(total: f32, fraction: f32, min: f32, max: f32) -> f32 {
    (total * fraction).clamp(min, max)
}

/// The default width up to which split views collapse: just under
/// [`breakpoints::MD`](crate::breakpoints::MD), so a sidebar sits beside the content only on
/// tablet-landscape widths and up. (libadwaita collapses at 400sp, which leaves a phone-width
/// content area beside a sidebar; we do not.)
pub const COLLAPSE_WIDTH: f32 = crate::breakpoints::MD - 1.0;

/// The narrowest the content area may get beside a sidebar before it collapses. Content must
/// never feel squeezed: below this the sidebar becomes a drawer or the parent page, whatever
/// the window width.
pub const MIN_CONTENT_WIDTH: f32 = 520.0;

/// Fallback sidebar rules (libadwaita's: 25% of the window clamped to 180-280, and a drawer takes
/// at most 85% of it); each is configurable under `[navigation]`.
pub const SIDEBAR_FRACTION: f32 = 0.25;
pub const SIDEBAR_MIN_WIDTH: f32 = 180.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 280.0;
pub const DRAWER_MAX_FRACTION: f32 = 0.85;

/// The minimum content width in effect: the desktop config's `[navigation] min-content-width`.
pub fn min_content_width() -> f32 {
    crate::config::current().min_content_width
}

/// Whether a split view with the default sidebar rules is collapsed at this window `width`.
/// [`NavigationSplitView::adapt`] uses the same rule, so apps can branch on this for their own
/// layout (headers, back buttons) and agree with it.
pub fn split_collapsed(width: f32) -> bool {
    let c = crate::config::current();
    collapses(
        width,
        c.sidebar_fraction,
        c.sidebar_min_width,
        c.sidebar_max_width,
        c.min_content_width,
    )
}

fn collapses(width: f32, fraction: f32, min: f32, max: f32, min_content: f32) -> bool {
    width <= collapse_width() || width - sidebar_width_for(width, fraction, min, max) < min_content
}

/// The collapse width in effect: the desktop config's `[navigation] collapse-width`.
pub fn collapse_width() -> f32 {
    crate::config::current().collapse_width
}

impl<'a, M: 'a> NavigationSplitView<'a, M> {
    /// Presentation overrides applied after theme geometry and style.
    pub fn sidebar_props(mut self, props: ContainerProps) -> Self {
        self.sidebar_props = props;
        self
    }
    /// Presentation overrides applied after theme geometry and style.
    pub fn content_props(mut self, props: ContainerProps) -> Self {
        self.content_props = props;
        self
    }
    pub fn new(sidebar: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            sidebar_props: ContainerProps::default(),
            content_props: ContainerProps::default(),
            sidebar: sidebar.into(),
            content: content.into(),
            sidebar_width: navigation_geometry("initial-sidebar-width", 240.0),
            // The desktop config decides; the constants are the fallbacks.
            fraction: crate::config::current().sidebar_fraction,
            min_width: crate::config::current().sidebar_min_width,
            max_width: crate::config::current().sidebar_max_width,
            collapsed: false,
            show_content: true,
            window_radius: 0.0,
            // The desktop config decides (default: drawer); `.collapsed_sidebar(..)` is an opt-out.
            collapsed_sidebar: crate::config::current().mobile_sidebar,
            drawer_open: false,
            on_close_drawer: None,
            min_content: min_content_width(),
            drawer_width: navigation_geometry("initial-drawer-width", 280.0),
        }
    }
    /// What the sidebar becomes when collapsed: [`CollapsedSidebar::Drawer`]
    /// (default) or [`CollapsedSidebar::ParentPage`].
    pub fn collapsed_sidebar(mut self, mode: CollapsedSidebar) -> Self {
        self.collapsed_sidebar = mode;
        self
    }
    /// Whether the drawer is open (only used with [`CollapsedSidebar::Drawer`]).
    pub fn show_sidebar(mut self, open: bool) -> Self {
        self.drawer_open = open;
        self
    }
    /// Message sent when the scrim behind the open drawer is clicked.
    pub fn on_close_drawer(mut self, msg: M) -> Self {
        self.on_close_drawer = Some(msg);
        self
    }
    /// Rounds the sidebar pane's outer corners to match a rounded window
    /// ([`window::State::radius`](crate::windows::State::radius)). The pane paints
    /// the sidebar color, so without this it would cover the window's corners.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
    /// Fixed sidebar width (overrides the fraction rules).
    pub fn sidebar_width(mut self, w: f32) -> Self {
        self.sidebar_width = w;
        self
    }
    pub fn sidebar_width_fraction(mut self, f: f32) -> Self {
        self.fraction = f;
        self
    }
    pub fn min_sidebar_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }
    pub fn max_sidebar_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }
    pub fn collapsed(mut self, c: bool) -> Self {
        self.collapsed = c;
        self
    }
    pub fn show_content(mut self, s: bool) -> Self {
        self.show_content = s;
        self
    }
    /// The narrowest content area allowed beside the sidebar (default
    /// [`MIN_CONTENT_WIDTH`], or the desktop config). Below it the sidebar collapses.
    pub fn min_content_width(mut self, w: f32) -> Self {
        self.min_content = w;
        self
    }
    /// Applies the adaptive rules for a window of `width`: collapse up to
    /// [`COLLAPSE_WIDTH`] *or* when the content area would be narrower than
    /// [`min_content_width`](Self::min_content_width), and size the sidebar by
    /// fraction/min/max.
    pub fn adapt(mut self, width: f32) -> Self {
        self.collapsed = collapses(
            width,
            self.fraction,
            self.min_width,
            self.max_width,
            self.min_content,
        );
        self.sidebar_width =
            sidebar_width_for(width, self.fraction, self.min_width, self.max_width);
        // A drawer uses the maximum sidebar width when it fits, but always leaves a strip of
        // content (and the scrim) visible: never the whole screen.
        self.drawer_width = self
            .max_width
            .min(width * crate::config::current().drawer_max_fraction);
        self
    }
}

impl<'a, M: Clone + 'a> From<NavigationSplitView<'a, M>> for Element<'a, M> {
    fn from(v: NavigationSplitView<'a, M>) -> Self {
        v.build()
    }
}

impl<'a, M: Clone + 'a> NavigationSplitView<'a, M> {
    fn build(self) -> Element<'a, M> {
        let v = self;
        let r = v.window_radius;
        // The sidebar pane carries the sidebar color (`.sidebar-pane`), so a flat header
        // above the list continues it. It sits at the window edge, so it rounds the
        // window's corners: left only when beside the content or as a drawer, all four
        // when it is the whole (collapsed, parent-page) window.
        let pane = move |radius: iced::border::Radius| {
            move |t: &crate::Theme| iced::widget::container::Style {
                border: iced::Border::default().rounded(radius),
                ..sc::sidebar(t)
            }
        };
        let left = iced::border::Radius {
            top_left: r,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: r,
        };
        if v.collapsed && v.collapsed_sidebar != CollapsedSidebar::Drawer {
            // The sidebar is the parent page; the content is pushed over it (and popped back)
            // with the navigation slide.
            let first: Element<'a, M> = v
                .sidebar_props
                .apply(
                    container(v.sidebar)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .into();
            // Both pages may carry custom surface props or arbitrary transparent
            // content. Let those targets own their backing/shape instead of guessing
            // a window-shaped dimmer and opaque backing for them.
            return crate::layouts::push_stack(first, v.content, v.show_content).into();
        }
        // Beside the content or as a drawer, one widget moves the sidebar between the two on the
        // same spring: crossing the collapse breakpoint slides it out of the layout (or in),
        // displacing the content, the way a drawer slides over it.
        let sidebar: Element<'a, M> = v
            .sidebar_props
            .apply(
                container(v.sidebar)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(pane(left)),
            )
            .into();
        let content: Element<'a, M> = v
            .content_props
            .apply(
                container(crate::layouts::content_area(v.content))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .into();
        let mut split = crate::layouts::adaptive_split(
            sidebar,
            content,
            !v.collapsed,
            v.sidebar_width,
            v.drawer_width,
        )
        .drawer_open(v.drawer_open)
        .window_radius(r);
        if let Some(m) = v.on_close_drawer {
            split = split.on_close(m);
        }
        split.into()
    }
}

/// `AdwOverlaySplitView`. When `collapsed`, the sidebar floats over the
/// content with a scrim (click it to send `on_close`).
pub struct OverlaySplitView<'a, M> {
    sidebar: Element<'a, M>,
    content: Element<'a, M>,
    sidebar_width: f32,
    collapsed: bool,
    show_sidebar: bool,
    on_close: Option<M>,
    sidebar_props: ContainerProps,
    scrim_props: ContainerProps,
    separator_props: ContainerProps,
}

impl<'a, M: 'a> OverlaySplitView<'a, M> {
    pub fn sidebar_props(mut self, props: ContainerProps) -> Self {
        self.sidebar_props = props;
        self
    }
    pub fn scrim_props(mut self, props: ContainerProps) -> Self {
        self.scrim_props = props;
        self
    }
    pub fn separator_props(mut self, props: ContainerProps) -> Self {
        self.separator_props = props;
        self
    }
    pub fn new(sidebar: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            sidebar_props: ContainerProps::default(),
            scrim_props: ContainerProps::default(),
            separator_props: ContainerProps::default(),
            sidebar: sidebar.into(),
            content: content.into(),
            sidebar_width: Metrics::desktop().sidebar_width,
            collapsed: false,
            show_sidebar: true,
            on_close: None,
        }
    }
    pub fn sidebar_width(mut self, w: f32) -> Self {
        self.sidebar_width = w;
        self
    }
    pub fn collapsed(mut self, c: bool) -> Self {
        self.collapsed = c;
        self
    }
    pub fn show_sidebar(mut self, s: bool) -> Self {
        self.show_sidebar = s;
        self
    }
    pub fn on_close(mut self, m: M) -> Self {
        self.on_close = Some(m);
        self
    }
}

impl<'a, M: Clone + 'a> From<OverlaySplitView<'a, M>> for Element<'a, M> {
    fn from(v: OverlaySplitView<'a, M>) -> Self {
        // Popovers opened in the content stay inside it, not under the sidebar.
        let content: Element<'a, M> = crate::layouts::content_area(v.content).into();
        if !v.show_sidebar {
            return content;
        }
        let side = v.sidebar_props.apply(
            container(v.sidebar)
                .width(Length::Fixed(v.sidebar_width))
                .height(Length::Fill),
        );
        if !v.collapsed {
            return row![
                side,
                v.separator_props.apply(
                    container(space().width(navigation_geometry("separator-width", 1.0)))
                        .height(Length::Fill)
                        .style(sc::sidebar_separator)
                ),
                content
            ]
            .into();
        }
        let mut scrim = mouse_area(
            v.scrim_props.apply(
                container(space())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(sc::backdrop),
            ),
        );
        if let Some(m) = v.on_close {
            scrim = scrim.on_press(m);
        }
        stack![content, opaque(scrim), opaque(side)].into()
    }
}

/// One page of a navigation stack.
pub struct NavigationPage<'a, M> {
    pub title: String,
    pub content: Element<'a, M>,
}

impl<'a, M> NavigationPage<'a, M> {
    pub fn new(title: impl Into<String>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
        }
    }
}

/// `AdwNavigationView`: shows the top page with a header bar and a back
/// button (sending `on_pop`) when the stack has more than one page.
pub fn navigation_view<'a, M: Clone + 'a>(
    pages: Vec<NavigationPage<'a, M>>,
    on_pop: M,
) -> Element<'a, M> {
    navigation_view_with_props(
        pages,
        on_pop,
        super::header_bar::HeaderBarProps::default(),
        super::toolbar::ToolbarProps::default(),
        ButtonProps::default(),
    )
}

/// Navigation stack with presentation-only overrides for its header, toolbar, and back control.
pub fn navigation_view_with_props<'a, M: Clone + 'a>(
    mut pages: Vec<NavigationPage<'a, M>>,
    on_pop: M,
    header_props: super::header_bar::HeaderBarProps,
    toolbar_props: super::toolbar::ToolbarProps,
    back_button_props: ButtonProps,
) -> Element<'a, M> {
    let depth = pages.len();
    let Some(top) = pages.pop() else {
        return space().into();
    };
    let mut bar = header_bar().title(top.title);
    if depth > 1 {
        bar = bar.pack_start(
            back_button_props
                .apply(crate::components::circular_button(Icon::GoPrevious).on_press(on_pop)),
        );
    }
    toolbar_props
        .apply(toolbar_view(top.content).top_bar(header_props.apply(bar)))
        .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_paged_sidebar_builds_with_rows_that_open_pages() {
        #[derive(Clone)]
        enum M {
            Pick(u8),
        }
        let places = [
            SidebarItem::new("All", M::Pick(0)),
            SidebarItem::new("Trash", M::Pick(1)),
        ];
        let root = vec![
            SidebarItem::new("Work", M::Pick(2)).opens(true),
            SidebarItem::new("Home", M::Pick(3)),
        ];
        let work = vec![
            SidebarItem::new("Work", M::Pick(2)).icon(Icon::GoPrevious),
            SidebarItem::new("Design", M::Pick(4)),
        ];
        assert!(root[0].opens && !root[1].opens);
        let _: Element<'_, M> = SidebarList::new(places)
            .fixed_top(2)
            .pages(vec![root, work], 1, vec![0, 1])
            .into();
    }

    #[test]
    fn a_sidebar_tree_of_nested_foldable_movable_rows_builds() {
        #[derive(Clone)]
        enum M {
            Pick(u8),
            Fold(u8),
            Move(u8),
            Drop(u8, crate::primitives::drag::Place),
        }
        let items = [
            SidebarItem::new("Back", M::Pick(0)).icon(Icon::GoPrevious),
            SidebarItem::new("Work", M::Pick(1))
                .expander(true, M::Fold(1))
                .reorder(M::Move(1), "Move here", |p| M::Drop(1, p))
                .nest("Move into Work"),
            SidebarItem::new("Design", M::Pick(2))
                .indent(1)
                .expander(false, M::Fold(2))
                .reorder(M::Move(2), "Move here", |p| M::Drop(2, p))
                .nest("Move into Design"),
            SidebarItem::new("Unassigned", M::Pick(3)).drop_target("Move here", |p| M::Drop(3, p)),
        ];
        assert_eq!(items[1].indent, 0);
        assert_eq!(items[2].indent, 1);
        assert!(
            items[1]
                .reorder
                .as_ref()
                .is_some_and(|r| r.inside.is_some())
        );
        assert!(items[3].drop_only.is_some());
        let _: Element<'_, M> = SidebarList::new(items).fixed_top(1).dragging(true).into();
    }

    #[test]
    fn a_sidebar_list_can_pick_its_style_by_its_own_width() {
        let l = SidebarList::<()>::new([SidebarItem::new("One", ())])
            .cards_below(crate::breakpoints::WidthClass::Md);
        assert_eq!(l.cards_below, Some(crate::breakpoints::WidthClass::Md));
        let _: Element<'_, ()> = l.into();
    }

    #[test]
    fn a_sidebar_list_follows_the_layout_that_is_shown() {
        use crate::adaptive::{Adaptive, Layouts};
        use crate::breakpoints::{Breakpoints, WidthClass};
        use crate::layouts::transition::Budgets;
        use std::time::Instant;

        let scale = Breakpoints::default();
        let mut a = Adaptive::new(
            Layouts::new()
                .layout(..WidthClass::Md, "narrow")
                .layout(WidthClass::Md.., "wide"),
        );
        let present = |k: &&str| match *k {
            "narrow" => (SidebarStyle::Cards, SidebarPadding::Lg),
            _ => (SidebarStyle::List, SidebarPadding::Sm),
        };
        let list = || SidebarList::<()>::new([SidebarItem::new("One", ())]);

        // Nothing shown yet: the list keeps its own presentation.
        let l = list().follow(&a, present);
        assert_eq!(
            (l.style, l.padding),
            (SidebarStyle::List, SidebarPadding::Md)
        );

        let t0 = Instant::now();
        a.resize_with(&scale, Budgets::ZERO, 1000.0, t0);
        let l = list().follow(&a, present);
        assert_eq!(
            (l.style, l.padding),
            (SidebarStyle::List, SidebarPadding::Sm)
        );

        a.resize_with(&scale, Budgets::ZERO, 500.0, t0);
        let l = list().follow(&a, present);
        assert_eq!(
            (l.style, l.padding),
            (SidebarStyle::Cards, SidebarPadding::Lg)
        );
    }

    use super::*;

    #[test]
    fn sidebar_width_follows_docs_rules() {
        // 25% of the width, clamped to 180..280.
        assert_eq!(sidebar_width_for(600.0, 0.25, 180.0, 280.0), 180.0);
        assert_eq!(sidebar_width_for(1000.0, 0.25, 180.0, 280.0), 250.0);
        assert_eq!(sidebar_width_for(2000.0, 0.25, 180.0, 280.0), 280.0);
    }

    fn split(mode: CollapsedSidebar) -> NavigationSplitView<'static, ()> {
        NavigationSplitView::new(space(), space()).collapsed_sidebar(mode)
    }

    #[test]
    fn drawer_is_the_default_collapsed_mode() {
        assert_eq!(CollapsedSidebar::default(), CollapsedSidebar::Drawer);
        assert_eq!(
            split(CollapsedSidebar::Drawer).collapsed_sidebar,
            CollapsedSidebar::Drawer
        );
    }

    #[test]
    fn drawer_never_takes_the_whole_screen() {
        // At 380px the drawer uses the max sidebar width; at 300px it is capped to 85%.
        let d = split(CollapsedSidebar::Drawer).adapt(380.0);
        assert!(d.collapsed && (d.drawer_width - 280.0).abs() < 1e-3);
        let d = split(CollapsedSidebar::Drawer).adapt(300.0);
        assert!((d.drawer_width - 255.0).abs() < 1e-3 && d.drawer_width < 300.0);
        // Not collapsed: the usual fraction/min/max rule.
        let wide = split(CollapsedSidebar::Drawer).adapt(1000.0);
        assert!(!wide.collapsed && (wide.sidebar_width - 250.0).abs() < 1e-3);
    }

    #[test]
    fn content_is_never_squeezed() {
        // Whenever the sidebar sits beside the content, the content keeps its minimum width.
        for w in (360..=1800).step_by(5) {
            let v = split(CollapsedSidebar::Drawer).adapt(w as f32);
            if !v.collapsed {
                assert!(
                    w as f32 - v.sidebar_width >= MIN_CONTENT_WIDTH,
                    "squeezed at {w}"
                );
            }
        }
        // Up to just under the `md` breakpoint it collapses; from there it does not.
        assert!(split(CollapsedSidebar::Drawer).adapt(767.0).collapsed);
        assert!(!split(CollapsedSidebar::Drawer).adapt(768.0).collapsed);
        // A wider sidebar needs more room: the content rule collapses it earlier.
        let wide_sidebar = split(CollapsedSidebar::Drawer)
            .sidebar_width_fraction(0.4)
            .max_sidebar_width(400.0)
            .adapt(800.0);
        assert!(
            wide_sidebar.collapsed,
            "320px sidebar leaves 480px of content"
        );
        // And the app-facing helper agrees with `adapt` for the defaults.
        for w in [400.0, 700.0, 767.0, 768.0, 1000.0] {
            assert_eq!(
                split_collapsed(w),
                split(CollapsedSidebar::Drawer).adapt(w).collapsed
            );
        }
    }

    #[test]
    fn full_page_uses_the_regular_width_rule() {
        let f = split(CollapsedSidebar::ParentPage).adapt(300.0);
        assert!(f.collapsed && (f.sidebar_width - 180.0).abs() < 1e-3);
    }

    #[test]
    fn sidebar_padding_enums_are_only_variant_ids() {
        let tokens = crate::theme_config::Tokens::default();
        for m in [Metrics::default(), Metrics::compact()] {
            for (padding, name) in SidebarPadding::ALL.into_iter().zip(["sm", "md", "lg"]) {
                assert_eq!(padding.variant_name(), name);
                assert_eq!(
                    padding.geometry_with_tokens(&m, &tokens),
                    SidebarGeometry {
                        row_height: m.sidebar_row_height,
                        row_padding: m.sidebar_row_padding,
                        gap: m.sidebar_spacing,
                        inset: m.sidebar_padding,
                    }
                );
            }
        }
    }

    #[test]
    fn breakpoint_conditions() {
        let bp = Breakpoint::max_width(400.0);
        assert!(bp.matches(Size::new(360.0, 600.0)));
        assert!(bp.matches(Size::new(400.0, 600.0)));
        assert!(!bp.matches(Size::new(401.0, 600.0)));
        assert!(Breakpoint::min_width(500.0).matches(Size::new(500.0, 1.0)));
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_list_gap_and_inset_are_parameters() {
        let l = SidebarList::<()>::new([SidebarItem::new("a", ())])
            .spacing(6.0)
            .inset(3.0);
        assert_eq!((l.spacing, l.inset), (Some(6.0), Some(3.0)));
        let _: Element<'_, ()> = l.into();
    }
}

#[cfg(test)]
mod token_resolver_tests {
    use super::geometry;

    #[test]
    fn geometry_preserves_metric_defaults_and_accepts_zero() {
        assert_eq!(geometry(|_| None, "indent", 14.0), 14.0);
        assert_eq!(geometry(|_| Some(22.0), "indent", 14.0), 22.0);
        assert_eq!(geometry(|_| Some(0.0), "separator-width", 1.0), 0.0);
        for invalid in [-1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(geometry(|_| Some(invalid), "indent", 14.0), 14.0);
        }
    }
}

#[cfg(test)]
mod local_variant_tests {
    use super::*;
    use crate::theme_config::{Schema, ThemeConfig};

    #[test]
    fn familiar_and_custom_variants_share_per_key_inheritance() {
        let config = config(
            "[navigation]\nrow-padding=17px\ninset=0px\n[navigation.sm]\nrow-height=51px\n[navigation.md]\nrow-height=51px\n[navigation.lg]\nrow-height=51px\n[navigation.custom]\nrow-height=51px",
        );
        let mut m = Metrics::default();
        m.sidebar_spacing = 9.0;
        let expected = SidebarGeometry {
            row_height: 51.0,
            row_padding: 17.0,
            gap: 9.0,
            inset: 0.0,
        };
        for name in ["sm", "md", "lg", "custom"] {
            assert_eq!(
                navigation_geometry_with_tokens(&m, &config.tokens(), name),
                expected
            );
        }
        for padding in SidebarPadding::ALL {
            assert_eq!(padding.geometry_with_tokens(&m, &config.tokens()), expected);
        }
    }

    #[test]
    fn absent_and_opted_out_variants_are_neutral() {
        use crate::theme_config::DefaultVariants;
        let m = Metrics::default();
        for text in ["", "[navigation]\nrow-height=63px\nrow-padding=19px"] {
            for opted_out in [false, true] {
                let mut builder = ThemeConfig::builder("neutral-navigation")
                    .files(false)
                    .toolkit_text(text)
                    .app_text("");
                if opted_out {
                    builder = builder.default_variants(DefaultVariants::None);
                }
                let tokens = builder.build().tokens();
                let expected = SidebarGeometry {
                    row_height: if text.is_empty() {
                        m.sidebar_row_height
                    } else {
                        63.0
                    },
                    row_padding: if text.is_empty() {
                        m.sidebar_row_padding
                    } else {
                        19.0
                    },
                    gap: m.sidebar_spacing,
                    inset: m.sidebar_padding,
                };
                for name in ["sm", "md", "lg", "custom", "undeclared"] {
                    assert_eq!(navigation_geometry_with_tokens(&m, &tokens, name), expected);
                    if opted_out {
                        assert!(!tokens.variant("navigation", name).exists());
                    }
                }
            }
        }
    }

    #[test]
    fn libadwaita_density_defaults_are_loaded_from_config() {
        let config = config(include_str!("../../themes/libadwaita.conf"));
        let tokens = config.tokens();
        // The theme's absolute geometry is independent of unrelated inherited metrics.
        for m in [Metrics::default(), Metrics::compact()] {
            for (padding, row_height, row_padding, gap, inset) in [
                (SidebarPadding::Sm, 28.0, 8.0, 1.0, 4.0),
                (SidebarPadding::Md, 36.0, 12.0, 2.0, 6.0),
                (SidebarPadding::Lg, 44.0, 14.0, 2.0, 6.0),
            ] {
                assert_eq!(
                    padding.geometry_with_tokens(&m, &tokens),
                    SidebarGeometry {
                        row_height,
                        row_padding,
                        gap,
                        inset,
                    }
                );
            }
        }
    }

    use iced::advanced::{Renderer as _, layout, renderer::Headless, widget::Tree};
    use iced::{Point, Rectangle};

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn config(text: &str) -> ThemeConfig {
        let schema = Schema::toolkit().variant("navigation", "custom", &[]);
        assert!(
            schema.validate(text).is_valid(),
            "invalid navigation fixture: {text}"
        );
        ThemeConfig::builder("navigation-layout")
            .files(false)
            .schema(schema)
            .toolkit_text(text)
            .app_text("")
            .build()
    }

    const RULES: &str = "[breakpoints]\nsm=300px\nmd=420px\n[navigation]\nbreakpoint-rules=xs:sm,md:lg\nrow-padding=17px\ngap=5px\ninset=7px\n[navigation.sm]\nrow-height=48px\n[navigation.lg]\nrow-height=80px\nrow-padding=23px";

    fn list() -> SidebarList<'static, ()> {
        SidebarList::new([
            SidebarItem::new("First", ()),
            SidebarItem::new("Second", ()),
        ])
        .metrics(Metrics::default())
    }

    fn rectangles(
        element: &mut Element<'_, ()>,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        width: f32,
    ) -> Vec<Rectangle> {
        let node = element.as_widget_mut().layout(
            tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, 500.0)),
        );
        fn collect(layout: layout::Layout<'_>, out: &mut Vec<Rectangle>) {
            out.push(layout.bounds());
            for child in layout.children() {
                collect(child, out);
            }
        }
        let mut out = vec![];
        collect(layout::Layout::new(&node), &mut out);
        out
    }

    fn assert_rect(rects: &[Rectangle], x: f32, y: f32, width: f32, height: f32) {
        assert!(
            rects.iter().any(|r| {
                (r.x - x).abs() < 0.01
                    && (r.y - y).abs() < 0.01
                    && (r.width - width).abs() < 0.01
                    && (r.height - height).abs() < 0.01
            }),
            "missing ({x}, {y}, {width}, {height}) in {rects:?}"
        );
    }

    #[test]
    fn every_density_name_uses_the_same_actual_layout_and_font_growth() {
        let renderer = renderer();
        let configured = config(
            "[navigation]\nrow-height=48px\ngap=3px\ninset=4px\n[navigation.sm]\nrow-padding=17px\n[navigation.md]\nrow-padding=17px\n[navigation.lg]\nrow-padding=17px\n[navigation.custom]\nrow-padding=17px",
        );
        for font_size in [14.0, 80.0] {
            let mut reference = None;
            for name in ["sm", "md", "lg", "custom"] {
                let mut element: Element<'_, ()> = SidebarList::new([SidebarItem::new("Row", ())
                    .row_props(
                        SidebarRowProps::default()
                            .label_props(TextProps::default().size(font_size)),
                    )])
                .theme_config(&configured)
                .variant(name)
                .into();
                let mut tree = Tree::new(element.as_widget());
                let rects = rectangles(&mut element, &mut tree, &renderer, 360.0);
                if font_size == 14.0 {
                    assert_rect(&rects, 4.0, 4.0, 352.0, 48.0);
                } else {
                    assert!(
                        rects.iter().any(|r| r.x == 4.0
                            && r.y == 4.0
                            && r.width == 352.0
                            && r.height > 48.0)
                    );
                }
                if let Some(reference) = &reference {
                    assert_eq!(&rects, reference, "{name}");
                } else {
                    reference = Some(rects);
                }
            }
        }

        let opted_out = ThemeConfig::builder("neutral-layout")
            .files(false)
            .default_variants(crate::theme_config::DefaultVariants::None)
            .toolkit_text("")
            .app_text("")
            .build();
        for config in [config(""), opted_out] {
            let mut m = Metrics::default();
            m.sidebar_row_height = 64.0;
            for name in ["sm", "md", "lg", "custom", "undeclared"] {
                let mut element: Element<'_, ()> =
                    list().metrics(m).theme_config(&config).variant(name).into();
                let mut tree = Tree::new(element.as_widget());
                let rects = rectangles(&mut element, &mut tree, &renderer, 360.0);
                assert_rect(&rects, 4.0, 4.0, 352.0, 64.0);
            }
        }
    }

    #[test]
    fn actual_layout_reacts_at_local_thresholds_and_resizes_back() {
        let renderer = renderer();
        let config = config(RULES);
        let mut element: Element<'_, ()> = list().theme_config(&config).into();
        let mut tree = Tree::new(element.as_widget());
        for (width, height, padding) in [
            (419.0, 48.0, 17.0),
            (420.0, 80.0, 23.0),
            (419.0, 48.0, 17.0),
        ] {
            let rects = rectangles(&mut element, &mut tree, &renderer, width);
            assert_rect(&rects, 7.0, 7.0, width - 14.0, height);
            assert_rect(&rects, 7.0, 7.0 + height + 5.0, width - 14.0, height);
            assert!(rects.iter().any(|r| (r.x - 7.0 - padding).abs() < 0.01
                && (r.width - (width - 14.0 - 2.0 * padding)).abs() < 0.01));
        }
    }

    #[test]
    fn explicit_density_item_and_geometry_overrides_win_in_actual_layout() {
        let renderer = renderer();
        let config = config(RULES);
        // Last explicit density selection wins, independent of width/token builder order.
        let mut element: Element<'_, ()> = SidebarList::new([
            SidebarItem::new("First", ()),
            SidebarItem::new("Second", ()).padding(SidebarPadding::Lg),
        ])
        .available_width(900.0)
        .variant("lg")
        .padding(SidebarPadding::Sm)
        .theme_config(&config)
        .spacing(11.0)
        .inset(3.0)
        .into();
        let mut tree = Tree::new(element.as_widget());
        let rects = rectangles(&mut element, &mut tree, &renderer, 500.0);
        assert_rect(&rects, 3.0, 3.0, 494.0, 48.0);
        assert_rect(&rects, 3.0, 62.0, 494.0, 80.0);

        let mut element: Element<'_, ()> = list()
            .theme_config(&config)
            .padding(SidebarPadding::Lg)
            .variant("sm")
            .available_width(900.0)
            .into();
        let mut tree = Tree::new(element.as_widget());
        let rects = rectangles(&mut element, &mut tree, &renderer, 500.0);
        assert_rect(&rects, 7.0, 7.0, 486.0, 48.0);
    }

    #[test]
    fn actual_layout_inherits_each_key_and_keeps_local_snapshots_isolated() {
        let renderer = renderer();
        let a = config(
            "[navigation]\nrow-padding=19px\ninset=4px\n[navigation.custom]\nrow-height=60px\n[navigation.md]\ngap=99px",
        );
        let b = config("[navigation]\nrow-height=72px\nrow-padding=9px\ngap=8px\ninset=6px");
        let mut m = Metrics::default();
        m.sidebar_spacing = 3.0;
        m.sidebar_row_height = 90.0;
        let installed = crate::theme_config::current().tokens();
        for (config, height, inset, gap, padding) in [
            (&a, 60.0, 4.0, 3.0, 19.0),
            (&b, 72.0, 6.0, 8.0, 9.0),
            (&a, 60.0, 4.0, 3.0, 19.0),
        ] {
            let mut element: Element<'_, ()> = list()
                .metrics(m)
                .theme_config(config)
                .variant("custom")
                .into();
            let mut tree = Tree::new(element.as_widget());
            let rects = rectangles(&mut element, &mut tree, &renderer, 360.0);
            assert_rect(&rects, inset, inset, 360.0 - 2.0 * inset, height);
            assert_rect(
                &rects,
                inset,
                inset + height + gap,
                360.0 - 2.0 * inset,
                height,
            );
            assert!(rects.iter().any(|r| (r.x - inset - padding).abs() < 0.01
                && (r.width - (360.0 - 2.0 * inset - 2.0 * padding)).abs() < 0.01));
        }
        assert_eq!(
            crate::theme_config::current()
                .tokens()
                .number("navigation", "inset"),
            installed.number("navigation", "inset")
        );
        let mut element: Element<'_, ()> = list()
            .metrics(m)
            .theme_config(&a)
            .variant("undeclared")
            .into();
        let mut tree = Tree::new(element.as_widget());
        let rects = rectangles(&mut element, &mut tree, &renderer, 360.0);
        assert_rect(&rects, 4.0, 4.0, 352.0, 90.0);
    }

    #[test]
    fn pinned_rows_and_paged_rows_use_the_same_local_density() {
        let renderer = renderer();
        let config = config(RULES);
        for paged in [false, true] {
            let mut list = list()
                .theme_config(&config)
                .available_width(420.0)
                .cards_below(crate::breakpoints::WidthClass::Sm)
                .fixed_top(1);
            if paged {
                list = list.pages(vec![vec![SidebarItem::new("Page", ())]], 0, vec![0]);
            }
            let mut element: Element<'_, ()> = list.into();
            let mut tree = Tree::new(element.as_widget());
            let rects = rectangles(&mut element, &mut tree, &renderer, 420.0);
            assert_rect(&rects, 7.0, 7.0, 406.0, 80.0);
            assert_rect(&rects, 7.0, 92.0, 406.0, 80.0);
        }
    }

    #[test]
    fn indentation_and_typed_child_overrides_use_the_resolved_layout() {
        let renderer = renderer();
        let config = config(&format!("{RULES}\nindent=29px"));
        let mut element: Element<'_, ()> =
            SidebarList::new([SidebarItem::new("Nested", ()).indent(2)])
                .theme_config(&config)
                .available_width(420.0)
                .into();
        let mut tree = Tree::new(element.as_widget());
        let rects = rectangles(&mut element, &mut tree, &renderer, 420.0);
        assert_rect(&rects, 65.0, 7.0, 348.0, 80.0);

        for style in [SidebarStyle::List, SidebarStyle::Cards] {
            let item = SidebarItem::new("Explicit", ())
                .row_props(
                    SidebarRowProps::default()
                        .content_props(ContainerProps::default().padding([0, 31])),
                )
                .card_props(super::super::rows::ActionRowProps::default().padding([4, 31]));
            let mut element: Element<'_, ()> = SidebarList::new([item])
                .theme_config(&config)
                .available_width(420.0)
                .style(style)
                .into();
            let mut tree = Tree::new(element.as_widget());
            let rects = rectangles(&mut element, &mut tree, &renderer, 420.0);
            assert!(
                rects.iter().any(|r| r.x == 38.0 && r.width == 344.0),
                "typed padding must override variant padding for {style:?}: {rects:?}"
            );
        }
    }

    #[test]
    fn selected_sidebar_pixels_follow_custom_target_corners_in_native_states() {
        use iced::advanced::{Shell, clipboard, renderer as renderer_style};
        use iced::widget::button::{Status, Style};
        use iced::{Border, Color, Event, mouse, window};
        let mut renderer = renderer();
        let config = config("[navigation]\nselection-opacity=0.5");
        let density = NavigationDensity {
            metrics: Metrics::default(),
            tokens: std::sync::Arc::new(config.tokens()),
            variant: "md".into(),
        };
        let pill = iced::border::Radius::from(20.0);
        let square = iced::border::Radius::from(0.0);
        let asymmetric = iced::border::Radius {
            top_left: 20.0,
            top_right: 0.0,
            bottom_right: 20.0,
            bottom_left: 0.0,
        };
        let mut theme = Theme::light();
        theme.semantic.sidebar.fg = Color::BLACK;
        // Deliberately unrelated to the density metrics and custom target corners.
        theme.metrics.radius_m = 7.0;
        for shape in [0, 1, 2, 3] {
            for status in [Status::Active, Status::Hovered, Status::Pressed] {
                for selected in [false, true] {
                    let radius_for = move |status| match shape {
                        0 => pill,
                        1 => square,
                        2 => asymmetric,
                        _ => match status {
                            Status::Active => pill,
                            Status::Hovered => square,
                            _ => asymmetric,
                        },
                    };
                    let props = SidebarRowProps::default().button_props(
                        ButtonProps::default()
                            .width(80)
                            .height(40)
                            .padding(0)
                            .style(move |_, status| Style {
                                background: match status {
                                    Status::Hovered => {
                                        Some(Color::from_rgba(0.0, 0.0, 0.0, 0.2).into())
                                    }
                                    Status::Pressed => {
                                        Some(Color::from_rgba(0.0, 0.0, 0.0, 0.4).into())
                                    }
                                    _ => None,
                                },
                                text_color: Color::TRANSPARENT,
                                border: Border::default().rounded(radius_for(status)),
                                ..Default::default()
                            }),
                    );
                    let mut element = sidebar_row_resolved(
                        None,
                        String::new(),
                        selected,
                        (),
                        false,
                        props,
                        &density,
                    );
                    let mut tree = Tree::new(element.as_widget());
                    let viewport = Rectangle::with_size(Size::new(100.0, 60.0));
                    let node = element.as_widget_mut().layout(
                        &mut tree,
                        &renderer,
                        &layout::Limits::new(Size::ZERO, viewport.size()),
                    );
                    let layout = layout::Layout::new(&node);
                    assert_eq!(layout.bounds().size(), Size::new(80.0, 40.0));
                    let cursor = if status == Status::Active {
                        mouse::Cursor::Unavailable
                    } else {
                        mouse::Cursor::Available(Point::new(40.0, 20.0))
                    };
                    let mut messages = vec![];
                    if status == Status::Pressed {
                        element.as_widget_mut().update(
                            &mut tree,
                            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                            layout,
                            cursor,
                            &renderer,
                            &mut clipboard::Null,
                            &mut Shell::new(&mut messages),
                            &viewport,
                        );
                    }
                    element.as_widget_mut().update(
                        &mut tree,
                        &Event::Window(window::Event::RedrawRequested(std::time::Instant::now())),
                        layout,
                        cursor,
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &viewport,
                    );
                    renderer.reset(viewport);
                    element.as_widget().draw(
                        &tree,
                        &mut renderer,
                        &theme,
                        &renderer_style::Style {
                            text_color: Color::TRANSPARENT,
                        },
                        layout,
                        cursor,
                        &viewport,
                    );
                    let pixels = renderer.screenshot(Size::new(100, 60), 1.0, Color::WHITE);
                    let pixel =
                        |x: usize, y: usize| &pixels[(y * 100 + x) * 4..(y * 100 + x) * 4 + 4];
                    let radius = radius_for(status);
                    let has_fill = selected || status != Status::Active;
                    for (x, y, corner) in [
                        (1, 1, radius.top_left),
                        (78, 1, radius.top_right),
                        (78, 38, radius.bottom_right),
                        (1, 38, radius.bottom_left),
                    ] {
                        if corner > 0.0 || !has_fill {
                            assert_eq!(
                                pixel(x, y),
                                &[255, 255, 255, 255],
                                "outside target: shape={shape}, {status:?}, selected={selected}"
                            );
                        } else {
                            assert!(pixel(x, y)[0] < 255, "square target corner lost its fill");
                        }
                    }
                    let expected_alpha = match status {
                        Status::Hovered => 0.2 + if selected { 0.5 * 0.8 } else { 0.0 },
                        Status::Pressed => 0.4 + if selected { 0.5 * 0.6 } else { 0.0 },
                        _ => {
                            if selected {
                                0.5
                            } else {
                                0.0
                            }
                        }
                    };
                    let expected = ((1.0_f32 - expected_alpha) * 255.0).round() as i16;
                    assert!(
                        (i16::from(pixel(40, 20)[0]) - expected).abs() <= 1,
                        "selection and native state fill must compose: {shape}, {status:?}, {selected}"
                    );
                    if status == Status::Pressed {
                        element.as_widget_mut().update(
                            &mut tree,
                            &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                            layout,
                            cursor,
                            &renderer,
                            &mut clipboard::Null,
                            &mut Shell::new(&mut messages),
                            &viewport,
                        );
                        assert_eq!(messages, vec![()]);
                    }
                    assert_eq!(pixel(90, 20), &[255, 255, 255, 255]);
                }
            }
        }
    }

    #[test]
    fn sidebar_selection_uses_the_draw_theme_not_view_build_metrics() {
        use iced::advanced::{Shell, clipboard, renderer as renderer_style};
        use iced::{Color, Event, mouse, window};
        let mut renderer = renderer();
        let config = config("[navigation]\nselection-opacity=0.5");
        let density = NavigationDensity {
            metrics: Metrics::default(),
            tokens: std::sync::Arc::new(config.tokens()),
            variant: "md".into(),
        };
        let props =
            SidebarRowProps::default().button_props(ButtonProps::default().width(80).height(40));
        let mut element =
            sidebar_row_resolved(None, String::new(), true, (), false, props, &density);
        let mut tree = Tree::new(element.as_widget());
        let viewport = Rectangle::with_size(Size::new(100.0, 60.0));
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let layout = layout::Layout::new(&node);
        element.as_widget_mut().update(
            &mut tree,
            &Event::Window(window::Event::RedrawRequested(std::time::Instant::now())),
            layout,
            mouse::Cursor::Unavailable,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut vec![]),
            &viewport,
        );
        for radius in [0.0, 20.0, 0.0] {
            let mut theme = Theme::light();
            theme.metrics.radius_m = radius;
            theme.semantic.sidebar.fg = Color::BLACK;
            renderer.reset(viewport);
            element.as_widget().draw(
                &tree,
                &mut renderer,
                &theme,
                &renderer_style::Style {
                    text_color: Color::TRANSPARENT,
                },
                layout,
                mouse::Cursor::Unavailable,
                &viewport,
            );
            let pixels = renderer.screenshot(Size::new(100, 60), 1.0, Color::WHITE);
            assert_eq!(pixels[(100 + 1) * 4] == 255, radius > 0.0);
            assert!(pixels[(20 * 100 + 40) * 4] < 255);
        }
    }

    #[test]
    fn local_density_cards_keep_subtitles_and_activation() {
        use iced::advanced::{Shell, clipboard, renderer as renderer_style};
        use iced::{Event, mouse};
        let mut renderer = renderer();
        let config = config(RULES);
        let mut element: Element<'_, ()> = SidebarList::new([
            SidebarItem::new("First", ()).subtitle("Description"),
            SidebarItem::new("Second", ()),
        ])
        .metrics(Metrics::default())
        .theme_config(&config)
        .cards_below(crate::breakpoints::WidthClass::Md)
        .into();
        let mut tree = Tree::new(element.as_widget());
        let rects = rectangles(&mut element, &mut tree, &renderer, 419.0);
        // Card row retains its subtitle and minimum height inside one boxed surface.
        assert!(
            rects
                .iter()
                .any(|r| r.x == 7.0 && r.y == 9.5 && r.width == 405.0 && r.height >= 48.0)
        );
        assert!(
            rects
                .iter()
                .any(|r| r.x == 7.0 && r.y > 9.5 && r.width == 405.0 && r.height == 48.0)
        );
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(419.0, 500.0)),
        );
        let layout = layout::Layout::new(&node);
        let viewport = Rectangle::with_size(Size::new(419.0, 500.0));
        // Resolve the Card's actual themed surface before targeting pointer input.
        renderer.reset(viewport);
        element.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::light(),
            &renderer_style::Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );
        let mut messages = vec![];
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            element.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                layout,
                mouse::Cursor::Available(Point::new(40.0, 30.0)),
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &viewport,
            );
        }
        assert_eq!(messages, vec![()]);
        // At the same local structural threshold the same descriptions become list rows.
        let rects = rectangles(&mut element, &mut tree, &renderer, 420.0);
        assert_rect(&rects, 7.0, 7.0, 406.0, 80.0);
    }
}
