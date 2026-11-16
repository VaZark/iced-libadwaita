//! `AdwHeaderBar` and `AdwWindowTitle`.

use crate::Element;
use crate::components::typography;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::styles::container as sc;
use crate::{DecorationLayout, Icon, IconSource, Metrics, Theme, WindowButton};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{column, container, mouse_area, row, space, text};
use iced::{Length, Padding};

/// Window controls for a [`HeaderBar`].
///
/// Which buttons appear, and on which side, follows the user's GNOME
/// `button-layout` (`DecorationLayout::detect`, GNOME's default is a lone close
/// button at the end). A button is shown only if the layout includes it *and* you
/// provide its message. The icons use bundled SVGs; override them with `HeaderBar::window_control_icons`.
///
/// In mobile mode ([`breakpoints::is_mobile`](crate::breakpoints::is_mobile)) don't add
/// controls at all.
pub struct WindowControls<M> {
    pub on_minimize: Option<M>,
    pub on_maximize: Option<M>,
    pub on_close: Option<M>,
    /// Shows the restore icon on the maximize button.
    pub maximized: bool,
    /// Overrides the detected layout (e.g. for tests or non-GNOME setups).
    pub layout: Option<DecorationLayout>,
}

impl<M> Default for WindowControls<M> {
    fn default() -> Self {
        Self {
            on_minimize: None,
            on_maximize: None,
            on_close: None,
            maximized: false,
            layout: None,
        }
    }
}

/// Artwork overrides for a header bar's window buttons.
/// Each source accepts a bundled icon, custom SVG, or named system icon.
#[derive(Debug, Clone)]
pub struct WindowControlIcons {
    pub close: IconSource,
    pub minimize: IconSource,
    pub maximize: IconSource,
    pub restore: IconSource,
}
impl Default for WindowControlIcons {
    fn default() -> Self {
        Self {
            close: Icon::WindowClose.into(),
            minimize: Icon::WindowMinimize.into(),
            maximize: Icon::WindowMaximize.into(),
            restore: Icon::WindowRestore.into(),
        }
    }
}

/// `AdwWindowTitle`: centered title with optional subtitle.
pub fn window_title<'a, M: 'a>(title: &str, subtitle: Option<&str>) -> Element<'a, M> {
    window_title_with_props(
        title,
        subtitle,
        &TextProps::default(),
        &TextProps::default(),
    )
}

/// Window title with typed text presentation overrides.
pub fn window_title_with_props<'a, M: 'a>(
    title: &str,
    subtitle: Option<&str>,
    title_props: &TextProps,
    subtitle_props: &TextProps,
) -> Element<'a, M> {
    // Bold unless the theme's `[window-decorations] titlebar-font-weight` says normal.
    let name = if crate::theme_config::titlebar_bold() {
        typography::heading(title.to_string())
    } else {
        typography::body(title.to_string()).wrapping(text::Wrapping::None)
    };
    let mut c = column![title_props.apply(name)].align_x(Horizontal::Center);
    if let Some(s) = subtitle {
        c = c.push(
            subtitle_props.apply(
                typography::caption(s.to_string())
                    .wrapping(text::Wrapping::None)
                    .style(crate::styles::text::dim),
            ),
        );
    }
    c.into()
}

/// `AdwHeaderBar`.
pub struct HeaderBar<'a, M> {
    start: Vec<Element<'a, M>>,
    end: Vec<Element<'a, M>>,
    title: Option<String>,
    subtitle: Option<String>,
    title_widget: Option<Element<'a, M>>,
    controls: Option<WindowControls<M>>,
    control_icons: WindowControlIcons,
    show_start_buttons: bool,
    show_end_buttons: bool,
    on_drag: Option<M>,
    on_double_click: Option<M>,
    on_middle_click: Option<M>,
    on_right_click: Option<M>,
    corners: (f32, f32),
    flat: Option<bool>,
    spacing: Option<f32>,
    padding: Option<Padding>,
    height: Option<f32>,
    title_gap: Option<f32>,
    metrics: Metrics,
    title_props: TextProps,
    subtitle_props: TextProps,
    controls_props: ButtonProps,
    container_props: ContainerProps,
}

