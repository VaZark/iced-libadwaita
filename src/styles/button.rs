//! Button styles: `.default`, `.flat`, `.suggested-action`,
//! `.destructive-action`, `.pill`, `.circular`, `.card`, `.osd`, …

use crate::Theme;
use crate::color::{alpha, fade};
use crate::semantic::Role;
use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Vector};

/// Position of a button inside a `.linked` group (shared corners).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Only,
    Start,
    Middle,
    End,
}

fn border(radius: impl Into<iced::border::Radius>) -> Border {
    Border {
        radius: radius.into(),
        ..Border::default()
    }
}

fn linked_radius(r: f32, pos: Position) -> iced::border::Radius {
    match pos {
        Position::Only => r.into(),
        Position::Start => iced::border::left(r),
        Position::End => iced::border::right(r),
        Position::Middle => 0.0.into(),
    }
}

/// Overrides what the variant's tokens set (`bg`, `fg`, `border`, `border-width`, `radius`,
/// `hover-bg`, `active-bg`) on `style`, the look the built-in computed for `status`. A token the
/// theme leaves unset keeps the built-in value.
fn apply(
    v: &crate::theme_config::ResolvedVariant<'_>,
    theme: &Theme,
    status: Status,
    mut style: Style,
) -> Style {
    let s = &theme.semantic;
    let disabled = |c: Color| fade(c, s.opacity.disabled);
    let fill = |c: Color| Some(Background::Color(c));
    match (v.color("bg"), status) {
        (Some(bg), Status::Active) => style.background = fill(bg),
        (Some(bg), Status::Hovered) => {
            style.background = fill(v.color("hover-bg").unwrap_or_else(|| s.hover_fill(bg)))
        }
        (Some(bg), Status::Pressed) => {
            style.background = fill(v.color("active-bg").unwrap_or_else(|| s.active_fill(bg)))
        }
        (Some(bg), Status::Disabled) => style.background = fill(disabled(bg)),
        (None, Status::Hovered) => {
            if let Some(c) = v.color("hover-bg") {
                style.background = fill(c);
            }
        }
        (None, Status::Pressed) => {
            if let Some(c) = v.color("active-bg") {
                style.background = fill(c);
            }
        }
        (None, _) => {}
    }
    // A gradient fill paints over the flat one; hover and pressed derive from it per stop.
    let base = v.gradient("bg-gradient");
    let gradient = match status {
        Status::Active => base.cloned(),
        Status::Hovered => v
            .gradient("hover-bg-gradient")
            .cloned()
            .or_else(|| base.map(|g| g.map(|c| s.hover_fill(c)))),
        Status::Pressed => v
            .gradient("active-bg-gradient")
            .cloned()
            .or_else(|| base.map(|g| g.map(|c| s.active_fill(c)))),
        Status::Disabled => base.map(|g| g.faded(s.opacity.disabled)),
    };
    if let Some(g) = gradient {
        style.background = Some(g.background());
    }
    if let Some(fg) = v.color("fg") {
        style.text_color = if status == Status::Disabled {
            disabled(fg)
        } else {
            fg
        };
    }
    // A menu bar item turns white on the navy highlight: its own text color while hovered/pressed.
    match status {
        Status::Hovered => {
            if let Some(fg) = v.color("hover-fg") {
                style.text_color = fg;
            }
        }
        Status::Pressed => {
            if let Some(fg) = v.color("active-fg").or_else(|| v.color("hover-fg")) {
                style.text_color = fg;
            }
        }
        _ => {}
    }
    // A drawn bevel (`border-style` raised, sunken, …) is the border: no solid one under it.
    let bevelled = v.text("border-style").is_some_and(|s| s != "solid");
    if !bevelled {
        if let Some(c) = v.color("border") {
            style.border.color = c;
            if style.border.width == 0.0 {
                style.border.width = theme.metrics.border_width;
            }
        }
        if let Some(w) = v.length("border-width") {
            style.border.width = w;
        }
    }
    if let Some(r) = v.length("radius") {
        style.border.radius = r.into();
    }
    style
}

/// The built-in look of the default variant called `name`; the neutral button for any other name.
fn builtin(name: &str) -> fn(&Theme, Status) -> Style {
    match name {
        "raised" => raised_base,
        "flat" => flat_base,
        "headerbar" => headerbar_base,
        "suggested" => suggested_base,
        "destructive" => destructive_base,
        "opaque" => opaque_base,
        "pill" => pill_base,
        "suggested-pill" => suggested_pill_base,
        "link" => link_base,
        "osd" => osd_base,
        "card" => card_base,
        _ => default_base,
    }
}

