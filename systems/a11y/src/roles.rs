//! Roles: ARIA's and each element's own, the names Chrome reports them by, and which are named by their content.

use accesskit::Role;

use super::*;

/// ARIA roles as AccessKit's.
pub(crate) fn aria_role<K>(r: &str, el: &El<K>) -> Option<Role> {
    Some(match r {
        "alert" => Role::Alert,
        "alertdialog" => Role::AlertDialog,
        "application" => Role::Application,
        "article" => Role::Article,
        "banner" => Role::Banner,
        "blockquote" => Role::Blockquote,
        "button" => Role::Button,
        "caption" => Role::Caption,
        "cell" => Role::Cell,
        "checkbox" => Role::CheckBox,
        "code" => Role::Code,
        "columnheader" => Role::ColumnHeader,
        "combobox" => Role::ComboBox,
        "complementary" => Role::Complementary,
        "contentinfo" => Role::ContentInfo,
        "definition" => Role::Definition,
        "deletion" => Role::ContentDeletion,
        "dialog" => Role::Dialog,
        "document" => Role::Document,
        "emphasis" => Role::Emphasis,
        "feed" => Role::Feed,
        "figure" => Role::Figure,
        "form" => Role::Form,
        "generic" | "none" | "presentation" => Role::GenericContainer,
        "grid" => Role::Grid,
        "gridcell" => Role::GridCell,
        "group" => Role::Group,
        "heading" => Role::Heading,
        "img" | "image" => Role::Image,
        "insertion" => Role::ContentInsertion,
        "link" => Role::Link,
        "list" => Role::List,
        "listbox" => Role::ListBox,
        "listitem" => Role::ListItem,
        "log" => Role::Log,
        "main" => Role::Main,
        "marquee" => Role::Marquee,
        "math" => Role::Math,
        "menu" => Role::Menu,
        "menubar" => Role::MenuBar,
        "menuitem" => Role::MenuItem,
        "menuitemcheckbox" => Role::MenuItemCheckBox,
        "menuitemradio" => Role::MenuItemRadio,
        "meter" => Role::Meter,
        "navigation" => Role::Navigation,
        "note" => Role::Note,
        "option" => Role::ListBoxOption,
        "paragraph" => Role::Paragraph,
        "progressbar" => Role::ProgressIndicator,
        "radio" => Role::RadioButton,
        "radiogroup" => Role::RadioGroup,
        "region" => Role::Region,
        "row" => Role::Row,
        "rowgroup" => Role::RowGroup,
        "rowheader" => Role::RowHeader,
        "scrollbar" => Role::ScrollBar,
        "search" => Role::Search,
        "searchbox" => Role::SearchInput,
        "separator" => Role::Splitter,
        "slider" => Role::Slider,
        "spinbutton" => Role::SpinButton,
        "status" => Role::Status,
        "strong" => Role::Strong,
        "switch" => Role::Switch,
        "tab" => Role::Tab,
        "table" => Role::Table,
        "tablist" => Role::TabList,
        "tabpanel" => Role::TabPanel,
        "term" => Role::Term,
        "textbox" => {
            if truthy(el.p("aria-multiline")) {
                Role::MultilineTextInput
            } else {
                Role::TextInput
            }
        }
        "time" => Role::Time,
        "timer" => Role::Timer,
        "toolbar" => Role::Toolbar,
        "tooltip" => Role::Tooltip,
        "tree" => Role::Tree,
        "treegrid" => Role::TreeGrid,
        "treeitem" => Role::TreeItem,
        _ => return None,
    })
}