/// Builds an empty [`HeaderBar`].
pub fn header_bar<'a, M>() -> HeaderBar<'a, M> {
    HeaderBar::new()
}

impl<'a, M> HeaderBar<'a, M> {
    /// Owned presentation overrides for the concrete child.
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = props;
        self
    }
    /// Owned presentation overrides for the concrete child.
    pub fn subtitle_props(mut self, props: TextProps) -> Self {
        self.subtitle_props = props;
        self
    }
    /// Owned presentation overrides for the concrete child.
    pub fn controls_props(mut self, props: ButtonProps) -> Self {
        self.controls_props = props;
        self
    }
    /// Owned presentation overrides for the concrete child.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }
    pub fn new() -> Self {
        Self {
            title_props: TextProps::default(),
            subtitle_props: TextProps::default(),
            controls_props: ButtonProps::default(),
            container_props: ContainerProps::default(),
            start: Vec::new(),
            end: Vec::new(),
            title: None,
            subtitle: None,
            title_widget: None,
            controls: None,
            control_icons: WindowControlIcons::default(),
            show_start_buttons: true,
            show_end_buttons: true,
            on_drag: None,
            on_double_click: None,
            on_middle_click: None,
            on_right_click: None,
            corners: (0.0, 0.0),
            flat: None,
            spacing: None,
            padding: None,
            height: None,
            title_gap: None,
            metrics: Metrics::desktop(),
        }
    }
    /// `pack_start`.
    pub fn pack_start(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.start.push(e.into());
        self
    }
    /// `pack_end` (added right-to-left like GTK: first call is rightmost).
    pub fn pack_end(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.end.insert(0, e.into());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn subtitle(mut self, t: impl Into<String>) -> Self {
        self.subtitle = Some(t.into());
        self
    }
    /// `title_widget`: replaces the title (e.g. a [`ToggleGroup`](crate::components::ToggleGroup)).
    pub fn title_widget(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.title_widget = Some(e.into());
        self
    }
    /// `show-start-title-buttons`: window buttons the layout puts at the start.
    /// In split views only the pane at that window edge shows them.
    pub fn show_start_title_buttons(mut self, show: bool) -> Self {
        self.show_start_buttons = show;
        self
    }
    /// `show-end-title-buttons`: window buttons the layout puts at the end.
    pub fn show_end_title_buttons(mut self, show: bool) -> Self {
        self.show_end_buttons = show;
        self
    }
    pub fn window_controls(mut self, c: WindowControls<M>) -> Self {
        self.controls = Some(c);
        self
    }
    /// Message sent on press of the empty header area (use `window::drag`).
    pub fn on_drag(mut self, m: M) -> Self {
        self.on_drag = Some(m);
        self
    }
    /// Replaces window-control artwork without changing layout or actions.
    pub fn window_control_icons(mut self, icons: WindowControlIcons) -> Self {
        self.control_icons = icons;
        self
    }
    /// Message for double-click (use `window::toggle_maximize`).
    pub fn on_double_click(mut self, m: M) -> Self {
        self.on_double_click = Some(m);
        self
    }
    /// Message for a middle click on the bar.
    pub fn on_middle_click(mut self, m: M) -> Self {
        self.on_middle_click = Some(m);
        self
    }
    /// Message for a right click on the bar.
    pub fn on_right_click(mut self, m: M) -> Self {
        self.on_right_click = Some(m);
        self
    }
    /// Rounds the bar's top corners to match the window corners (pass
    /// [`window::State::radius`](crate::windows::State::radius); 0 when maximized).
    pub fn window_radius(mut self, r: f32) -> Self {
        self.corners = (r, r);
        self
    }
    /// Flat header (`%headerbar-flat`: no background, no shadow): the bar shows
    /// whatever is behind it, so it flows into the content. Used in dialogs
    /// (Nautilus-style) and inline contexts.
    ///
    /// Without a call to this, the bar follows [`Theme::flat_headerbars`](crate::Theme::flat_headerbars).
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = Some(flat);
        self
    }
    /// Rounds only the outer corners: `left` and `right` radii. Use in split views
    /// where each pane has its own header (left pane: `(r, 0)`, right pane: `(0, r)`).
    /// Makes this header bar part of the window's client-side decorations: the window buttons
    /// the user's layout puts on the edges `chrome` says it touches, the drag area (press to
    /// move, double-click to maximize), and the window's rounded corners at those edges. Every
    /// press is sent through `on_action`; perform it with
    /// [`Action::perform`](crate::windows::Action::perform). Without client-side decorations it
    /// only clears the window buttons.
    pub fn chrome(
        self,
        chrome: &crate::windows::Chrome,
        on_action: impl Fn(crate::windows::Action) -> M,
    ) -> Self {
        use crate::windows::Action;
        let r = chrome.radius();
        let bar = self.window_corners(
            if chrome.left { r } else { 0.0 },
            if chrome.right { r } else { 0.0 },
        );
        if !chrome.csd {
            return bar
                .show_start_title_buttons(false)
                .show_end_title_buttons(false);
        }
        let mut bar = bar
            .show_start_title_buttons(chrome.left)
            .show_end_title_buttons(chrome.right)
            .window_controls(WindowControls {
                on_minimize: chrome.minimize.then(|| on_action(Action::Minimize)),
                on_maximize: chrome.maximize.then(|| on_action(Action::ToggleMaximize)),
                on_close: Some(on_action(Action::Close)),
                maximized: chrome.state.maximized,
                layout: None,
            })
            .on_drag(on_action(Action::Drag));
        // What a click on the bar does is the theme's `[window-decorations] titlebar-*-click`.
        let act = |click: &str| Action::titlebar(click, chrome).map(&on_action);
        if let Some(m) = act("double") {
            bar = bar.on_double_click(m);
        }
        if let Some(m) = act("middle") {
            bar = bar.on_middle_click(m);
        }
        if let Some(m) = act("right") {
            bar = bar.on_right_click(m);
        }
        bar
    }

    /// The gap between the items packed at each side (default: the theme's `space_s`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The padding around the bar's content (default: the theme's `headerbar_padding`).
    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The height of the bar (default: the theme's `headerbar_height`).
    pub fn height(mut self, px: f32) -> Self {
        self.height = Some(px.max(0.0));
        self
    }
    /// The least gap between the title and the items at either side (default: the theme's
    /// `space_m`).
    pub fn title_gap(mut self, px: f32) -> Self {
        self.title_gap = Some(px.max(0.0));
        self
    }
    pub fn window_corners(mut self, left: f32, right: f32) -> Self {
        self.corners = (left, right);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> Default for HeaderBar<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

/// Overlay only explicitly configured action colors on the underlying button style.
fn control_style(
    mut style: iced::widget::button::Style,
    theme: &Theme,
    status: iced::widget::button::Status,
    action: &str,
) -> iced::widget::button::Style {
    use iced::widget::button::Status;
    let color = |part| crate::theme_config::window_control_color(&theme.tokens, action, part);
    let disabled = |c| crate::color::fade(c, theme.semantic.opacity.disabled);
    if let Some(bg) = color("bg") {
        let bg = match status {
            Status::Hovered => theme.semantic.hover_fill(bg),
            Status::Pressed => theme.semantic.active_fill(bg),
            Status::Disabled => disabled(bg),
            Status::Active => bg,
        };
        style.background = Some(iced::Background::Color(bg));
    }
    if let Some(fg) = color("fg") {
        style.text_color = if status == Status::Disabled {
            disabled(fg)
        } else {
            fg
        };
    }
    if let Some(border) = color("border") {
        style.border.color = if status == Status::Disabled {
            disabled(border)
        } else {
            border
        };
        style.border.width = style.border.width.max(1.0);
    }
    style
}

/// How far from the window's side edge a button whose top edge is `above` below the top must
/// sit so that its outer corner is at least `clearance` inside the arc of a window corner of
/// `radius`. The corner point `(inset, above)` has to be within `radius - clearance` of the arc's
/// center `(radius, radius)`. No rounded corner, or a button below the curve: nothing is needed.
pub(crate) fn corner_inset(radius: f32, above: f32, clearance: f32) -> f32 {
    if radius <= 0.0 || above >= radius {
        return 0.0;
    }
    let reach = radius - clearance.max(0.0);
    let down = radius - above;
    if reach <= down {
        // No sideways position keeps that clearance at this height: as far as the arc goes.
        return radius;
    }
    radius - (reach * reach - down * down).sqrt()
}

/// A window button with the chosen artwork, sized and rounded by the theme's `[window-decorations]` tokens.
fn ctl<'a, M: Clone + 'a>(
    icon: IconSource,
    kind: &'static str,
    msg: M,
    flat: Option<bool>,
    props: &ButtonProps,
) -> Element<'a, M> {
    let size = crate::theme_config::window_button_size();
    let radius = crate::theme_config::window_button_radius();
    let icon_size = crate::theme_config::window_button_icon_size();
    let filled = crate::theme_config::window_buttons_filled();
    // A theme that names a button variant for the window buttons (`[window-decorations] button-style`) styles
    // them as that variant, bevel and gradient included; its radius and size are the variant's.
    if let Some(name) = crate::theme_config::window_button_style() {
        let section = format!("button.{name}");
        let face: Element<'a, M> = if crate::theme_config::classic_window_glyphs()
            && let Some(glyph) = crate::primitives::pixmap::window_glyph(kind)
        {
            // Classic buttons: the theme gives the outer size, the face is what the bevel leaves,
            // and the glyph is a pixel pattern in the button's text color.
            let (w, h) = crate::theme_config::window_button_dims();
            let rings = crate::components::bevel_of(&section).map_or(0.0, |(_, width)| {
                crate::primitives::bevel::rings(width) as f32
            });
            container(crate::components::Pixels::new(glyph, section.clone()))
                .center_x(Length::Fixed((w - 2.0 * rings).max(1.0)))
                .center_y(Length::Fixed((h - 2.0 * rings).max(1.0)))
                .into()
        } else {
            container(icon.view(icon_size))
                .center(Length::Fixed(size))
                .into()
        };
        let content: Element<'a, M> = match crate::components::bevel_of(&section) {
            Some((style, width)) => crate::components::Bevel::new(face, section, style, width)
                .pressable(true)
                .into(),
            None => face.into(),
        };
        let variant = crate::styles::button::variant(&name);
        return props
            .apply(
                iced::widget::button(content)
                    .padding(0)
                    .style(move |theme, status| {
                        control_style(variant(theme, status), theme, status, kind)
                    })
                    .on_press(msg),
            )
            .into();
    }
    props
        .apply(
            iced::widget::button(container(icon.view(icon_size)).center(Length::Fixed(size)))
                .padding(0)
                .style(move |t: &Theme, status| {
                    // The bare glyph in the title bar's foreground, with a fill only under the
                    // pointer; a theme asks for a disc at rest with `[window-decorations] button-fill=circle`.
                    let mut style = if filled && flat.unwrap_or(t.flat_headerbars) {
                        crate::styles::button::circular(t, status)
                    } else {
                        crate::styles::button::headerbar(t, status)
                    };
                    style.border.radius = radius.into();
                    control_style(style, t, status, kind)
                })
                .on_press(msg),
        )
        .into()
}