/// `base` with the tokens of the variant `name` over it, when the theme declares it. The public
/// styles are these: each is the default variant of its name, so a theme can restyle it.
fn tuned(name: &str, theme: &Theme, status: Status, base: fn(&Theme, Status) -> Style) -> Style {
    let style = base(theme, status);
    let v = theme.variant("button", name);
    if v.exists() {
        apply(&v, theme, status, style)
    } else {
        style
    }
}

/// The style of the **variant** `name`: a named set of style tokens (`[button.<name>]` in a theme
/// file, or declared by the app with [`Schema::variant`](crate::theme_config::Schema::variant)),
/// over the built-in look of the toolkit's default variant of that name. A name the theme does not
/// declare (an app that opted out of the defaults, or a typo) draws the neutral button.
///
/// ```ignore
/// button(label).style(styles::button::variant("brand"))
/// ```
pub fn variant(name: &str) -> impl Fn(&Theme, Status) -> Style + Clone + use<> {
    let name = name.to_string();
    move |theme, status| {
        let v = theme.variant("button", &name);
        if !v.exists() {
            return default(theme, status);
        }
        apply(&v, theme, status, builtin(&name)(theme, status))
    }
}

/// [`variant`] for a button that is a circle of `size` px: the corner radius is half the size
/// whatever the variant or the theme says.
pub fn circular_variant(name: &str, size: f32) -> impl Fn(&Theme, Status) -> Style + Clone + use<> {
    let style = variant(name);
    move |theme, status| {
        let mut s = style(theme, status);
        s.border.radius = (size / 2.0).into();
        s
    }
}

