//! The gallery's two-level navigation: functional **groups** in the sidebar, and one nested
//! page per component inside a group showing all of its properties and states.

use iced_libadwaita::Icon;

/// A level-1 page (a sidebar entry). Groups with details list their components; the others
/// (`Typography`, `Theming`) are a single page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Page {
    Buttons,
    Inputs,
    Lists,
    Layout,
    Navigation,
    Feedback,
    Dialogs,
    Typography,
    Theming,
}

impl Page {
    pub const ALL: [Page; 9] = [
        Page::Buttons,
        Page::Inputs,
        Page::Lists,
        Page::Layout,
        Page::Navigation,
        Page::Feedback,
        Page::Dialogs,
        Page::Typography,
        Page::Theming,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Page::Buttons => "Buttons",
            Page::Inputs => "Inputs",
            Page::Lists => "Rows & Lists",
            Page::Layout => "Layout",
            Page::Navigation => "Navigation",
            Page::Feedback => "Feedback",
            Page::Dialogs => "Dialogs & Sheets",
            Page::Typography => "Typography",
            Page::Theming => "Theming",
        }
    }

    /// Shown under the title when the sidebar is presented as cards.
    pub fn description(self) -> &'static str {
        match self {
            Page::Buttons => "Styles, shapes, content and toggles",
            Page::Inputs => "Text, switches, checks, ranges and choosers",
            Page::Lists => "Action, entry, expander and preference rows",
            Page::Layout => "Cards, status pages, wrapping and clamping",
            Page::Navigation => "Switchers, tabs, carousels and menus",
            Page::Feedback => "Banners, toasts, progress, avatars and tags",
            Page::Dialogs => "Alerts, about, shortcuts and sheets",
            Page::Typography => "Text styles and colors",
            Page::Theming => "Colors, radii, shadows and motion",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Page::Buttons => Icon::Check,
            Page::Inputs => Icon::Edit,
            Page::Lists => Icon::OpenMenu,
            Page::Layout => Icon::Window,
            Page::Navigation => Icon::GoNext,
            Page::Feedback => Icon::DialogInformation,
            Page::Dialogs => Icon::Sidebar,
            Page::Typography => Icon::Search,
            Page::Theming => Icon::Settings,
        }
    }

    /// The components of this group, in labeled sections. Empty: the page is the content.
    /// How each entry is presented follows the rule in `README.md`.
    pub fn sections(self) -> Vec<(&'static str, Vec<Entry>)> {
        use Action::*;
        use Detail::*;
        use Inline as I;
        let n = Entry::Nested;
        let a = Entry::Action;
        let i = Entry::Inline;
        match self {
            Page::Buttons => vec![
                (
                    "Buttons",
                    vec![
                        n(Button),
                        n(ButtonContent),
                        i(I::LinkButton),
                        i(I::CardButton),
                    ],
                ),
                ("Groups", vec![n(ToggleGroup)]),
            ],
            Page::Inputs => vec![
                ("Text", vec![n(TextInput), n(TextView)]),
                (
                    "Toggles",
                    vec![i(I::Switch), i(I::CheckButton), i(I::RadioButton)],
                ),
                ("Ranges & choosers", vec![i(I::Scale), n(Dropdown)]),
            ],
            Page::Lists => vec![
                (
                    "Rows",
                    vec![
                        n(ActionRow),
                        n(SwitchAndComboRows),
                        n(EntryRow),
                        i(I::ExpanderRow),
                        i(I::ButtonRow),
                    ],
                ),
                ("Groups", vec![n(PreferencesGroup)]),
            ],
            Page::Layout => vec![
                ("Containers", vec![n(Card), i(I::Clamp), i(I::StatusPage)]),
                ("Flow", vec![i(I::WrapBox), i(I::Collapse)]),
            ],
            Page::Navigation => vec![
                (
                    "Switchers",
                    vec![i(I::ViewSwitcher), i(I::TabBar), i(I::PathBar)],
                ),
                ("Paging", vec![i(I::Carousel)]),
                (
                    "Menus & shortcuts",
                    vec![i(I::MenuButton), i(I::ShortcutLabel)],
                ),
            ],
            Page::Feedback => vec![
                ("Messages", vec![i(I::Banner), n(Toast)]),
                ("Status", vec![i(I::Progress), i(I::Avatar), i(I::Tag)]),
            ],
            Page::Dialogs => vec![(
                "Overlays",
                vec![
                    a(AlertDialog),
                    a(AboutDialog),
                    a(ShortcutsDialog),
                    a(PreferencesDialog),
                    a(BottomSheet),
                ],
            )],
            Page::Theming => vec![
                ("Look", vec![n(ThemeColors), n(ThemeShape)]),
                ("Behavior", vec![n(ThemeMotion), n(ThemeWindow)]),
                ("Reference", vec![i(I::Surfaces)]),
            ],
            Page::Typography => vec![],
        }
    }
}

