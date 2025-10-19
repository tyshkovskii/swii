use std::path::Path;

use crate::editor_config;

// Constants for title parsing
pub mod constants {
    // Title separators
    pub const EM_DASH_SEPARATOR: &str = " — ";
    pub const REGULAR_DASH_SEPARATOR: &str = " - ";

    // File path prefixes
    pub const FILE_URL_PREFIX: &str = "file://";

    // Command patterns for terminal/build output tabs
    pub const COMMAND_PATTERNS: &[&str] = &[
        " run ", " dev", " build", " test", " start", "npm ", "yarn ", "bun ", "cargo ", "pnpm ",
    ];

    // Valid file extension length range
    pub const MIN_EXTENSION_LENGTH: usize = 2;
    pub const MAX_EXTENSION_LENGTH: usize = 4;

    // Minimum meaningful project name length
    pub const MIN_PROJECT_NAME_LENGTH: usize = 1;

    // Maximum length difference for heuristic detection
    pub const HEURISTIC_LENGTH_THRESHOLD: usize = 5;
}

/// Extracts project name from window title using various editor-specific patterns
pub fn extract_project_from_title(title: &str) -> Option<String> {
    if title.is_empty() {
        return None;
    }

    // Em dash formats: Handle different patterns
    if title.contains(constants::EM_DASH_SEPARATOR) {
        let parts: Vec<&str> = title.split(constants::EM_DASH_SEPARATOR).collect();
        if parts.len() == 2 {
            let first_part = parts[0].trim();
            let second_part = parts[1].trim();

            // Case 1: "ProjectName — filename.ext" (Zed style)
            if second_part.contains('.') && !first_part.contains('/') && !first_part.is_empty() {
                return Some(first_part.to_string());
            }

            // Case 2: "command — ProjectName" (Cursor/VS Code style)
            if !second_part.contains('/') && !second_part.contains('.') && !second_part.is_empty() {
                return Some(second_part.to_string());
            }
        }
    }

    // VS Code / Cursor format with regular dash: "filename.ext - ProjectName - Visual Studio Code"
    if editor_config::title_contains_vscode_or_cursor(title) {
        let parts: Vec<&str> = title.split(constants::REGULAR_DASH_SEPARATOR).collect();
        if parts.len() >= 3 {
            // Second part should be the project name
            let project_name = parts[1].trim();
            if !project_name.is_empty() && !project_name.contains('/') {
                return Some(project_name.to_string());
            }
        }
    }

    // Xcode format: "ProjectName" or "ProjectName - filename.ext"
    // Only apply Xcode logic if we detect this is actually from Xcode or has Xcode-like pattern
    if title.contains("Xcode") {
        let project_name = if let Some(dash_pos) = title.find(constants::REGULAR_DASH_SEPARATOR) {
            &title[..dash_pos]
        } else {
            title
        };

        let cleaned = project_name.trim();
        if !cleaned.is_empty() && !cleaned.contains("Xcode") && !cleaned.contains('/') {
            return Some(cleaned.to_string());
        }
    }

    // Handle potential Xcode-style titles (ProjectName - filename) without "Xcode" in title
    if title.contains(constants::REGULAR_DASH_SEPARATOR)
        && !editor_config::title_contains_editor(title)
    {
        let parts: Vec<&str> = title.split(constants::REGULAR_DASH_SEPARATOR).collect();
        if parts.len() == 2 {
            let potential_project = parts[0].trim();
            let potential_file = parts[1].trim();

            // If second part looks like a filename, first part might be project
            if potential_file.contains('.') && !potential_project.contains('/') {
                return Some(potential_project.to_string());
            }
        }
    }

    // Zed format: Often just "ProjectName" (single word/phrase without common editor names)
    if !title.contains(constants::REGULAR_DASH_SEPARATOR)
        && !title.contains('/')
        && !title.contains('.')
        && !editor_config::title_contains_editor(title)
        && !title.contains("Chrome")
        && !title.contains("Safari")
        && !title.contains("Firefox")
    {
        let cleaned = title.trim();
        if !cleaned.is_empty() && cleaned.len() > 1 {
            return Some(cleaned.to_string());
        }
    }

    // IntelliJ family format: "ProjectName [path] - IntelliJ IDEA"
    if editor_config::title_contains_intellij_family(title)
    {
        if let Some(bracket_pos) = title.find('[') {
            let project_name = title[..bracket_pos].trim();
            if !project_name.is_empty() {
                return Some(project_name.to_string());
            }
        }
    }

    // Sublime Text format: "filename.ext - Sublime Text"
    if title.contains("Sublime Text") {
        let parts: Vec<&str> = title.split(constants::REGULAR_DASH_SEPARATOR).collect();
        if parts.len() >= 2 {
            let file_part = parts[0].trim();
            // Try to extract project from filename path
            if file_part.contains('/') {
                return extract_project_from_file_path(file_part);
            }
        }
    }

    // Generic fallback: if title contains a path, try to extract project from it
    if title.contains('/') {
        return extract_project_from_file_path(title);
    }

    None
}