impl<'a, M: Clone + 'a> From<HeaderBar<'a, M>> for Element<'a, M> {
    fn from(h: HeaderBar<'a, M>) -> Self {
        let m = h.metrics;
        let (corner_l, corner_r) = h.corners;
        let flat = h.flat;
        let mut start = h.start;
        let mut end = h.end;
        let is_titlebar = h.controls.is_some();
        if let Some(c) = h.controls {
            let layout = c.layout.clone().unwrap_or_else(DecorationLayout::detect);
            let make = |b: WindowButton| -> Option<Element<'a, M>> {
                match b {
                    WindowButton::Minimize => c.on_minimize.clone().map(|m| {
                        ctl(
                            h.control_icons.minimize.clone(),
                            "minimize",
                            m,
                            flat,
                            &h.controls_props,
                        )
                    }),
                    WindowButton::Maximize => c.on_maximize.clone().map(|m| {
                        ctl(
                            if c.maximized {
                                h.control_icons.restore.clone()
                            } else {
                                h.control_icons.maximize.clone()
                            },
                            if c.maximized { "restore" } else { "maximize" },
                            m,
                            flat,
                            &h.controls_props,
                        )
                    }),
                    WindowButton::Close => c.on_close.clone().map(|m| {
                        ctl(
                            h.control_icons.close.clone(),
                            "close",
                            m,
                            flat,
                            &h.controls_props,
                        )
                    }),
                }
            };
            // The window buttons sit together, `[window-decorations] button-gap` apart.
            let gap = crate::theme_config::window_button_gap();
            let close_gap = crate::theme_config::window_close_gap();
            let group = |buttons: Vec<(WindowButton, Element<'a, M>)>| -> Option<Element<'a, M>> {
                if buttons.is_empty() {
                    return None;
                }
                let mut r = row![].spacing(0.0).align_y(Vertical::Center);
                for (i, (which, el)) in buttons.into_iter().enumerate() {
                    if i > 0 {
                        let g = if which == WindowButton::Close {
                            close_gap.unwrap_or(gap)
                        } else {
                            gap
                        };
                        r = r.push(space().width(g));
                    }
                    r = r.push(el);
                }
                Some(r.into())
            };
            let made = |list: &[WindowButton]| -> Vec<(WindowButton, Element<'a, M>)> {
                list.iter()
                    .copied()
                    .filter_map(|b| make(b).map(|el| (b, el)))
                    .collect()
            };
            // Clear of the title bar's edge, so the buttons keep their distance from a rounded
            // window corner (`[window-decorations] button-inset`).
            let inset = crate::theme_config::window_button_inset();
            // The buttons are centered in the bar: this far below its top edge.
            let pad = h.padding.unwrap_or(m.headerbar_padding);
            let (_, button_height) = crate::theme_config::window_button_dims();
            let above = pad.top + (h.height.unwrap_or(m.headerbar_height) - button_height) / 2.0;
            let clearance = crate::theme_config::window_button_corner_clearance().unwrap_or(above / 2.0);
            // Near a rounded window corner the side inset grows so the button's corner keeps
            // `clearance` from the arc, however large the radius is.
            let inset_at = |side_pad: f32, radius: f32| {
                inset.max(corner_inset(radius, above, clearance) - side_pad)
            };
            let (inset_l, inset_r) = (inset_at(pad.left, corner_l), inset_at(pad.right, corner_r));
            // And apart from the title bar's own actions (`[window-decorations] button-separation`).
            let apart = crate::theme_config::window_button_separation();
            if h.show_start_buttons {
                start.splice(
                    0..0,
                    group(made(&layout.start)).map(|g| {
                        Element::from(
                            container(g).padding(iced::Padding::ZERO.left(inset_l).right(apart)),
                        )
                    }),
                );
            }
            if h.show_end_buttons {
                end.extend(group(made(&layout.end)).map(|g| {
                    Element::from(
                        container(g).padding(iced::Padding::ZERO.right(inset_r).left(apart)),
                    )
                }));
            }
        }
        let mut title: Element<'a, M> = match (h.title_widget, h.title) {
            (Some(w), _) => w,
            (None, Some(t)) => window_title_with_props(
                &t,
                h.subtitle.as_deref(),
                &h.title_props,
                &h.subtitle_props,
            ),
            (None, None) => space().into(),
        };
        // The theme's title bar: the application icon, and where the title sits.
        if is_titlebar && crate::theme_config::titlebar_icon() {
            title = row![crate::Icon::Window.view(m.icon_size), title]
                .spacing(m.space_xs)
                .align_y(Vertical::Center)
                .into();
        }
        let title = match crate::theme_config::titlebar_align() {
            "start" if is_titlebar => {
                start.push(title);
                Element::from(space())
            }
            "end" if is_titlebar => {
                end.insert(0, title);
                Element::from(space())
            }
            _ => title,
        };
        let side = |items: Vec<Element<'a, M>>| -> Element<'a, M> {
            row(items)
                .spacing(h.spacing.unwrap_or(m.space_s))
                .align_y(Vertical::Center)
                .into()
        };
        let bar = container(crate::layouts::slots::slots(
            side(start),
            title,
            side(end),
            h.height.unwrap_or(m.headerbar_height),
            h.title_gap.unwrap_or(m.space_m),
        ))
        .padding(h.padding.unwrap_or(m.headerbar_padding))
        .width(Length::Fill)
        .style(move |t: &Theme| {
            if flat.unwrap_or(t.flat_headerbars) {
                // Text color only; the background comes from what is behind the bar.
                return iced::widget::container::Style {
                    text_color: Some(t.semantic.headerbar.fg),
                    ..Default::default()
                };
            }
            let mut style = iced::widget::container::Style {
                border: iced::Border::default()
                    .rounded(iced::border::top_left(corner_l).top_right(corner_r)),
                ..sc::headerbar(t)
            };
            // The title bar of a window has its own gradient over the header bar's.
            if is_titlebar && let Some(g) = t.gradient("titlebar") {
                style.background = Some(g.background());
            }
            style
        });

