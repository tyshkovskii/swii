//! Configuration for editor applications and their properties.
//!
//! This module centralizes editor-related data that is used across multiple
//! parts of the application, including window detection, path mapping, and
//! title parsing.

/// List of supported code editor applications.
/// This list is used for identifying editor windows and filtering them from
/// other application windows.
pub const EDITOR_APPLICATIONS: &[&str] = &[
    "Visual Studio Code",
    "Code",
    "VSCode",
    "Zed",
    "Sublime Text",
    "Sublime Text 3",
    "Sublime Text 4",
    "Atom",
    "Vim",
    "MacVim",
    "Neovim",
    "Emacs",
    "GNU Emacs",
    "IntelliJ IDEA",
    "PyCharm",
    "WebStorm",
    "PhpStorm",
    "RubyMine",
    "CLion",
    "GoLand",
    "DataGrip",
    "Rider",
    "Android Studio",
    "Xcode",
    "TextEdit",
    "TextMate",
    "Brackets",
    "Nova",
    "CotEditor",
    "BBEdit",
    "Nano",
    "Cursor",
    "Fleet",
    "Helix",
];

/// Mapping of editor application names to their typical installation paths on macOS.
/// This is used as a fallback when trying to determine the application bundle path
/// for a given process ID.
pub const EDITOR_PATHS: &[(&str, &str)] = &[
    ("Visual Studio Code", "/Applications/Visual Studio Code.app"),
    ("Code", "/Applications/Visual Studio Code.app"),
    ("VSCode", "/Applications/Visual Studio Code.app"),
    ("Zed", "/Applications/Zed.app"),
    ("Sublime Text", "/Applications/Sublime Text.app"),
    ("Sublime Text 3", "/Applications/Sublime Text.app"),
    ("Sublime Text 4", "/Applications/Sublime Text.app"),
    ("Atom", "/Applications/Atom.app"),
    ("Cursor", "/Applications/Cursor.app"),
    ("Xcode", "/Applications/Xcode.app"),
    ("IntelliJ IDEA", "/Applications/IntelliJ IDEA.app"),
    ("PyCharm", "/Applications/PyCharm.app"),
    ("WebStorm", "/Applications/WebStorm.app"),
];

/// Returns true if the given application name is a known code editor.
/// This function performs both exact case-insensitive matches and substring matches
/// to handle editor variants like "Visual Studio Code - Insiders".
pub fn is_editor_application(app_name: &str) -> bool {
    EDITOR_APPLICATIONS
        .iter()
        .any(|&editor| app_name.eq_ignore_ascii_case(editor) || app_name.contains(editor))
}

/// Gets the typical installation path for an editor application.
pub fn get_editor_path(app_name: &str) -> Option<&'static str> {
    EDITOR_PATHS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(app_name))
        .map(|(_, path)| *path)
}

/// Checks if a title contains any known editor name.
/// This is useful for title parsing logic to avoid false positives.
pub fn title_contains_editor(title: &str) -> bool {
    EDITOR_APPLICATIONS
        .iter()
        .any(|&editor| title.contains(editor))
}

/// Checks if a title contains any IntelliJ family editor.
pub fn title_contains_intellij_family(title: &str) -> bool {
    title.contains("IntelliJ")
        || title.contains("PyCharm")
        || title.contains("WebStorm")
        || title.contains("PhpStorm")
        || title.contains("RubyMine")
        || title.contains("CLion")
        || title.contains("GoLand")
        || title.contains("DataGrip")
        || title.contains("Rider")
}

/// Checks if a title contains "Visual Studio Code" or "Cursor".
pub fn title_contains_vscode_or_cursor(title: &str) -> bool {
    title.contains("Visual Studio Code") || title.contains("Cursor")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_editor_application() {
        // Test known editors
        assert!(is_editor_application("Visual Studio Code"));
        assert!(is_editor_application("Code"));
        assert!(is_editor_application("Zed"));
        assert!(is_editor_application("Cursor"));
        assert!(is_editor_application("Sublime Text"));

        // Test case insensitive
        assert!(is_editor_application("visual studio code"));
        assert!(is_editor_application("VISUAL STUDIO CODE"));

        // Test substring matching for variants
        assert!(is_editor_application("Visual Studio Code - Insiders"));
        assert!(is_editor_application("Sublime Text 4"));
        assert!(is_editor_application("Code - OSS"));

        // Test non-editors
        assert!(!is_editor_application("Safari"));
        assert!(!is_editor_application("Chrome"));
        assert!(!is_editor_application("Finder"));
    }

    #[test]
    fn test_get_editor_path() {
        assert_eq!(
            get_editor_path("Visual Studio Code"),
            Some("/Applications/Visual Studio Code.app")
        );
        assert_eq!(
            get_editor_path("Code"),
            Some("/Applications/Visual Studio Code.app")
        );
        assert_eq!(get_editor_path("Zed"), Some("/Applications/Zed.app"));
        assert_eq!(get_editor_path("Unknown Editor"), None);
    }
}

