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
    pub fn sections(self) -> Vec<(&'static str, Vec<Detail>)> {
        use Detail::*;
        match self {
            Page::Buttons => vec![
                ("Buttons", vec![Button, ButtonContent, LinkAndCard]),
                ("Groups", vec![ToggleGroup]),
            ],
            Page::Inputs => vec![
                ("Text", vec![TextInput, TextView]),
                ("Toggles", vec![Switch, CheckButton, RadioButton]),
                ("Ranges & choosers", vec![Scale, Dropdown]),
            ],
            Page::Lists => vec![
                ("Rows", vec![ActionRow, SwitchAndComboRows, EntryRow, ExpanderRow, ButtonRow]),
                ("Groups", vec![PreferencesGroup]),
            ],
            Page::Layout => vec![
                ("Containers", vec![Card, Clamp, StatusPage]),
                ("Flow", vec![WrapBox, Collapse]),
            ],
            Page::Navigation => vec![
                ("Switchers", vec![ViewSwitcher, TabBar, PathBar]),
                ("Paging", vec![Carousel]),
                ("Menus & shortcuts", vec![MenuButton, ShortcutLabel]),
            ],
            Page::Feedback => vec![
                ("Messages", vec![Banner, Toast]),
                ("Status", vec![Progress, Avatar, Tag]),
            ],
            Page::Dialogs => vec![("Overlays", vec![AlertDialog, AboutDialog, ShortcutsDialog, PreferencesDialog, BottomSheet])],
            Page::Typography | Page::Theming => vec![],
        }
    }
}

/// A level-2 page: one component with all its properties and states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Detail {
    // Buttons
    Button,
    ButtonContent,
    LinkAndCard,
    ToggleGroup,
    // Inputs
    TextInput,
    TextView,
    Switch,
    CheckButton,
    RadioButton,
    Scale,
    Dropdown,
    // Lists
    ActionRow,
    SwitchAndComboRows,
    EntryRow,
    ExpanderRow,
    ButtonRow,
    PreferencesGroup,
    // Layout
    Card,
    Clamp,
    StatusPage,
    WrapBox,
    Collapse,
    // Navigation
    ViewSwitcher,
    TabBar,
    PathBar,
    Carousel,
    MenuButton,
    ShortcutLabel,
    // Feedback
    Banner,
    Toast,
    Progress,
    Avatar,
    Tag,
    // Dialogs
    AlertDialog,
    AboutDialog,
    ShortcutsDialog,
    PreferencesDialog,
    BottomSheet,
}

impl Detail {
    pub fn all() -> Vec<Detail> {
        Page::ALL.iter().flat_map(|p| p.sections()).flat_map(|(_, d)| d).collect()
    }

    pub fn page(self) -> Page {
        Page::ALL
            .into_iter()
            .find(|p| p.sections().iter().any(|(_, d)| d.contains(&self)))
            .expect("every detail belongs to a page")
    }

    pub fn title(self) -> &'static str {
        match self {
            Detail::Button => "Button",
            Detail::ButtonContent => "Button content",
            Detail::LinkAndCard => "Link & card button",
            Detail::ToggleGroup => "Toggle group",
            Detail::TextInput => "Text input",
            Detail::TextView => "Multi-line text",
            Detail::Switch => "Switch",
            Detail::CheckButton => "Check button",
            Detail::RadioButton => "Radio button",
            Detail::Scale => "Scale",
            Detail::Dropdown => "Dropdown",
            Detail::ActionRow => "Action row",
            Detail::SwitchAndComboRows => "Switch & combo rows",
            Detail::EntryRow => "Entry row",
            Detail::ExpanderRow => "Expander row",
            Detail::ButtonRow => "Button row",
            Detail::PreferencesGroup => "Preferences group",
            Detail::Card => "Card",
            Detail::Clamp => "Clamp",
            Detail::StatusPage => "Status page",
            Detail::WrapBox => "Wrap box",
            Detail::Collapse => "Collapse behavior",
            Detail::ViewSwitcher => "View switcher",
            Detail::TabBar => "Tab bar",
            Detail::PathBar => "Path bar",
            Detail::Carousel => "Carousel",
            Detail::MenuButton => "Menu button",
            Detail::ShortcutLabel => "Shortcut label",
            Detail::Banner => "Banner",
            Detail::Toast => "Toast",
            Detail::Progress => "Progress & spinner",
            Detail::Avatar => "Avatar",
            Detail::Tag => "Tag",
            Detail::AlertDialog => "Alert dialog",
            Detail::AboutDialog => "About dialog",
            Detail::ShortcutsDialog => "Shortcuts dialog",
            Detail::PreferencesDialog => "Preferences dialog",
            Detail::BottomSheet => "Bottom sheet",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Detail::Button => "Default, flat, suggested, destructive, pill and circular",
            Detail::ButtonContent => "Icon and label, split button",
            Detail::LinkAndCard => "Link button and card button, enabled and disabled",
            Detail::ToggleGroup => "Exclusive choices: text, icons, crowded and disabled",
            Detail::TextInput => "States, password, color override and width",
            Detail::TextView => "Editable, placeholder and read-only",
            Detail::Switch => "On, off and disabled",
            Detail::CheckButton => "Checked, unchecked and disabled",
            Detail::RadioButton => "A group of exclusive options",
            Detail::Scale => "Slider, progress and row variants",
            Detail::Dropdown => "Selected, empty, disabled options and disabled",
            Detail::ActionRow => "Title, subtitle, prefix, suffix and chevron",
            Detail::SwitchAndComboRows => "Switch, combo, spin and slider rows, with disabled",
            Detail::EntryRow => "Entry and password rows",
            Detail::ExpanderRow => "Rows that reveal more rows",
            Detail::ButtonRow => "Neutral, suggested and destructive",
            Detail::PreferencesGroup => "Title, description and a header suffix",
            Detail::Card => "One chainable box: padding, children and separators",
            Detail::Clamp => "Limit the width of content",
            Detail::StatusPage => "Empty and error states",
            Detail::WrapBox => "Children that wrap onto new lines",
            Detail::Collapse => "Stacked or flex when a row is too narrow",
            Detail::ViewSwitcher => "Switch between views",
            Detail::TabBar => "Closable tabs",
            Detail::PathBar => "A location in the header",
            Detail::Carousel => "Swipe or click through pages",
            Detail::MenuButton => "Dropdown or full-page menu",
            Detail::ShortcutLabel => "Keyboard accelerators",
            Detail::Banner => "A message with an action",
            Detail::Toast => "A brief message at the bottom",
            Detail::Progress => "Progress bars, colors and a spinner",
            Detail::Avatar => "Sizes and initials",
            Detail::Tag => "Colored labels",
            Detail::AlertDialog => "A question with responses",
            Detail::AboutDialog => "Application information",
            Detail::ShortcutsDialog => "Grouped shortcuts",
            Detail::PreferencesDialog => "A page of preferences in a dialog",
            Detail::BottomSheet => "A panel sliding up from the bottom",
        }
    }
}