        let mut area = mouse_area(h.container_props.apply(bar));
        if let Some(d) = h.on_drag {
            area = area.on_press(d);
        }
        if let Some(d) = h.on_double_click {
            area = area.on_double_click(d);
        }
        if let Some(d) = h.on_middle_click {
            area = area.on_middle_press(d);
        }
        if let Some(d) = h.on_right_click {
            area = area.on_right_press(d);
        }
        area.into()
    }
}

#[allow(dead_code)]
fn _assert_types(_: Padding, _: &Theme, _: text::Text<'static, Theme>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_colors_overlay_both_control_styles_and_derive_states() {
        use iced::widget::button::Status;
        let cfg = crate::theme_config::ThemeConfig::builder("test")
            .files(false)
            .toolkit_text(r##"{
    "window-decorations": {
        "close-bg": "#ff0000",
        "close-fg": "#ffffff",
        "close-border": "#000000",
    },
}
"##)
            .build();
        let theme = Theme::light().with_theme_config(&cfg);
        for base in [
            crate::styles::button::headerbar(&theme, Status::Active),
            crate::styles::button::variant("default")(&theme, Status::Active),
        ] {
            let unchanged = control_style(base, &theme, Status::Active, "minimize");
            assert_eq!(unchanged.background, base.background);
            assert_eq!(unchanged.text_color, base.text_color);
            assert_eq!(unchanged.border, base.border);
            for status in [
                Status::Active,
                Status::Hovered,
                Status::Pressed,
                Status::Disabled,
            ] {
                let styled = control_style(base, &theme, status, "close");
                let red = iced::Color::from_rgb(1.0, 0.0, 0.0);
                let bg = match status {
                    Status::Active => red,
                    Status::Hovered => theme.semantic.hover_fill(red),
                    Status::Pressed => theme.semantic.active_fill(red),
                    Status::Disabled => crate::color::fade(red, theme.semantic.opacity.disabled),
                };
                assert_eq!(styled.background, Some(iced::Background::Color(bg)));
                assert_eq!(styled.border.width, base.border.width.max(1.0));
                assert_eq!(styled.border.radius, base.border.radius);
                let fg = if status == Status::Disabled {
                    crate::color::fade(iced::Color::WHITE, theme.semantic.opacity.disabled)
                } else {
                    iced::Color::WHITE
                };
                assert_eq!(styled.text_color, fg);
            }
        }
    }

    #[test]
    fn a_header_bar_takes_spacing_padding_and_height() {
        let bar = HeaderBar::<u8>::new()
            .title("T")
            .spacing(2.0)
            .padding([1.0, 3.0])
            .height(40.0)
            .title_gap(5.0);
        assert_eq!(
            (bar.spacing, bar.height, bar.title_gap),
            (Some(2.0), Some(40.0), Some(5.0))
        );
        let _: Element<'_, u8> = bar.into();
    }
    use crate::windows::{Action, Chrome, State};

    #[test]
    fn chrome_puts_buttons_and_corners_on_the_edges_it_touches() {
        let r = Metrics::default().radius_window;
        let chrome = Chrome::new(State::default())
            .csd_override(true)
            .edges(false, true);
        let bar = HeaderBar::<Action>::new().chrome(&chrome, |a| a);
        assert!(!bar.show_start_buttons && bar.show_end_buttons);
        assert_eq!(bar.corners, (0.0, r));
        assert!(bar.on_drag.is_some() && bar.on_double_click.is_some());
        assert!(bar.controls.is_some());
    }

    #[test]
    fn a_dialog_chrome_has_no_double_click_to_maximize() {
        let chrome = Chrome::new(State::default())
            .csd_override(true)
            .close_only();
        let bar = HeaderBar::<Action>::new().chrome(&chrome, |a| a);
        assert!(bar.on_double_click.is_none());
        let c = bar.controls.unwrap();
        assert!(c.on_close.is_some() && c.on_minimize.is_none() && c.on_maximize.is_none());
    }

    #[test]
    fn without_client_side_decorations_the_header_draws_no_window_chrome() {
        let chrome = Chrome::new(State::default()).csd_override(false);
        let bar = HeaderBar::<Action>::new().chrome(&chrome, |a| a);
        assert!(!bar.show_start_buttons && !bar.show_end_buttons);
        assert_eq!(bar.corners, (0.0, 0.0));
        assert!(bar.on_drag.is_none() && bar.controls.is_none());
    }
}