/// Determines which part is the project name and which is the tab name based on heuristics
pub fn determine_project_and_tab_from_parts<'a>(
    first_part: &'a str,
    second_part: &'a str,
) -> (&'a str, &'a str) {
    // Heuristic 1: Check for file extensions (tab names often have file extensions)
    let first_has_extension = has_file_extension(first_part);
    let second_has_extension = has_file_extension(second_part);

    if first_has_extension && !second_has_extension {
        // First part is likely a filename (tab), second part is likely project
        return (second_part, first_part);
    }

    if second_has_extension && !first_has_extension {
        // Second part is likely a filename (tab), first part is likely project
        return (first_part, second_part);
    }

    // Heuristic 2: Check for command-like patterns (tab names for terminal/command output)
    let first_is_command = is_command_like(first_part);
    let second_is_command = is_command_like(second_part);

    if first_is_command && !second_is_command {
        // First part is likely a command (tab), second part is likely project
        return (second_part, first_part);
    }

    if second_is_command && !first_is_command {
        // Second part is likely a command (tab), first part is likely project
        return (first_part, second_part);
    }

    // Heuristic 3: Length and complexity (project names tend to be shorter and simpler)
    // Only apply this heuristic if there's a significant length difference (at least 5 characters)
    if first_part.len() + constants::HEURISTIC_LENGTH_THRESHOLD <= second_part.len()
        && !first_part.contains('/')
        && !first_part.contains(' ')
    {
        // First part is likely the project name (much shorter, no path separators)
        return (first_part, second_part);
    }

    if second_part.len() + constants::HEURISTIC_LENGTH_THRESHOLD <= first_part.len()
        && !second_part.contains('/')
        && !second_part.contains(' ')
    {
        // Second part is likely the project name (much shorter, no path separators)
        return (second_part, first_part);
    }

    // Default fallback: assume first part is project, second part is tab
    // This handles the common Zed pattern: "project — filename"
    (first_part, second_part)
}

/// Checks if a text string has a valid file extension
pub fn has_file_extension(text: &str) -> bool {
    if let Some(dot_pos) = text.rfind('.') {
        let extension = &text[dot_pos + 1..];
        // Check if it looks like a valid file extension (2-4 chars, alphanumeric)
        extension.len() >= constants::MIN_EXTENSION_LENGTH
            && extension.len() <= constants::MAX_EXTENSION_LENGTH
            && extension.chars().all(|c| c.is_alphanumeric())
    } else {
        false
    }
}

/// Checks if a text string looks like a command or terminal output
pub fn is_command_like(text: &str) -> bool {
    // Check for common command patterns
    constants::COMMAND_PATTERNS
        .iter()
        .any(|pattern| text.contains(pattern))
        || (text.contains(' ') && text.len() > 10) // Longer phrases with spaces are likely commands
}