/// One entry of a group page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Entry {
    /// Opens a nested page: the component has two or more significantly different properties.
    Nested(Detail),
    /// A single trigger: a button on the row instead of a chevron.
    Action(Action),
    /// A single demo, shown directly on the group page.
    Inline(Inline),
}

/// A level-2 page: a component with two or more significantly different properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Detail {
    // Buttons
    Button,
    ButtonContent,
    ToggleGroup,
    // Inputs
    TextInput,
    TextView,
    Dropdown,
    // Lists
    ActionRow,
    SwitchAndComboRows,
    EntryRow,
    PreferencesGroup,
    // Layout
    Card,
    // Feedback
    Toast,
    // Theming
    ThemeColors,
    ThemeShape,
    ThemeMotion,
    ThemeWindow,
}

/// A component with one demo, rendered inline on its group page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inline {
    LinkButton,
    CardButton,
    Switch,
    CheckButton,
    RadioButton,
    Scale,
    ExpanderRow,
    ButtonRow,
    Clamp,
    StatusPage,
    WrapBox,
    Collapse,
    ViewSwitcher,
    TabBar,
    PathBar,
    Carousel,
    MenuButton,
    ShortcutLabel,
    Banner,
    Progress,
    Avatar,
    Tag,
    Surfaces,
}

/// A component whose whole demo is one trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    AlertDialog,
    AboutDialog,
    ShortcutsDialog,
    PreferencesDialog,
    BottomSheet,
}

impl Action {
    pub fn title(self) -> &'static str {
        match self {
            Action::AlertDialog => "Alert dialog",
            Action::AboutDialog => "About dialog",
            Action::ShortcutsDialog => "Shortcuts dialog",
            Action::PreferencesDialog => "Preferences dialog",
            Action::BottomSheet => "Bottom sheet",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Action::AlertDialog => "A question with Cancel, Save and Discard responses",
            Action::AboutDialog => "Application name, version, links and license",
            Action::ShortcutsDialog => "Keyboard shortcuts in sections",
            Action::PreferencesDialog => "A page of preference rows in a dialog",
            Action::BottomSheet => "A panel sliding up from the bottom",
        }
    }
    /// The label of the button on the row.
    pub fn button(self) -> &'static str {
        "Open"
    }
}

impl Detail {
    pub fn all() -> Vec<Detail> {
        Page::ALL
            .iter()
            .flat_map(|p| p.sections())
            .flat_map(|(_, e)| e)
            .filter_map(|e| match e {
                Entry::Nested(d) => Some(d),
                _ => None,
            })
            .collect()
    }

    pub fn page(self) -> Page {
        Page::ALL
            .into_iter()
            .find(|p| {
                p.sections()
                    .iter()
                    .any(|(_, e)| e.contains(&Entry::Nested(self)))
            })
            .expect("every detail belongs to a page")
    }

    pub fn title(self) -> &'static str {
        match self {
            Detail::Button => "Button",
            Detail::ButtonContent => "Button content",
            Detail::ToggleGroup => "Toggle group",
            Detail::TextInput => "Text input",
            Detail::TextView => "Multi-line text",
            Detail::Dropdown => "Dropdown",
            Detail::ActionRow => "Action row",
            Detail::SwitchAndComboRows => "Switch & combo rows",
            Detail::EntryRow => "Entry row",
            Detail::PreferencesGroup => "Preferences group",
            Detail::Card => "Card",
            Detail::Toast => "Toast",
            Detail::ThemeColors => "Colors",
            Detail::ThemeShape => "Shape & shadows",
            Detail::ThemeMotion => "Motion",
            Detail::ThemeWindow => "Window & menus",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Detail::Button => "Styles, shapes and their disabled states",
            Detail::ButtonContent => "Icon and label, split button",
            Detail::ToggleGroup => "Text, icons, crowded and as a row",
            Detail::TextInput => "Validation, password, color override and width",
            Detail::TextView => "Editable, placeholder and read-only",
            Detail::Dropdown => "Selected, empty and disabled; combo rows with disabled options",
            Detail::ActionRow => "Subtitle, prefix, suffix and chevron",
            Detail::SwitchAndComboRows => "Switch, combo, spin and slider rows, with disabled",
            Detail::EntryRow => "Entry and password rows",
            Detail::PreferencesGroup => "Title, description and a header suffix",
            Detail::Card => "Padding, chained children, separators and size",
            Detail::Toast => "Call to action, status dot, radius, placement and colors",
            Detail::ThemeColors => "Scheme, accent, density and a card color override",
            Detail::ThemeShape => "Control, card and toast radii, shadows, with a live preview",
            Detail::ThemeMotion => "Animations on or off, easings and springs",
            Detail::ThemeWindow => "Menu presentation, mobile sidebar and the path bar title",
        }
    }
}