/// Owned presentation overrides for a nested `HeaderBar`; values and events stay parent-owned.
#[derive(Clone, Default)]
pub struct HeaderBarProps {
    pub title_props: Option<TextProps>,
    pub subtitle_props: Option<TextProps>,
    pub controls_props: Option<ButtonProps>,
    pub container_props: Option<ContainerProps>,
    pub spacing: Option<f32>,
    pub padding: Option<Padding>,
    pub height: Option<f32>,
    pub title_gap: Option<f32>,
    pub flat: Option<bool>,
    pub metrics: Option<Metrics>,
}
impl HeaderBarProps {
    pub fn title_props(mut self, value: impl Into<TextProps>) -> Self {
        self.title_props = Some(value.into());
        self
    }
    pub fn subtitle_props(mut self, value: impl Into<TextProps>) -> Self {
        self.subtitle_props = Some(value.into());
        self
    }
    pub fn controls_props(mut self, value: impl Into<ButtonProps>) -> Self {
        self.controls_props = Some(value.into());
        self
    }
    pub fn container_props(mut self, value: impl Into<ContainerProps>) -> Self {
        self.container_props = Some(value.into());
        self
    }
    pub fn spacing(mut self, value: impl Into<f32>) -> Self {
        self.spacing = Some(value.into());
        self
    }
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.padding = Some(value.into());
        self
    }
    pub fn height(mut self, value: impl Into<f32>) -> Self {
        self.height = Some(value.into());
        self
    }
    pub fn title_gap(mut self, value: impl Into<f32>) -> Self {
        self.title_gap = Some(value.into());
        self
    }
    pub fn flat(mut self, value: impl Into<bool>) -> Self {
        self.flat = Some(value.into());
        self
    }
    pub fn metrics(mut self, value: impl Into<Metrics>) -> Self {
        self.metrics = Some(value.into());
        self
    }
    pub fn apply<'a, M: 'a>(&self, widget: HeaderBar<'a, M>) -> HeaderBar<'a, M> {
        let mut widget = widget;
        if let Some(props) = &self.title_props {
            widget.title_props.merge(props);
        }
        if let Some(props) = &self.subtitle_props {
            widget.subtitle_props.merge(props);
        }
        if let Some(props) = &self.controls_props {
            widget.controls_props.merge(props);
        }
        if let Some(props) = &self.container_props {
            widget.container_props.merge(props);
        }
        if let Some(value) = &self.spacing {
            widget = widget.spacing(value.clone());
        }
        if let Some(value) = &self.padding {
            widget = widget.padding(value.clone());
        }
        if let Some(value) = &self.height {
            widget = widget.height(value.clone());
        }
        if let Some(value) = &self.title_gap {
            widget = widget.title_gap(value.clone());
        }
        if let Some(value) = &self.flat {
            widget = widget.flat(value.clone());
        }
        if let Some(value) = &self.metrics {
            widget = widget.metrics(value.clone());
        }
        widget
    }
}

