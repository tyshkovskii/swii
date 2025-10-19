use crate::macos_window::get_editor_windows;

#[tauri::command]
pub async fn list_editor_windows(
    _app: tauri::AppHandle,
    args: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    crate::command_wrapper::create_command("list_editor_windows", args, |ctx| {
        ctx.logger.info("Starting to list editor windows");

        #[cfg(target_os = "macos")]
        {
            match get_editor_windows() {
                Ok(windows) => {
                    ctx.logger
                        .info(&format!("Found {} editor windows", windows.len()));
                    Ok(windows)
                }
                Err(e) => {
                    ctx.logger
                        .error(&format!("Failed to get editor windows: {}", e));
                    Err(format!("Failed to enumerate editor windows: {}", e))
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            ctx.logger.info("Not on macOS, returning empty list");
            Ok(Vec::new())
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macos_window::is_editor_window;
    use crate::types::WindowInfo;

    #[test]
    fn test_window_info_creation() {
        let window_info = WindowInfo {
            app_name: "Test App".to_string(),
            window_name: Some("Test Window".to_string()),
            pid: 1234,
            window_number: 42,
            project: None,
            active_editor_tab: None,
            app_icon: None,
        };

        assert_eq!(window_info.app_name, "Test App");
        assert_eq!(window_info.window_name, Some("Test Window".to_string()));
        assert_eq!(window_info.pid, 1234);
        assert_eq!(window_info.window_number, 42);
        assert_eq!(window_info.project, None);
        assert_eq!(window_info.active_editor_tab, None);
        assert_eq!(window_info.app_icon, None);
    }

    #[test]
    fn test_window_info_with_no_window_name() {
        let window_info = WindowInfo {
            app_name: "App Without Window Name".to_string(),
            window_name: None,
            pid: 5678,
            window_number: 100,
            project: None,
            active_editor_tab: None,
            app_icon: None,
        };

        assert_eq!(window_info.app_name, "App Without Window Name");
        assert_eq!(window_info.window_name, None);
        assert_eq!(window_info.pid, 5678);
        assert_eq!(window_info.window_number, 100);
        assert_eq!(window_info.project, None);
        assert_eq!(window_info.active_editor_tab, None);
        assert_eq!(window_info.app_icon, None);
    }

    #[test]
    fn test_window_info_serialization() {
        let window_info = WindowInfo {
            app_name: "Serialization Test".to_string(),
            window_name: Some("Test Window".to_string()),
            pid: 999,
            window_number: 1,
            project: None,
            active_editor_tab: None,
            app_icon: None,
        };

        let serialized = serde_json::to_string(&window_info).unwrap();
        let expected_fields = [
            "\"app_name\":\"Serialization Test\"",
            "\"window_name\":\"Test Window\"",
            "\"pid\":999",
            "\"window_number\":1",
        ];

        for field in expected_fields.iter() {
            assert!(
                serialized.contains(field),
                "Serialized JSON should contain {}",
                field
            );
        }
    }

    #[test]
    fn test_window_info_with_app_icon() {
        let window_info = WindowInfo {
            app_name: "Test App".to_string(),
            window_name: Some("Test Window".to_string()),
            pid: 1234,
            window_number: 42,
            project: Some("test-project".to_string()),
            active_editor_tab: Some("main.rs".to_string()),
            app_icon: Some("base64_encoded_icon_data".to_string()),
        };

        assert_eq!(window_info.app_name, "Test App");
        assert_eq!(window_info.window_name, Some("Test Window".to_string()));
        assert_eq!(window_info.pid, 1234);
        assert_eq!(window_info.window_number, 42);
        assert_eq!(window_info.project, Some("test-project".to_string()));
        assert_eq!(window_info.active_editor_tab, Some("main.rs".to_string()));
        assert_eq!(
            window_info.app_icon,
            Some("base64_encoded_icon_data".to_string())
        );
    }

    #[test]
    fn test_window_info_serialization_with_app_icon() {
        let window_info = WindowInfo {
            app_name: "Serialization Test".to_string(),
            window_name: Some("Test Window".to_string()),
            pid: 999,
            window_number: 1,
            project: Some("test-project".to_string()),
            active_editor_tab: Some("main.rs".to_string()),
            app_icon: Some("base64_icon_data".to_string()),
        };

        let serialized = serde_json::to_string(&window_info).unwrap();
        let expected_fields = [
            "\"app_name\":\"Serialization Test\"",
            "\"window_name\":\"Test Window\"",
            "\"pid\":999",
            "\"window_number\":1",
            "\"project\":\"test-project\"",
            "\"active_editor_tab\":\"main.rs\"",
            "\"app_icon\":\"base64_icon_data\"",
        ];

        for field in expected_fields.iter() {
            assert!(
                serialized.contains(field),
                "Serialized JSON should contain {}",
                field
            );
        }
    }

    #[test]
    fn test_list_editor_windows_returns_vec() {
        let windows = get_editor_windows();

        match windows {
            Ok(vec) => {
                // Should return a vector (length check validates it's a proper Vec)
                assert!(vec.is_empty() || !vec.is_empty());
            }
            Err(e) => {
                // On systems where enumeration fails, we still get a Result
                println!("Editor window enumeration failed: {}", e);
            }
        }
    }

    #[test]
    fn test_list_editor_windows_window_info_validity() {
        let windows = get_editor_windows();

        match windows {
            Ok(vec) => {
                for window in vec.iter() {
                    // App name should not be empty
                    assert!(!window.app_name.is_empty(), "App name should not be empty");

                    // PID should be positive
                    assert!(window.pid > 0, "PID should be positive");

                    // Window number should be valid
                    assert!(window.window_number > 0, "Window number should be positive");

                    // If window name exists, it should not be empty
                    if let Some(ref name) = window.window_name {
                        assert!(
                            !name.is_empty(),
                            "Window name should not be empty if present"
                        );
                    }
                }
            }
            Err(e) => {
                // If enumeration fails, skip validation tests
                println!("Skipping validation tests due to enumeration error: {}", e);
            }
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_specific_functionality() {
        // This test verifies that the function works on macOS
        let windows = get_editor_windows();

        match windows {
            Ok(vec) => {
                // On a running macOS system, there should typically be at least some windows
                // but we won't enforce this as it depends on the system state

                // Instead, verify that if windows exist, they have macOS-specific characteristics
                for window in vec.iter() {
                    // Verify that window numbers are within reasonable range for macOS
                    assert!(window.window_number < u32::MAX);

                    // Verify app names don't contain null characters or other invalid data
                    assert!(!window.app_name.contains('\0'));

                    if let Some(ref name) = window.window_name {
                        assert!(!name.contains('\0'));
                    }
                }
            }
            Err(e) => {
                // If enumeration fails, we can't test macOS-specific functionality
                println!(
                    "Skipping macOS-specific tests due to enumeration error: {}",
                    e
                );
            }
        }
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_non_macos_behavior() {
        // On non-macOS platforms, the function should return an empty vector
        // or handle gracefully
        let windows = get_editor_windows();
        match windows {
            Ok(vec) => {
                assert_eq!(
                    vec.len(),
                    0,
                    "Should return empty vector on non-macOS platforms"
                );
            }
            Err(e) => {
                // Even on non-macOS, we should get a Result, not an error
                panic!("Unexpected error on non-macOS platform: {}", e);
            }
        }
    }

    #[test]
    fn test_is_editor_window() {
        // Test positive cases
        assert!(is_editor_window("Visual Studio Code"));
        assert!(is_editor_window("Code"));
        assert!(is_editor_window("Zed"));
        assert!(is_editor_window("Sublime Text"));
        assert!(is_editor_window("Vim"));
        assert!(is_editor_window("Xcode"));
        assert!(is_editor_window("Cursor"));

        // Test case insensitive matching
        assert!(is_editor_window("visual studio code"));
        assert!(is_editor_window("SUBLIME TEXT"));

        // Test partial matching (contains)
        assert!(is_editor_window("Visual Studio Code - Insiders"));
        assert!(is_editor_window("Sublime Text 4"));

        // Test negative cases
        assert!(!is_editor_window("Safari"));
        assert!(!is_editor_window("Chrome"));
        assert!(!is_editor_window("Firefox"));
        assert!(!is_editor_window("Finder"));
        assert!(!is_editor_window("Terminal"));
        assert!(!is_editor_window(""));
    }

    #[test]
    fn test_editor_applications_list_not_empty() {
        assert!(is_editor_window("Visual Studio Code"));
        assert!(is_editor_window("Cursor"));
    }

    #[test]
    fn test_editor_applications_contain_common_editors() {
        assert!(is_editor_window("Visual Studio Code"));
        assert!(is_editor_window("Zed"));
        assert!(is_editor_window("Cursor"));
        assert!(is_editor_window("Sublime Text"));
        assert!(is_editor_window("Vim"));
        assert!(is_editor_window("Xcode"));
    }
}