/// Translucent state-layer button built from `fg` (`alpha(currentColor, x)`).
fn tinted(
    theme: &Theme,
    status: Status,
    fg: Color,
    rest: f32,
    radius: iced::border::Radius,
) -> Style {
    let o = theme.semantic.opacity;
    let (layer, text) = match status {
        Status::Active => (rest, fg),
        Status::Hovered => (
            if rest == 0.0 {
                o.flat_hover
            } else {
                o.control_hover
            },
            fg,
        ),
        Status::Pressed => (
            if rest == 0.0 {
                o.flat_active
            } else {
                o.control_active
            },
            fg,
        ),
        Status::Disabled => (rest, fade(fg, o.disabled)),
    };
    Style {
        background: (layer > 0.0).then(|| Background::Color(alpha(fg, layer))),
        text_color: text,
        border: border(radius),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Solid-fill button (suggested / destructive / opaque).
fn filled(
    theme: &Theme,
    status: Status,
    fill: Color,
    fg: Color,
    radius: iced::border::Radius,
) -> Style {
    let s = &theme.semantic;
    let (bg, text) = match status {
        Status::Active => (fill, fg),
        Status::Hovered => (s.hover_fill(fill), fg),
        Status::Pressed => (s.active_fill(fill), fg),
        Status::Disabled => (fade(fill, s.opacity.disabled), fade(fg, s.opacity.disabled)),
    };
    Style {
        background: Some(Background::Color(bg)),
        text_color: text,
        border: border(radius),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// The standard button: translucent foreground background, 6px radius.
fn default_base(theme: &Theme, status: Status) -> Style {
    tinted(
        theme,
        status,
        theme.semantic.window.fg,
        theme.semantic.opacity.control,
        theme.metrics.radius_m.into(),
    )
}

/// `.raised`: identical to default (kept for API parity with GTK).
fn raised_base(theme: &Theme, status: Status) -> Style {
    default_base(theme, status)
}

/// `.flat`: transparent until hovered.
fn flat_base(theme: &Theme, status: Status) -> Style {
    tinted(
        theme,
        status,
        theme.semantic.window.fg,
        0.0,
        theme.metrics.radius_m.into(),
    )
}

/// `.flat` using the headerbar foreground.
fn headerbar_base(theme: &Theme, status: Status) -> Style {
    tinted(
        theme,
        status,
        theme.semantic.headerbar.fg,
        0.0,
        theme.metrics.radius_m.into(),
    )
}

/// `.suggested-action`.
fn suggested_base(theme: &Theme, status: Status) -> Style {
    role(Role::Primary)(theme, status)
}

/// `.destructive-action`.
fn destructive_base(theme: &Theme, status: Status) -> Style {
    role(Role::Destructive)(theme, status)
}

/// `.opaque`: solid primary fill (same as suggested, kept for parity).
fn opaque_base(theme: &Theme, status: Status) -> Style {
    suggested_base(theme, status)
}

/// Solid button for any [`Role`]. `Role::Neutral` falls back to [`default`].
pub fn role(r: Role) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| match r {
        Role::Neutral => default(theme, status),
        _ => {
            let sw = theme.semantic.swatch(r);
            filled(theme, status, sw.bg, sw.fg, theme.metrics.radius_m.into())
        }
    }
}

/// [`role`] with a **hard-coded** text color, overriding the contrast-chosen default.
/// `Role::Neutral` falls back to [`default`].
pub fn role_with_text(r: Role, fg: Color) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| match r {
        Role::Neutral => default(theme, status),
        _ => {
            let sw = theme.semantic.swatch(r);
            filled(theme, status, sw.bg, fg, theme.metrics.radius_m.into())
        }
    }
}

/// `.pill`: fully rounded standard button.
fn pill_base(theme: &Theme, status: Status) -> Style {
    tinted(
        theme,
        status,
        theme.semantic.window.fg,
        theme.semantic.opacity.control,
        theme.metrics.radius_pill.into(),
    )
}

/// `.pill.suggested-action`.
fn suggested_pill_base(theme: &Theme, status: Status) -> Style {
    let sw = theme.semantic.primary;
    filled(
        theme,
        status,
        sw.bg,
        sw.fg,
        theme.metrics.radius_pill.into(),
    )
}

/// `.circular`: use with equal width/height.
pub fn circular(theme: &Theme, status: Status) -> Style {
    pill(theme, status)
}

/// `.circular.suggested-action`: use with equal width/height.
pub fn circular_suggested(theme: &Theme, status: Status) -> Style {
    suggested_pill(theme, status)
}

/// `.circular.flat`.
pub fn circular_flat(theme: &Theme, status: Status) -> Style {
    tinted(
        theme,
        status,
        theme.semantic.window.fg,
        0.0,
        theme.metrics.radius_pill.into(),
    )
}

/// The events of a button and nothing else: transparent, with a hover and press tint over the
/// content's bounds, rounded like a card. The content owns its surface and its layout. Use it as
/// `button(content).padding(0).style(styles::button::events).on_press(m)`: a button is events,
/// not layout.
pub fn events(theme: &Theme, status: Status) -> Style {
    let s = &theme.semantic;
    let mut st = row_look(theme, status, s.card.fg, theme.metrics.radius_l, 0.04, 0.08);
    st.border = iced::Border::default().rounded(theme.metrics.radius_l);
    st
}

/// `.card`: a clickable card surface.
fn card_base(theme: &Theme, status: Status) -> Style {
    let s = &theme.semantic;
    let bg = match status {
        Status::Hovered => crate::color::mix(s.card.bg, s.card.fg, 0.04),
        Status::Pressed => crate::color::mix(s.card.bg, s.card.fg, 0.08),
        _ => s.card.bg,
    };
    Style {
        background: Some(Background::Color(bg)),
        text_color: if status == Status::Disabled {
            fade(s.card.fg, s.opacity.disabled)
        } else {
            s.card.fg
        },
        border: theme.shadows.card().border(theme.metrics.radius_l),
        shadow: theme.shadows.card().shadow,
        snap: false,
    }
}

/// `.osd`: dark translucent button for overlays.
fn osd_base(theme: &Theme, status: Status) -> Style {
    let osd = theme.semantic.osd;
    let layer = match status {
        Status::Hovered => 0.15,
        Status::Pressed => 0.25,
        _ => 0.0,
    };
    Style {
        background: Some(Background::Color(crate::color::mix(osd.bg, osd.fg, layer))),
        text_color: osd.fg,
        border: border(theme.metrics.radius_pill),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// `.link`: accent-colored text that underlines on hover (underline omitted;
/// iced buttons cannot underline children).
fn link_base(theme: &Theme, status: Status) -> Style {
    let c = theme.semantic.accent.color;
    Style {
        background: None,
        text_color: match status {
            Status::Disabled => fade(c, theme.semantic.opacity.disabled),
            Status::Hovered | Status::Pressed => {
                crate::color::mix(c, theme.semantic.window.fg, 0.2)
            }
            Status::Active => c,
        },
        border: border(theme.metrics.radius_s),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// The look of a pressable row: a layer of the foreground on hover and press, `radius` corners.
fn row_look(
    theme: &Theme,
    status: Status,
    fg: Color,
    radius: f32,
    hover: f32,
    press: f32,
) -> Style {
    let o = theme.semantic.opacity;
    let layer = match status {
        Status::Hovered => hover,
        Status::Pressed => press,
        _ => 0.0,
    };
    Style {
        background: (layer > 0.0).then(|| Background::Color(alpha(fg, layer))),
        text_color: if status == Status::Disabled {
            fade(fg, o.disabled)
        } else {
            fg
        },
        border: border(radius),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// A row inside a boxed list (`row.activatable`): flat with a subtle layer, square corners.
pub fn list_row(theme: &Theme, status: Status) -> Style {
    // `row.activatable:hover` 3%, `:active` 8% (libadwaita _lists.scss).
    row_look(theme, status, theme.semantic.card.fg, 0.0, 0.03, 0.08)
}

/// A flat pressable row on the page itself (a track in a list, a result): the whole row is the
/// target, with `radius` corners and a layer of the window foreground at `hover` and `pressed`
/// opacity (`0..=1`; 0.08 and 0.14 read clearly on a plain page). [`list_row`] is the same inside
/// a boxed list, with the libadwaita values.
///
/// ```ignore
/// button(row).width(Length::Fill).style(style::button::flat_row(8.0, 0.08, 0.14))
/// ```
pub fn flat_row(radius: f32, hover: f32, pressed: f32) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        row_look(
            theme,
            status,
            theme.semantic.window.fg,
            radius,
            hover,
            pressed,
        )
    }
}

/// Add a color layer beneath the target's fill without changing any target geometry
/// or text/border/shadow properties. Opaque custom fills continue to cover the layer;
/// transparent hover/pressed fills compose over it just as separate stacked fills do.
pub(crate) fn with_underlay(mut target: Style, underlay: Color) -> Style {
    if underlay.a <= 0.0 {
        return target;
    }
    target.background = Some(match target.background {
        None => Background::Color(underlay),
        Some(Background::Color(fill)) => {
            Background::Color(crate::color::composite_over(fill, underlay))
        }
        Some(Background::Gradient(iced::Gradient::Linear(mut gradient))) => {
            if gradient.stops.iter().all(Option::is_none) {
                Background::Color(underlay)
            } else {
                for stop in gradient.stops.iter_mut().flatten() {
                    stop.color = crate::color::composite_over(stop.color, underlay);
                }
                Background::Gradient(iced::Gradient::Linear(gradient))
            }
        }
    });
    target
}

/// A row in a `.navigation-sidebar`; `selected` shows the selection fill.
pub fn sidebar_row(selected: bool) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let fg = theme.semantic.sidebar.fg;
        let o = theme.semantic.opacity;
        let layer = match (selected, status) {
            (true, Status::Hovered) => 0.14,
            (true, _) => 0.10,
            (false, Status::Hovered) => o.flat_hover * 0.6,
            (false, Status::Pressed) => o.flat_active * 0.6,
            _ => 0.0,
        };
        Style {
            background: (layer > 0.0).then(|| Background::Color(alpha(fg, layer))),
            text_color: fg,
            border: border(theme.metrics.radius_m),
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// A toggle in an `AdwToggleGroup`/view switcher. The active toggle uses the
/// *active toggle* surface with a faint shadow; others are plain.
pub fn toggle(active: bool, pos: Position) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let o = theme.semantic.opacity;
        let r = linked_radius(theme.metrics.radius_m, pos);
        if active {
            let t = theme.semantic.active_toggle;
            Style {
                background: Some(Background::Color(t.bg)),
                text_color: if status == Status::Disabled {
                    fade(t.fg, o.disabled)
                } else {
                    t.fg
                },
                border: border(r),
                shadow: Shadow {
                    color: alpha(Color::BLACK, if theme.is_dark() { 0.3 } else { 0.12 }),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
                snap: false,
            }
        } else {
            tinted(theme, status, theme.semantic.window.fg, 0.0, r)
        }
    }
}

/// Standard button with linked-group corners (`.linked`).
pub fn linked(pos: Position) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let mut s = default(theme, status);
        s.border.radius = linked_radius(theme.metrics.radius_m, pos);
        s
    }
}

/// Flat button that reads as pressed when `active` (menu buttons, tabs).
pub fn flat_toggle(active: bool) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let mut s = flat(theme, status);
        if active && status == Status::Active {
            s.background = Some(Background::Color(alpha(
                theme.semantic.window.fg,
                theme.semantic.opacity.flat_hover,
            )));
        }
        s
    }
}

/// An item in a `.menu` popover: flat, 6px radius, a hover layer, and a
/// slightly stronger layer when selected.
pub fn menu_item(selected: bool) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let s = &theme.semantic;
        let fg = s.popover.fg;
        // Selected is the *active* state, in the accent color; disabled is dimmed and inert.
        let accent = s.accent.color;
        if status == Status::Disabled {
            return Style {
                background: None,
                text_color: crate::color::fade(
                    if selected { accent } else { fg },
                    s.opacity.disabled,
                ),
                border: border(theme.metrics.radius_menu),
                shadow: Shadow::default(),
                snap: false,
            };
        }
        let (tint, layer) = if selected {
            (
                accent,
                match status {
                    Status::Hovered => 0.20,
                    Status::Pressed => 0.26,
                    _ => 0.14,
                },
            )
        } else {
            (
                fg,
                match status {
                    Status::Hovered => 0.10,
                    Status::Pressed => 0.16,
                    _ => 0.0,
                },
            )
        };
        Style {
            background: (layer > 0.0).then(|| Background::Color(alpha(tint, layer))),
            text_color: if selected { accent } else { fg },
            border: border(theme.metrics.radius_menu),
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// The standard button: translucent foreground background, 6px radius (`[button.default]`).
pub fn default(theme: &Theme, status: Status) -> Style {
    tuned("default", theme, status, default_base)
}

/// `.raised`: identical to default (`[button.raised]`).
pub fn raised(theme: &Theme, status: Status) -> Style {
    tuned("raised", theme, status, raised_base)
}

/// `.flat`: transparent until hovered (`[button.flat]`).
pub fn flat(theme: &Theme, status: Status) -> Style {
    tuned("flat", theme, status, flat_base)
}

/// `.flat` using the headerbar foreground (`[button.headerbar]`).
pub fn headerbar(theme: &Theme, status: Status) -> Style {
    tuned("headerbar", theme, status, headerbar_base)
}

/// `.suggested-action`: the primary color (`[button.suggested]`).
pub fn suggested(theme: &Theme, status: Status) -> Style {
    tuned("suggested", theme, status, suggested_base)
}

/// `.destructive-action` (`[button.destructive]`).
pub fn destructive(theme: &Theme, status: Status) -> Style {
    tuned("destructive", theme, status, destructive_base)
}

/// `.opaque`: solid primary fill (`[button.opaque]`).
pub fn opaque(theme: &Theme, status: Status) -> Style {
    tuned("opaque", theme, status, opaque_base)
}

/// `.pill`: fully rounded standard button (`[button.pill]`).
pub fn pill(theme: &Theme, status: Status) -> Style {
    tuned("pill", theme, status, pill_base)
}

/// `.pill.suggested-action` (`[button.suggested-pill]`).
pub fn suggested_pill(theme: &Theme, status: Status) -> Style {
    tuned("suggested-pill", theme, status, suggested_pill_base)
}

/// `.link`: accent-colored text (`[button.link]`).
pub fn link(theme: &Theme, status: Status) -> Style {
    tuned("link", theme, status, link_base)
}

/// `.osd`: dark translucent button for overlays (`[button.osd]`).
pub fn osd(theme: &Theme, status: Status) -> Style {
    tuned("osd", theme, status, osd_base)
}

/// `.card`: a clickable card surface (`[button.card]`).
pub fn card(theme: &Theme, status: Status) -> Style {
    tuned("card", theme, status, card_base)
}

#[cfg(test)]
mod target_effect_tests {
    use super::*;

    #[test]
    fn underlay_preserves_full_target_geometry_and_state_styles() {
        let theme = Theme::light();
        let underlay = alpha(theme.semantic.sidebar.fg, 0.3);
        let asymmetric = iced::border::Radius {
            top_left: 19.0,
            top_right: 0.0,
            bottom_right: 7.0,
            bottom_left: 3.0,
        };
        for radius in [0.0.into(), 999.0.into(), asymmetric] {
            for status in [
                Status::Active,
                Status::Hovered,
                Status::Pressed,
                Status::Disabled,
            ] {
                let mut target = sidebar_row(false)(&theme, status);
                target.border = Border {
                    radius,
                    width: 2.0,
                    color: Color::WHITE,
                };
                target.shadow = Shadow {
                    color: Color::BLACK,
                    offset: Vector::new(3.0, 4.0),
                    blur_radius: 5.0,
                };
                target.snap = true;
                let effect = with_underlay(target, underlay);
                assert_eq!(effect.border, target.border);
                assert_eq!(effect.shadow, target.shadow);
                assert_eq!(effect.text_color, target.text_color);
                assert_eq!(effect.snap, target.snap);
                let expected = match target.background {
                    Some(Background::Color(fill)) => crate::color::composite_over(fill, underlay),
                    None => underlay,
                    _ => unreachable!(),
                };
                assert_eq!(effect.background, Some(Background::Color(expected)));
                assert_eq!(
                    with_underlay(target, Color::TRANSPARENT).background,
                    target.background
                );
            }
        }
    }

    #[test]
    fn tuned_custom_styles_and_gradients_keep_their_properties() {
        use crate::theme_config::{Schema, ThemeConfig};
        let config = ThemeConfig::builder("target-shape")
            .files(false)
            .schema(Schema::toolkit().variant("button", "custom", &[]))
            .toolkit_text(r##"{
    "button.custom": {
        "radius": 99,
        "bg": "#ffffff",
        "hover-bg": "#0000ff",
        "active-bg": "#ff0000",
        "border-width": 2,
        "border": "#00ff00",
    },
}
"##)
            .app_text("").build();
        let theme = Theme::light().with_theme_config(&config);
        for status in [
            Status::Active,
            Status::Hovered,
            Status::Pressed,
            Status::Disabled,
        ] {
            let target = variant("custom")(&theme, status);
            let effect = with_underlay(target, Color::BLACK);
            assert_eq!(effect.border, target.border);
            assert_eq!(effect.border.radius, 99.0.into());
            assert_eq!(effect.text_color, target.text_color);
            if status != Status::Disabled {
                // An opaque custom fill still covers the selection beneath it.
                assert_eq!(effect.background, target.background);
            }
        }
        let gradient = iced::gradient::Linear::new(0.7)
            .add_stop(0.0, Color::from_rgba(1.0, 0.0, 0.0, 0.4))
            .add_stop(1.0, Color::from_rgb(0.0, 0.0, 1.0));
        let target = Style {
            background: Some(Background::Gradient(gradient.into())),
            ..Default::default()
        };
        let underlay = Color::from_rgba(0.0, 1.0, 0.0, 0.2);
        let Some(Background::Gradient(iced::Gradient::Linear(effect))) =
            with_underlay(target, underlay).background
        else {
            panic!("gradient must be retained")
        };
        assert_eq!(effect.angle, gradient.angle);
        for (before, after) in gradient.stops.iter().zip(effect.stops.iter()) {
            match (before, after) {
                (Some(before), Some(after)) => {
                    assert_eq!(before.offset, after.offset);
                    assert_eq!(
                        after.color,
                        crate::color::composite_over(before.color, underlay)
                    );
                }
                (None, None) => {}
                _ => panic!("gradient stop layout changed"),
            }
        }
    }
}

#[cfg(test)]
mod row_tests {
    use super::*;

    #[test]
    fn a_flat_row_has_its_own_corners_and_layers() {
        let t = Theme::light();
        let f = flat_row(8.0, 0.08, 0.14);
        assert!(f(&t, Status::Active).background.is_none());
        let hover = f(&t, Status::Hovered);
        let press = f(&t, Status::Pressed);
        let alpha_of = |s: Style| match s.background {
            Some(Background::Color(c)) => c.a,
            _ => 0.0,
        };
        // The layer is the foreground at that opacity (the foreground may itself be translucent).
        let fg = t.semantic.window.fg.a;
        assert!((alpha_of(hover) - fg * 0.08).abs() < 1e-5);
        assert!((alpha_of(press) - fg * 0.14).abs() < 1e-5);
        assert_eq!(hover.border.radius, 8.0.into());
        // The boxed-list row keeps its square corners and quieter layers.
        assert_eq!(list_row(&t, Status::Hovered).border.radius, 0.0.into());
    }
}