#[cfg(test)]
mod child_props_tests {
    use super::*;
    #[test]
    fn reusable_props_forward_geometry_without_replacing_actions() {
        let bar = HeaderBar::<u8>::new()
            .title("Title")
            .on_drag(7)
            .window_controls(WindowControls {
                on_close: Some(8),
                ..Default::default()
            });
        let bar = HeaderBarProps::default()
            .spacing(3.0)
            .height(41.0)
            .title_props(TextProps::default().size(21))
            .controls_props(ButtonProps::default().width(35))
            .apply(bar);
        assert_eq!(bar.spacing, Some(3.0));
        assert_eq!(bar.height, Some(41.0));
        assert_eq!(bar.title_props.size, Some(iced::Pixels(21.0)));
        assert_eq!(bar.controls_props.width, Some(Length::Fixed(35.0)));
        assert_eq!(bar.on_drag, Some(7));
        assert_eq!(bar.controls.as_ref().unwrap().on_close, Some(8));
        let _: Element<'_, u8> = bar.into();
    }
}

#[cfg(test)]
mod corner_tests {
    use super::corner_inset;

    #[test]
    fn a_square_corner_or_a_button_below_the_curve_needs_nothing() {
        assert_eq!(corner_inset(0.0, 12.0, 6.0), 0.0);
        assert_eq!(corner_inset(15.0, 15.0, 6.0), 0.0);
        assert_eq!(corner_inset(15.0, 20.0, 6.0), 0.0);
    }

