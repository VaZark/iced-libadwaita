//! `AdwHeaderBar` and `AdwWindowTitle`.

use crate::Element;
use crate::components::typography;
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
    let mut c = column![typography::heading(title.to_string())].align_x(Horizontal::Center);
    if let Some(s) = subtitle {
        c = c.push(
            typography::caption(s.to_string())
                .wrapping(text::Wrapping::None)
                .style(crate::styles::text::dim),
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
}

/// Builds an empty [`HeaderBar`].
pub fn header_bar<'a, M>() -> HeaderBar<'a, M> {
    HeaderBar::new()
}

impl<'a, M> HeaderBar<'a, M> {
    pub fn new() -> Self {
        Self {
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
        // What a click on the bar does is the theme's `[windows] titlebar-*-click`.
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

/// A window button with the chosen artwork, sized and rounded by the theme's `[windows]` tokens.
fn ctl<'a, M: Clone + 'a>(icon: IconSource, msg: M, flat: Option<bool>) -> Element<'a, M> {
    let size = crate::theme_config::window_button_size();
    let radius = crate::theme_config::window_button_radius();
    let icon_size = crate::theme_config::window_button_icon_size();
    iced::widget::button(container(icon.view(icon_size)).center(Length::Fixed(size)))
        .padding(0)
        .style(move |t: &Theme, status| {
            let mut style = if flat.unwrap_or(t.flat_headerbars) {
                crate::styles::button::circular(t, status)
            } else {
                crate::styles::button::headerbar(t, status)
            };
            style.border.radius = radius.into();
            style
        })
        .on_press(msg)
        .into()
}

impl<'a, M: Clone + 'a> From<HeaderBar<'a, M>> for Element<'a, M> {
    fn from(h: HeaderBar<'a, M>) -> Self {
        let m = h.metrics;
        let (corner_l, corner_r) = h.corners;
        let flat = h.flat;
        let mut start = h.start;
        let mut end = h.end;
        if let Some(c) = h.controls {
            let layout = c.layout.clone().unwrap_or_else(DecorationLayout::detect);
            let make = |b: WindowButton| -> Option<Element<'a, M>> {
                match b {
                    WindowButton::Minimize => c
                        .on_minimize
                        .clone()
                        .map(|m| ctl(h.control_icons.minimize.clone(), m, flat)),
                    WindowButton::Maximize => c.on_maximize.clone().map(|m| {
                        ctl(
                            if c.maximized {
                                h.control_icons.restore.clone()
                            } else {
                                h.control_icons.maximize.clone()
                            },
                            m,
                            flat,
                        )
                    }),
                    WindowButton::Close => c
                        .on_close
                        .clone()
                        .map(|m| ctl(h.control_icons.close.clone(), m, flat)),
                }
            };
            // The window buttons sit together, `[windows] button-gap` apart.
            let group = |buttons: Vec<Element<'a, M>>| -> Option<Element<'a, M>> {
                (!buttons.is_empty()).then(|| {
                    row(buttons)
                        .spacing(crate::theme_config::window_button_gap())
                        .align_y(Vertical::Center)
                        .into()
                })
            };
            if h.show_start_buttons {
                let buttons: Vec<_> = layout.start.iter().copied().filter_map(&make).collect();
                start.splice(0..0, group(buttons));
            }
            if h.show_end_buttons {
                let buttons: Vec<_> = layout.end.iter().copied().filter_map(&make).collect();
                end.extend(group(buttons));
            }
        }
        let title: Element<'a, M> = match (h.title_widget, h.title) {
            (Some(w), _) => w,
            (None, Some(t)) => window_title(&t, h.subtitle.as_deref()),
            (None, None) => space().into(),
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
            iced::widget::container::Style {
                border: iced::Border::default()
                    .rounded(iced::border::top_left(corner_l).top_right(corner_r)),
                ..sc::headerbar(t)
            }
        });

        let mut area = mouse_area(bar);
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
    fn a_header_bar_takes_spacing_padding_and_height() {
        let bar = HeaderBar::<u8>::new()
            .title("T")
            .spacing(2.0)
            .padding([1.0, 3.0])
            .height(40.0)
            .title_gap(5.0);
        assert_eq!((bar.spacing, bar.height, bar.title_gap), (Some(2.0), Some(40.0), Some(5.0)));
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