/// AccessKit's roles as Chrome names them (CDP's AXNode role), for the parity records.
pub fn chrome_role(r: Role) -> &'static str {
    match r {
        Role::Alert => "alert",
        Role::AlertDialog => "alertdialog",
        Role::Application => "application",
        Role::Article => "article",
        Role::Banner => "banner",
        Role::Blockquote => "blockquote",
        Role::Button | Role::DefaultButton => "button",
        Role::Caption => "caption",
        Role::Cell => "cell",
        Role::CheckBox => "checkbox",
        Role::Code => "code",
        Role::ColumnHeader => "columnheader",
        Role::ComboBox | Role::EditableComboBox => "combobox",
        Role::Complementary => "complementary",
        Role::ContentInfo => "contentinfo",
        Role::Definition => "definition",
        Role::ContentDeletion => "deletion",
        Role::Dialog => "dialog",
        Role::DisclosureTriangle => "DisclosureTriangle",
        Role::Document => "document",
        Role::Emphasis => "emphasis",
        Role::Feed => "feed",
        Role::Figure => "figure",
        Role::Form => "form",
        Role::Grid => "grid",
        Role::GridCell => "gridcell",
        Role::Group | Role::Details => "group",
        Role::Heading => "heading",
        Role::Image | Role::SvgRoot => "image",
        Role::ContentInsertion => "insertion",
        Role::Label => "LabelText",
        Role::Link => "link",
        Role::List => "list",
        Role::ListBox => "listbox",
        Role::ListItem => "listitem",
        Role::ListBoxOption | Role::MenuListOption => "option",
        Role::Log => "log",
        Role::Main => "main",
        Role::Marquee => "marquee",
        Role::Math => "math",
        Role::Menu => "menu",
        Role::MenuBar => "menubar",
        Role::MenuItem => "menuitem",
        Role::MenuItemCheckBox => "menuitemcheckbox",
        Role::MenuItemRadio => "menuitemradio",
        Role::MenuListPopup => "MenuListPopup",
        Role::Meter => "meter",
        Role::Navigation => "navigation",
        Role::Note => "note",
        Role::Paragraph => "paragraph",
        Role::ProgressIndicator => "progressbar",
        Role::RadioButton => "radio",
        Role::RadioGroup => "radiogroup",
        Role::Region => "region",
        Role::Row => "row",
        Role::RowGroup => "rowgroup",
        Role::RowHeader => "rowheader",
        Role::ScrollBar => "scrollbar",
        Role::Search => "search",
        Role::SearchInput => "searchbox",
        Role::Splitter => "separator",
        Role::Slider => "slider",
        Role::SpinButton | Role::NumberInput => "spinbutton",
        Role::Status => "status",
        Role::Strong => "strong",
        Role::Switch => "switch",
        Role::Tab => "tab",
        Role::Table => "table",
        Role::TabList => "tablist",
        Role::TabPanel => "tabpanel",
        Role::Term => "term",
        Role::TextInput | Role::MultilineTextInput | Role::PasswordInput | Role::EmailInput | Role::PhoneNumberInput | Role::UrlInput => "textbox",
        Role::Time => "time",
        Role::Timer => "timer",
        Role::Toolbar => "toolbar",
        Role::Tooltip => "tooltip",
        Role::Tree => "tree",
        Role::TreeGrid => "treegrid",
        Role::TreeItem => "treeitem",
        Role::TextRun => "StaticText",
        Role::Video => "video",
        Role::Audio => "audio",
        Role::Canvas => "canvas",
        _ => "generic",
    }
}

/// Roles named by their content (accname's "name from content").
pub(crate) fn from_content(r: Role) -> bool {
    matches!(
        r,
        Role::Button
            | Role::DefaultButton
            | Role::Link
            | Role::Heading
            | Role::Tab
            | Role::ListBoxOption
            | Role::MenuListOption
            | Role::CheckBox
            | Role::RadioButton
            | Role::Switch
            | Role::MenuItem
            | Role::MenuItemCheckBox
            | Role::MenuItemRadio
            | Role::Cell
            | Role::GridCell
            | Role::ColumnHeader
            | Role::RowHeader
            | Role::Row
            | Role::TreeItem
            | Role::Tooltip
            | Role::DisclosureTriangle
            | Role::Label
    )
}

/// Roles that can't be named (ARIA's "naming prohibited").
pub(crate) fn unnamed(r: Role) -> bool {
    matches!(r, Role::Row | Role::GenericContainer | Role::Paragraph | Role::Caption | Role::Code | Role::ContentDeletion | Role::ContentInsertion | Role::Emphasis | Role::Strong | Role::Label)
}

pub(crate) fn is_field(r: Role) -> bool {
    matches!(r, Role::TextInput | Role::MultilineTextInput | Role::PasswordInput | Role::EmailInput | Role::PhoneNumberInput | Role::UrlInput | Role::SearchInput | Role::SpinButton | Role::NumberInput)
}

pub(crate) const SECTIONING: &[&str] = &["article", "aside", "main", "nav", "section"];
pub(crate) const LABELABLE: &[&str] = &["input", "textarea", "select", "button", "meter", "progress", "output"];