    #[test]
    fn the_button_corner_keeps_its_clearance_from_the_arc() {
        for (radius, above, clearance) in [(15.0, 11.5, 6.0), (30.0, 12.0, 6.0), (30.0, 12.0, 8.0)] {
            let inset = corner_inset(radius, above, clearance);
            // The distance from the corner point to the arc, along the radius.
            let d = ((radius - inset).powi(2) + (radius - above).powi(2)).sqrt();
            let gap = radius - d;
            assert!(
                (gap - clearance).abs() < 0.01 || inset == 0.0,
                "radius {radius}: gap {gap} for clearance {clearance} (inset {inset})"
            );
        }
        // A bigger radius, the same button: further in.
        assert!(corner_inset(30.0, 12.0, 6.0) > corner_inset(15.0, 12.0, 6.0));
        // The old bubblegum geometry (6 px in, 12 px down, radius 30) touched the arc.
        let touching = ((30.0f32 - 6.0).powi(2) + (30.0f32 - 12.0).powi(2)).sqrt();
        assert!((30.0 - touching).abs() < 0.01);
        assert!(corner_inset(30.0, 12.0, 6.0) > 6.0);
    }

    #[test]
    fn no_position_clears_the_arc_when_the_clearance_exceeds_the_room_above() {
        // More clearance than the button has above it: as far in as the radius.
        assert_eq!(corner_inset(30.0, 12.0, 20.0), 30.0);
    }
}
