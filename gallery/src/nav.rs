//! The gallery's two-level navigation: functional **groups** in the sidebar, and one nested
//! page per component inside a group showing all of its properties and states.

use iced_adaptive_design::Icon;
use iced_adaptive_design::widget as w;

/// A level-1 page (a sidebar entry). Groups with details list their components; the others
/// (`Typography`, `Theming`) are a single page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Page {
    Buttons,
    Inputs,
    Lists,
    Data,
    Layout,
    Navigation,
    Feedback,
    Dialogs,
    Typography,
    Theming,
}

impl Page {
    pub const ALL: [Page; 10] = [
        Page::Buttons,
        Page::Inputs,
        Page::Lists,
        Page::Data,
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
            Page::Data => "Data",
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
            Page::Lists => "Action, entry, tree and preference rows",
            Page::Data => "Tables, trees, virtualization and async loading",
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
            Page::Data => Icon::Folder,
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
        use Detail::*;
        let n = Entry::Nested;
        match self {
            Page::Buttons => vec![
                (
                    "Buttons",
                    vec![
                        n(Button),
                        n(ButtonContent),
                        n(IconToggle),
                        n(LinkButton),
                        n(CardButton),
                    ],
                ),
                ("Groups", vec![n(ToggleGroup)]),
                ("Icons", vec![n(Icons)]),
            ],
            Page::Inputs => vec![
                (
                    "Text",
                    vec![
                        n(TextInput),
                        n(TextView),
                        n(EditableText),
                        n(Autocomplete),
                        n(ChipGroup),
                    ],
                ),
                (
                    "Forms",
                    vec![
                        n(Field),
                        n(Form),
                        n(Stepper),
                        n(NumberInput),
                        n(ColorPicker),
                    ],
                ),
                ("Toggles", vec![n(Switch), n(CheckButton), n(RadioButton)]),
                (
                    "Ranges & choosers",
                    vec![
                        n(Scale),
                        n(RangeSlider),
                        n(Dropdown),
                        n(MultiSelect),
                        n(DateInput),
                        n(DateRange),
                    ],
                ),
            ],
            Page::Lists => vec![
                (
                    "Rows",
                    vec![
                        n(ActionRow),
                        n(SwitchAndComboRows),
                        n(EntryRow),
                        n(TreeRow),
                        n(ButtonRow),
                    ],
                ),
                ("Lists", vec![n(ItemRow)]),
                ("Groups", vec![n(PreferencesGroup), n(Accordion)]),
            ],
            Page::Data => vec![
                (
                    "Tables",
                    vec![n(ResponsiveTable), n(Data), n(TableVirtual), n(TableAsync)],
                ),
                (
                    "Trees",
                    vec![
                        n(Tree),
                        n(TreeAsync),
                        n(TreeVirtualized),
                        n(TreeFiles),
                        n(TreeDocs),
                    ],
                ),
                ("Timed", vec![n(SyncedText), n(VirtualList)]),
                (
                    "Charts",
                    vec![
                        n(LineChart),
                        n(BarChart),
                        n(PieChart),
                        n(DonutChart),
                        n(BubbleChart),
                        n(ComboChart),
                        n(Funnel),
                        n(Sankey),
                        n(Sunburst),
                    ],
                ),
            ],
            Page::Layout => vec![
                (
                    "Containers",
                    vec![
                        n(AppShell),
                        n(Container),
                        n(Surface),
                        n(PageHeader),
                        n(FlowGrid),
                        n(TableGrid),
                        n(SectionHeading),
                        n(Card),
                        n(Clamp),
                        n(StatusPage),
                        n(Splitter),
                    ],
                ),
                (
                    "Flow",
                    vec![
                        n(WrapBox),
                        n(Collapse),
                        n(ClipWindow),
                        n(Divider),
                        n(Spoiler),
                    ],
                ),
                (
                    "Meta",
                    vec![
                        n(MetaComponent),
                        n(MetaParent),
                        n(Composed),
                        n(Reorderable),
                        n(Resizable),
                    ],
                ),
            ],
            Page::Navigation => vec![
                ("Switchers", vec![n(Tabs), n(ViewSwitcher), n(PathBar)]),
                ("Paging", vec![n(Carousel), n(History)]),
                (
                    "Menus & shortcuts",
                    vec![n(MenuButton), n(ShortcutLabel), n(AnchoredPopover)],
                ),
            ],
            Page::Feedback => vec![
                ("Messages", vec![n(Banner), n(Toast)]),
                (
                    "Status",
                    vec![
                        n(Progress),
                        n(Avatar),
                        n(Tag),
                        n(BadgeChip),
                        n(Indicator),
                        n(Timeline),
                    ],
                ),
            ],
            Page::Dialogs => vec![(
                "Overlays",
                vec![
                    n(AlertDialog),
                    n(AboutDialog),
                    n(ShortcutsDialog),
                    n(PreferencesDialog),
                    n(Docked),
                ],
            )],
            Page::Theming => vec![
                (
                    "Look",
                    vec![
                        n(ThemeColors),
                        n(ThemeShape),
                        n(ColorHelpers),
                        n(ThemeConfig),
                    ],
                ),
                ("Behavior", vec![n(ThemeMotion), n(ThemeWindow)]),
                ("Reference", vec![n(Surfaces)]),
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
    Field,
    Form,
    Stepper,
    Data,
    Autocomplete,
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
    Divider,
    Splitter,
    ColorPicker,
    NumberInput,
    Timeline,
    Spoiler,

    TableVirtual,
    TableAsync,
    Tree,
    TreeAsync,
    TreeVirtualized,
    TreeFiles,
    TreeDocs,

    Tabs,
    AlertDialog,
    AboutDialog,
    ShortcutsDialog,
    PreferencesDialog,
    AppShell,

    LinkButton,
    Switch,
    CheckButton,
    RadioButton,
    Scale,
    TreeRow,
    ButtonRow,
    Clamp,
    StatusPage,
    WrapBox,
    Collapse,
    ViewSwitcher,
    Avatar,
    Tag,
    ShortcutLabel,
    MultiSelect,
    AnchoredPopover,
    Surfaces,

    MetaComponent,
    Reorderable,
    Resizable,
    PathBar,
    MenuButton,
    Banner,
    Progress,
    BadgeChip,
    Indicator,
    Docked,
    ResponsiveTable,
    Accordion,
    CardButton,
    Carousel,
    DateInput,
    RangeSlider,
    DateRange,
    MetaParent,
    Container,
    VirtualList,
    EditableText,
    PageHeader,
    ClipWindow,
    Surface,
    IconToggle,
    FlowGrid,
    TableGrid,
    SectionHeading,
    ItemRow,
    ChipGroup,
    History,
    Icons,
    Composed,
    SyncedText,
    LineChart,
    BarChart,
    PieChart,
    DonutChart,
    BubbleChart,
    ComboChart,
    Funnel,
    Sankey,
    Sunburst,
    // (the last ones moved from inline demos: two or more properties)
    // Theming
    ThemeColors,
    ColorHelpers,
    ThemeConfig,
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
    ButtonRow,
    Clamp,
    StatusPage,
    WrapBox,
    Collapse,
    ViewSwitcher,
    PathBar,
    Carousel,
    MenuButton,
    ShortcutLabel,
    Banner,
    Progress,
    Avatar,
    Tag,
    BadgeChip,
    Indicator,
    MultiSelect,
    DateInput,
    RangeSlider,
    DateRange,
    AnchoredPopover,
    ResponsiveTable,
    Accordion,
    Surfaces,
}

impl Detail {
    pub fn all() -> Vec<Detail> {
        Page::ALL
            .iter()
            .flat_map(|p| p.sections())
            .flat_map(|(_, e)| e)
            .map(|e| match e {
                Entry::Nested(d) => d,
            })
            .collect()
    }

    /// How wide the page of this component reads: charts and wide demos get more room.
    pub fn container_size(self) -> w::ContainerSize {
        use w::ContainerSize::{Md, Sm};
        match self {
            Detail::LineChart
            | Detail::BarChart
            | Detail::PieChart
            | Detail::Funnel
            | Detail::Sankey
            | Detail::Sunburst
            | Detail::Composed => Md,
            _ => Sm,
        }
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
            Detail::Field => "Form field",
            Detail::Form => "Form",
            Detail::Stepper => "Stepper",
            Detail::Data => "Sorting and pagination",
            Detail::Autocomplete => "Autocomplete",
            Detail::Dropdown => "Dropdown",
            Detail::ActionRow => "Action row",
            Detail::SwitchAndComboRows => "Switch & combo rows",
            Detail::EntryRow => "Entry row",
            Detail::PreferencesGroup => "Preferences group",
            Detail::Card => "Card",
            Detail::Toast => "Toast",
            Detail::Divider => "Divider",
            Detail::Splitter => "Splitter",
            Detail::ColorPicker => "Color picker",
            Detail::NumberInput => "Number input",
            Detail::Timeline => "Timeline",
            Detail::Spoiler => "Spoiler",
            Detail::TableVirtual => "Table: virtualization",
            Detail::TableAsync => "Table: async chunks",
            Detail::Tree => "Tree",
            Detail::TreeAsync => "Tree: async loading",
            Detail::TreeVirtualized => "Tree: virtualization",
            Detail::TreeFiles => "Tree: files",
            Detail::TreeDocs => "Tree: docs editor",
            Detail::Tabs => "Tabs",
            Detail::AlertDialog => "Alert dialog",
            Detail::AboutDialog => "About dialog",
            Detail::ShortcutsDialog => "Shortcuts dialog",
            Detail::PreferencesDialog => "Preferences dialog",
            Detail::AppShell => "AppShell",
            Detail::LinkButton => "Link button",
            Detail::Switch => "Switch",
            Detail::CheckButton => "Check button",
            Detail::RadioButton => "Radio button",
            Detail::Scale => "Scale",
            Detail::TreeRow => "Tree row",
            Detail::ButtonRow => "Button row",
            Detail::Clamp => "Clamp",
            Detail::StatusPage => "Status page",
            Detail::WrapBox => "Wrap box",
            Detail::Collapse => "Collapse behavior",
            Detail::ViewSwitcher => "View switcher",
            Detail::Avatar => "Avatar",
            Detail::Tag => "Tag",
            Detail::ShortcutLabel => "Shortcut label",
            Detail::MultiSelect => "Multi select",
            Detail::AnchoredPopover => "Anchored popover",
            Detail::Surfaces => "Surfaces",
            Detail::PathBar => "Path bar",
            Detail::MenuButton => "Menu button",
            Detail::Banner => "Banner",
            Detail::Progress => "Progress",
            Detail::BadgeChip => "Badge & chip",
            Detail::Indicator => "Indicator",
            Detail::Docked => "Docked surfaces",
            Detail::ResponsiveTable => "Responsive table",
            Detail::Accordion => "Accordion",
            Detail::CardButton => "Card button",
            Detail::Carousel => "Carousel",
            Detail::DateInput => "Date input",
            Detail::RangeSlider => "Range slider",
            Detail::DateRange => "Date range",
            Detail::MetaComponent => "Meta component",
            Detail::Resizable => "Resizable",
            Detail::Reorderable => "Reorderable list",
            Detail::MetaParent => "Meta parent",
            Detail::Container => "Container",
            Detail::VirtualList => "Virtual list",
            Detail::EditableText => "Editable text",
            Detail::PageHeader => "Detail header",
            Detail::ClipWindow => "Clip window",
            Detail::Surface => "Page surfaces",
            Detail::Icons => "Icons",
            Detail::IconToggle => "Icon toggle button",
            Detail::FlowGrid => "Flow grid",
            Detail::TableGrid => "Table grid",
            Detail::SectionHeading => "Section heading",
            Detail::ItemRow => "Item row",
            Detail::ChipGroup => "Chip group",
            Detail::History => "History",
            Detail::Composed => "Composed component",
            Detail::LineChart => "Line chart",
            Detail::BarChart => "Bar chart",
            Detail::PieChart => "Pie chart",
            Detail::DonutChart => "Donut chart",
            Detail::BubbleChart => "Bubble chart",
            Detail::ComboChart => "Combo chart",
            Detail::Funnel => "Funnel",
            Detail::Sankey => "Sankey",
            Detail::Sunburst => "Sunburst",
            Detail::SyncedText => "Synced text",
            Detail::ThemeColors => "Colors",
            Detail::ColorHelpers => "Color helpers",
            Detail::ThemeConfig => "App theme config",
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
            Detail::Field => "Label, control and message with shared validation",
            Detail::Form => "Submit, reset and a multi-step wizard with gated Next",
            Detail::Stepper => "Steps with statuses, click-to-jump, icons, vertical, narrow",
            Detail::Data => "Sortable table and cards, pager, infinite scroll, load more",
            Detail::Autocomplete => "Async suggestions: loading, results, empty, error, in a field",
            Detail::Dropdown => "Selected, empty and disabled; combo rows with disabled options",
            Detail::ActionRow => "Subtitle, prefix, suffix and chevron",
            Detail::SwitchAndComboRows => "Switch, combo, spin and slider rows, with disabled",
            Detail::EntryRow => "Entry and password rows",
            Detail::PreferencesGroup => "Title, description and a header suffix",
            Detail::Card => "Padding, chained children, separators and size",
            Detail::Toast => "Call to action, status dot, radius, placement and colors",
            Detail::Divider => "Orientation, label, variant and size",
            Detail::Splitter => "Resizable panes: fixed and flexible, limits, collapsing",
            Detail::ColorPicker => "Saturation, hue and alpha, swatches, formats and an input",
            Detail::NumberInput => "Integer or fixed decimals, limits, step handles",
            Detail::Timeline => "Events along a line, sharing the stepper primitive",
            Detail::Spoiler => "Part of the height or of the data, the rest on request",
            Detail::TableVirtual => "100,000 rows, only the ones in view built",
            Detail::TableAsync => {
                "5,000 rows fetched in chunks as you scroll, with placeholders and Retry"
            }
            Detail::Tree => "Lines, checkboxes, search, filter and drag and drop",
            Detail::TreeAsync => {
                "Branches that load when opened, with a spinner, an error and Retry"
            }
            Detail::TreeVirtualized => "2,000 nodes, only the rows in view built",
            Detail::TreeFiles => "A file browser with lines, icons and selection",
            Detail::TreeDocs => "Search, drag and drop and a count per branch",
            Detail::Tabs => "Variants, icons, disabled tabs and panels",
            Detail::AlertDialog => "A question with Cancel, Save and Discard responses",
            Detail::AboutDialog => "Application name, version, links and license",
            Detail::ShortcutsDialog => "Keyboard shortcuts in sections",
            Detail::PreferencesDialog => "A page of preference rows in a dialog",
            Detail::AppShell => "A header, navbar and drawer that adapt to the window width",
            Detail::LinkButton => "Label, uri and disabled",
            Detail::Switch => "On, off and disabled",
            Detail::CheckButton => "Checked, label and disabled",
            Detail::RadioButton => "A choice among options, with disabled",
            Detail::Scale => "Value and range",
            Detail::TreeRow => "Rows, or any elements, nested in a row (within a page)",
            Detail::ButtonRow => "Neutral, suggested and destructive",
            Detail::Clamp => "Maximum width",
            Detail::StatusPage => "Icon, title, description and action",
            Detail::WrapBox => "Spacing and wrapping children",
            Detail::Collapse => "Stacked and flex controls in a row",
            Detail::ViewSwitcher => "Icons and labels",
            Detail::Avatar => "Size, name and initials",
            Detail::Tag => "Roles",
            Detail::ShortcutLabel => "Key combinations",
            Detail::MultiSelect => "Several choices in a dropdown",
            Detail::AnchoredPopover => "Side, flip and shift",
            Detail::Surfaces => "The semantic surfaces and status colors",
            Detail::PathBar => "Path and search variants",
            Detail::MenuButton => "Menu, primary menu and popover presentations",
            Detail::Banner => "Roles, buttons and revealing",
            Detail::Progress => "Bars, values and the indeterminate state",
            Detail::BadgeChip => "Roles, solid, icons, selected and disabled",
            Detail::Indicator => "Corner, offset and straddling badges",
            Detail::Docked => "Footer, banner, drawer and bottom sheet: side, size, mode and scope",
            Detail::ResponsiveTable => "A table that becomes cards, by mode",
            Detail::Accordion => "Single and multiple open items",
            Detail::CardButton => "Styles, content and states",
            Detail::Carousel => "Pages, indicators and dragging",
            Detail::DateInput => "Calendar levels, bounds and validation",
            Detail::RangeSlider => "Two thumbs, steps, gaps and rows",
            Detail::DateRange => "Two dates, hover preview and bounds",
            Detail::MetaComponent => "Picked by container width, by selection, or both",
            Detail::Resizable => "A frame to try responsive layouts at any size",
            Detail::Reorderable => {
                "Edit and preview, edit row, labels, own controls, disabled, lift"
            }
            Detail::MetaParent => "Notifies the children that announced an action",
            Detail::LineChart => "Lines and areas: smooth, stepped, stacked, 100%",
            Detail::BarChart => "Grouped, stacked, 100%, horizontal, histogram",
            Detail::PieChart => "Parts of a whole, as a pie or a donut",
            Detail::DonutChart => "Rings with the total in the middle",
            Detail::BubbleChart => "Two axes and a size",
            Detail::ComboChart => "Bars, lines and areas on one scale",
            Detail::Funnel => "Stages that narrow",
            Detail::Sankey => "Flows between nodes",
            Detail::Sunburst => "A tree as rings",
            Detail::VirtualList => "Fixed-height rows, only those in view are built",
            Detail::EditableText => "Clamped text that swaps to an entry",
            Detail::PageHeader => "Cover, title, facts and actions, wide or narrow",
            Detail::Icons => "The built-in symbolic icons",
            Detail::IconToggle => "One icon that is on or off",
            Detail::ClipWindow => "A window that moves its content instead of squeezing it",
            Detail::Surface => "Page surfaces with the gutter in one place",
            Detail::FlowGrid => "Equal items that flow into columns, or a list in one",
            Detail::TableGrid => "Cells of different widths on a 12-column grid",
            Detail::SectionHeading => "A heading with a link at the far end",
            Detail::ItemRow => "A row of a music, podcast or file list, with its sort header",
            Detail::ChipGroup => "Wrapping toggle chips, filtered as you type",
            Detail::History => "Where Back goes: open, go and back",
            Detail::Container => "Centered content at a named maximum width",
            Detail::Composed => "Several components editing one shared state, as one widget",
            Detail::SyncedText => "Timed lines with the current one highlighted",
            Detail::ThemeColors => "Scheme, accent, density and a card color override",
            Detail::ColorHelpers => "Seeded colors and a contrast floor",
            Detail::ThemeConfig => "Hardcoded, app theme, toolkit theme: layered and regenerable",
            Detail::ThemeShape => "Control, card and toast radii, shadows, with a live preview",
            Detail::ThemeMotion => "Animations on or off, easings and springs",
            Detail::ThemeWindow => "Menu presentation, mobile sidebar and the path bar title",
        }
    }
}