/// Extracts both project name and tab name from a window title
pub fn extract_project_and_tab_from_title(title: &str) -> (Option<String>, Option<String>) {
    if title.is_empty() {
        return (None, None);
    }

    // Em dash formats: Handle different patterns
    if title.contains(constants::EM_DASH_SEPARATOR) {
        let parts: Vec<&str> = title.split(constants::EM_DASH_SEPARATOR).collect();
        if parts.len() == 2 {
            let first_part = parts[0].trim();
            let second_part = parts[1].trim();

            // Validate that both parts are meaningful (not just symbols or empty)
            if !first_part.is_empty()
                && !second_part.is_empty()
                && first_part.len() > constants::MIN_PROJECT_NAME_LENGTH
                && second_part.len() > constants::MIN_PROJECT_NAME_LENGTH
                && !first_part
                    .chars()
                    .all(|c| c.is_ascii_punctuation() || c == '—' || c == '–')
                && !second_part
                    .chars()
                    .all(|c| c.is_ascii_punctuation() || c == '—' || c == '–')
                && first_part != "—"
                && first_part != "–"
                && first_part != "-"
                && second_part != "—"
                && second_part != "–"
                && second_part != "-"
            {
                // Smart detection: determine which part is project vs tab based on content
                let (project, tab) = determine_project_and_tab_from_parts(first_part, second_part);
                return (Some(project.to_string()), Some(tab.to_string()));
            }
        }
    }

    // VS Code / Cursor format with regular dash: "filename.ext - ProjectName - Visual Studio Code"
    if editor_config::title_contains_vscode_or_cursor(title) {
        let parts: Vec<&str> = title.split(constants::REGULAR_DASH_SEPARATOR).collect();
        if parts.len() >= 3 {
            let tab_name = parts[0].trim();
            let project_name = parts[1].trim();
            if !tab_name.is_empty() && !project_name.is_empty() && !project_name.contains('/') {
                return (Some(project_name.to_string()), Some(tab_name.to_string()));
            }
        }
    }

    // Fallback to just project extraction (no tab info available)
    if let Some(project) = extract_project_from_title(title) {
        // Validate that the extracted project is meaningful
        // Exclude single characters, punctuation, and known invalid patterns
        if project.len() > constants::MIN_PROJECT_NAME_LENGTH
            && !project
                .chars()
                .all(|c| c.is_ascii_punctuation() || c == '—' || c == '–')
            && !project.contains('—')
            && !project.contains('–')
            && !project.ends_with(' ')
            && !project.starts_with(' ')
            && project != "—"
            && project != "–"
            && project != "-"
        {
            return (Some(project), None);
        }
    }

    (None, None)
}

/// Extracts project name from a file path by looking for meaningful directory names
pub fn extract_project_from_file_path(path: &str) -> Option<String> {
    let path_parts: Vec<&str> = path.split('/').collect();

    // Look for common project indicators in the path
    for (i, part) in path_parts.iter().enumerate() {
        if *part == "src" || *part == "lib" || *part == "app" {
            // The directory before src/lib/app might be the project
            if i > 0 {
                let potential_project = path_parts[i - 1];
                if !potential_project.is_empty() && potential_project.len() > 1 {
                    return Some(potential_project.to_string());
                }
            }
        }
    }

    // Fallback: take the last meaningful directory name
    for part in path_parts.iter().rev() {
        if !part.is_empty()
            && !part.starts_with('.')
            && part.len() > 1
            && !part.chars().all(|c| c.is_numeric())
        {
            return Some(part.to_string());
        }
    }

    None
}

/// Extracts project name from a file path by looking for project root indicators
pub fn extract_project_name_from_path(file_path: &str) -> Option<String> {
    // Remove file:// prefix if present
    let clean_path = if file_path.starts_with(constants::FILE_URL_PREFIX) {
        &file_path[constants::FILE_URL_PREFIX.len()..]
    } else {
        file_path
    };

    let path = Path::new(clean_path);

    // Look for common project indicators and extract project name
    let mut current = path;

    // Go up the directory tree looking for project root indicators
    while let Some(parent) = current.parent() {
        current = parent;

        if let Some(dir_name) = current.file_name() {
            let dir_name_str = dir_name.to_string_lossy();

            // Check for common project root indicators
            if current.join("Cargo.toml").exists()
                || current.join("package.json").exists()
                || current.join(".git").exists()
                || current.join("Gemfile").exists()
                || current.join("requirements.txt").exists()
                || current.join("pom.xml").exists()
                || current.join("build.gradle").exists()
                || current.join("composer.json").exists()
                || current.join("go.mod").exists()
                || current.join("Pipfile").exists()
                || current.join("pyproject.toml").exists()
            {
                return Some(dir_name_str.to_string());
            }
        }
    }

    // Fallback: if no project root found, extract meaningful directory name
    // Go up a few levels and take a reasonable directory name
    let mut path_components: Vec<&str> = clean_path.split('/').collect();
    path_components.retain(|&x| !x.is_empty());

    // Look for meaningful project-like directory names
    for (i, component) in path_components.iter().enumerate().rev() {
        // Skip common non-project directories
        if *component == "src"
            || *component == "lib"
            || *component == "bin"
            || *component == "target"
            || *component == "node_modules"
            || component.starts_with('.')
        {
            continue;
        }

        // If we're in a src directory, the parent might be the project
        if i > 0 && (*component == "src" || *component == "lib") {
            if let Some(project_name) = path_components.get(i - 1) {
                return Some(project_name.to_string());
            }
        }

        // Take the first meaningful directory name
        if component.len() > 1 && !component.chars().all(|c| c.is_numeric()) {
            return Some(component.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_project_from_title_vscode() {
        // VS Code format: "filename.ext - ProjectName - Visual Studio Code"
        assert_eq!(
            extract_project_from_title("main.rs - swii - Visual Studio Code"),
            Some("swii".to_string())
        );

        assert_eq!(
            extract_project_from_title("App.tsx - my-project - Visual Studio Code"),
            Some("my-project".to_string())
        );

        // Cursor format (similar to VS Code)
        assert_eq!(
            extract_project_from_title("index.js - awesome-app - Cursor"),
            Some("awesome-app".to_string())
        );

        // Cursor em dash format: "command — ProjectName"
        assert_eq!(
            extract_project_from_title("bun run tauri dev — swii"),
            Some("swii".to_string())
        );
    }

    #[test]
    fn test_extract_project_from_title_xcode() {
        // Xcode format: "ProjectName" or "ProjectName - filename.ext"
        assert_eq!(
            extract_project_from_title("MyiOSApp"),
            Some("MyiOSApp".to_string())
        );

        assert_eq!(
            extract_project_from_title("SwiftUIDemo - ContentView.swift"),
            Some("SwiftUIDemo".to_string())
        );
    }

    #[test]
    fn test_extract_project_from_title_zed() {
        // Zed format: Often just "ProjectName"
        assert_eq!(
            extract_project_from_title("rust-analyzer"),
            Some("rust-analyzer".to_string())
        );

        assert_eq!(
            extract_project_from_title("webserver"),
            Some("webserver".to_string())
        );

        // Zed em dash format: "ProjectName — filename.ext"
        assert_eq!(
            extract_project_from_title("switch — ARCHITECTURE.md"),
            Some("switch".to_string())
        );
    }

    #[test]
    fn test_extract_project_from_title_intellij() {
        // IntelliJ family format: "ProjectName [path] - IntelliJ IDEA"
        assert_eq!(
            extract_project_from_title(
                "my-spring-boot [~/projects/my-spring-boot] - IntelliJ IDEA"
            ),
            Some("my-spring-boot".to_string())
        );

        assert_eq!(
            extract_project_from_title("django-webapp [/Users/dev/django-webapp] - PyCharm"),
            Some("django-webapp".to_string())
        );
    }

    #[test]
    fn test_extract_project_from_title_sublime() {
        // Sublime Text with path in filename
        assert_eq!(
            extract_project_from_title("/Users/dev/my-project/src/main.py - Sublime Text"),
            Some("my-project".to_string())
        );
    }

    #[test]
    fn test_extract_project_from_title_invalid_cases() {
        // Empty title
        assert_eq!(extract_project_from_title(""), None);

        // Just application name
        assert_eq!(extract_project_from_title("Visual Studio Code"), None);

        // No meaningful project name
        assert_eq!(
            extract_project_from_title("Untitled - Visual Studio Code"),
            None
        );
    }

    #[test]
    fn test_extract_project_from_file_path() {
        // Path with src directory
        assert_eq!(
            extract_project_from_file_path("/Users/dev/my-project/src/main.rs"),
            Some("my-project".to_string())
        );

        // Path with lib directory
        assert_eq!(
            extract_project_from_file_path("/home/user/awesome-lib/lib/index.js"),
            Some("awesome-lib".to_string())
        );

        // Path with app directory
        assert_eq!(
            extract_project_from_file_path("/projects/web-app/app/controllers/home.py"),
            Some("web-app".to_string())
        );

        // Simple path without common indicators
        assert_eq!(
            extract_project_from_file_path("/Users/dev/simple-project/file.txt"),
            Some("file.txt".to_string())
        );
    }

    #[test]
    fn test_extract_project_and_tab_from_title_em_dash() {
        // Em dash format: "tab_name — project_name"
        assert_eq!(
            extract_project_and_tab_from_title("bun run tauri dev — swii"),
            (
                Some("swii".to_string()),
                Some("bun run tauri dev".to_string())
            )
        );

        assert_eq!(
            extract_project_and_tab_from_title("eslint.config.mjs — promptbook"),
            (
                Some("promptbook".to_string()),
                Some("eslint.config.mjs".to_string())
            )
        );

        assert_eq!(
            extract_project_and_tab_from_title("commands.rs — switch"),
            (Some("switch".to_string()), Some("commands.rs".to_string()))
        );

        assert_eq!(
            extract_project_and_tab_from_title("switch — ARCHITECTURE.md"),
            (
                Some("switch".to_string()),
                Some("ARCHITECTURE.md".to_string())
            )
        );
    }

    #[test]
    fn test_extract_project_and_tab_from_title_vscode_cursor() {
        // VS Code/Cursor format: "filename.ext - ProjectName - Visual Studio Code"
        assert_eq!(
            extract_project_and_tab_from_title("main.rs - swii - Visual Studio Code"),
            (Some("swii".to_string()), Some("main.rs".to_string()))
        );

        assert_eq!(
            extract_project_and_tab_from_title("App.tsx - my-project - Cursor"),
            (Some("my-project".to_string()), Some("App.tsx".to_string()))
        );

        assert_eq!(
            extract_project_and_tab_from_title("index.js - awesome-app - Visual Studio Code"),
            (
                Some("awesome-app".to_string()),
                Some("index.js".to_string())
            )
        );
    }

    #[test]
    fn test_extract_project_and_tab_from_title_fallback() {
        // Fallback to just project extraction when tab parsing fails
        assert_eq!(
            extract_project_and_tab_from_title("MyiOSApp"),
            (Some("MyiOSApp".to_string()), None)
        );

        // Should return None for both when nothing can be extracted
        assert_eq!(extract_project_and_tab_from_title(""), (None, None));

        assert_eq!(
            extract_project_and_tab_from_title("Visual Studio Code"),
            (None, None)
        );
    }

    #[test]
    fn test_extract_project_and_tab_from_title_edge_cases() {
        // Empty parts should be handled gracefully
        assert_eq!(extract_project_and_tab_from_title(" — "), (None, None));

        assert_eq!(
            extract_project_and_tab_from_title("something — "),
            (None, None)
        );

        // Malformed but recoverable: " — project" can extract project from fallback
        assert_eq!(
            extract_project_and_tab_from_title(" — project"),
            (Some("project".to_string()), None)
        );

        // Single em dash without spaces
        assert_eq!(
            extract_project_and_tab_from_title("file—project"),
            (None, None)
        );

        // Completely empty
        assert_eq!(extract_project_and_tab_from_title(""), (None, None));

        // Only punctuation
        assert_eq!(extract_project_and_tab_from_title("—"), (None, None));
    }

    #[test]
    fn test_determine_project_and_tab_from_parts() {
        // Test file extension detection
        assert_eq!(
            determine_project_and_tab_from_parts("switch", "ARCHITECTURE.md"),
            ("switch", "ARCHITECTURE.md")
        );

        assert_eq!(
            determine_project_and_tab_from_parts("commands.rs", "switch"),
            ("switch", "commands.rs")
        );

        assert_eq!(
            determine_project_and_tab_from_parts("promptbook", "eslint.config.mjs"),
            ("promptbook", "eslint.config.mjs")
        );

        // Test command detection
        assert_eq!(
            determine_project_and_tab_from_parts("bun run tauri dev", "swii"),
            ("swii", "bun run tauri dev")
        );

        assert_eq!(
            determine_project_and_tab_from_parts("npm start", "my-project"),
            ("my-project", "npm start")
        );

        // Test length heuristic (requires significant length difference)
        assert_eq!(
            determine_project_and_tab_from_parts("app", "some-really-long-filename-or-command"),
            ("app", "some-really-long-filename-or-command")
        );

        // Test default fallback
        assert_eq!(
            determine_project_and_tab_from_parts("project", "tab"),
            ("project", "tab")
        );
    }

    #[test]
    fn test_has_file_extension() {
        assert_eq!(has_file_extension("file.txt"), true);
        assert_eq!(has_file_extension("ARCHITECTURE.md"), true);
        assert_eq!(has_file_extension("commands.rs"), true);
        assert_eq!(has_file_extension("eslint.config.mjs"), true);
        assert_eq!(has_file_extension("package.json"), true);

        assert_eq!(has_file_extension("project"), false);
        assert_eq!(has_file_extension("switch"), false);
        assert_eq!(has_file_extension("no-extension"), false);
        assert_eq!(has_file_extension("file."), false);
        assert_eq!(has_file_extension("file.toolongext"), false);
        assert_eq!(has_file_extension("file.x"), false);
    }

    #[test]
    fn test_is_command_like() {
        assert_eq!(is_command_like("bun run tauri dev"), true);
        assert_eq!(is_command_like("npm start"), true);
        assert_eq!(is_command_like("yarn build"), true);
        assert_eq!(is_command_like("cargo test"), true);
        assert_eq!(is_command_like("pnpm dev"), true);
        assert_eq!(
            is_command_like("this is a very long command description"),
            true
        );

        assert_eq!(is_command_like("project"), false);
        assert_eq!(is_command_like("switch"), false);
        assert_eq!(is_command_like("file.txt"), false);
        assert_eq!(is_command_like("short"), false);
    }
}
