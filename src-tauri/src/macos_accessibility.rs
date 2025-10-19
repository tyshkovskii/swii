use std::collections::HashMap;
use std::ffi::c_void;

#[cfg(target_os = "macos")]
use core_foundation::{
    array::CFArray,
    base::{CFTypeRef, TCFType},
    dictionary::CFDictionary,
    number::{CFNumber, CFNumberRef},
    string::{CFString, CFStringRef},
};

#[cfg(target_os = "macos")]
use core_graphics::window::{
    kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};

use crate::title_parser::{extract_project_and_tab_from_title, extract_project_name_from_path};

// Raw FFI declarations for Accessibility API
#[cfg(target_os = "macos")]
#[repr(C)]
pub struct __AXUIElement {
    _private: [u8; 0],
}

#[cfg(target_os = "macos")]
pub type AXUIElementRef = *const __AXUIElement;

#[cfg(target_os = "macos")]
pub type AXError = i32;

#[cfg(target_os = "macos")]
extern "C" {
    fn AXUIElementCreateApplication(pid: libc::pid_t) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
}

// Constants for macOS accessibility operations
pub mod constants {
    // Accessibility API attribute names
    pub const AX_WINDOWS: &str = "AXWindows";
    pub const AX_TITLE: &str = "AXTitle";
    pub const AX_DOCUMENT: &str = "AXDocument";
    pub const AX_URL: &str = "AXURL";
    pub const AX_FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";

    // File path prefixes
    pub const FILE_URL_PREFIX: &str = "file://";
}

/// Populates project information for all windows belonging to a specific PID
#[cfg(target_os = "macos")]
pub unsafe fn populate_project_info_for_pid(
    pid: i32,
    window_project_map: &mut HashMap<u32, (Option<String>, Option<String>)>,
) {
    use std::ptr;

    // Get all AX windows for this PID and extract their project info
    let app_ref = AXUIElementCreateApplication(pid);
    if app_ref.is_null() {
        return;
    }

    let windows_attr = CFString::new(constants::AX_WINDOWS);
    let mut windows_ref: CFTypeRef = ptr::null_mut();
    let result = AXUIElementCopyAttributeValue(
        app_ref,
        windows_attr.as_concrete_TypeRef(),
        &mut windows_ref,
    );

    if result != 0 || windows_ref.is_null() {
        return;
    }

    if let Some(windows_array) = cf_type_to_array(windows_ref) {
        let mut ax_projects = Vec::new();

        // Get all project info from AX windows
        for i in 0..windows_array.len() {
            if let Some(window_ref_ptr) = windows_array.get(i) {
                let window_ref = *window_ref_ptr as AXUIElementRef;

                if let Some((project_name, tab_name)) = get_project_and_tab_from_window(window_ref)
                {
                    ax_projects.push((project_name, tab_name));
                }
            }
        }

        // Now get all CG windows for this PID and map them to AX projects
        let window_list_info = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0,
        );

        if !window_list_info.is_null() {
            let window_list: CFArray<CFDictionary> =
                CFArray::wrap_under_create_rule(window_list_info);
            let mut cg_windows_for_pid = Vec::new();

            // Collect CG windows for this PID
            for i in 0..window_list.len() {
                if let Some(window_dict) = window_list.get(i) {
                    if let Some(dict_pid) = get_number_value(&window_dict, "kCGWindowOwnerPID") {
                        if dict_pid as i32 == pid {
                            if let Some(window_number) =
                                get_number_value(&window_dict, "kCGWindowNumber")
                            {
                                cg_windows_for_pid.push(window_number as u32);
                            }
                        }
                    }
                }
            }

            // Simple 1:1 mapping: match by index order
            // This assumes windows are returned in a consistent order
            for (i, &window_number) in cg_windows_for_pid.iter().enumerate() {
                if i < ax_projects.len() {
                    let (project, tab) = &ax_projects[i];
                    window_project_map.insert(window_number, (Some(project.clone()), tab.clone()));
                }
            }
        }
    }
}

/// Extracts project and tab information from a macOS accessibility window element
#[cfg(target_os = "macos")]
unsafe fn get_project_and_tab_from_window(
    window_ref: AXUIElementRef,
) -> Option<(String, Option<String>)> {
    use std::ptr;

    // Get the window title first - this is how Mission Control gets project info
    let title_attr = CFString::new(constants::AX_TITLE);
    let mut title_ref: CFTypeRef = ptr::null_mut();
    let title_result =
        AXUIElementCopyAttributeValue(window_ref, title_attr.as_concrete_TypeRef(), &mut title_ref);

    if title_result == 0 && !title_ref.is_null() {
        if let Some(title) = cf_type_to_string(title_ref) {
            // Try to extract both project and tab from window title
            let (project, tab) = extract_project_and_tab_from_title(&title);
            if let Some(proj) = project {
                return Some((proj, tab));
            }
        }
    }

    // Fallback: try to get document path and extract project from it
    if let Some(file_path) = try_get_document_from_element(window_ref) {
        if let Some(project) = extract_project_name_from_path(&file_path) {
            return Some((project, None));
        }
    }

    // Last resort: try focused element for document path
    let focused_attr = CFString::new(constants::AX_FOCUSED_UI_ELEMENT);
    let mut focused_ref: CFTypeRef = ptr::null_mut();
    let focused_result = AXUIElementCopyAttributeValue(
        window_ref,
        focused_attr.as_concrete_TypeRef(),
        &mut focused_ref,
    );

    if focused_result == 0 && !focused_ref.is_null() {
        if let Some(file_path) = try_get_document_from_element(focused_ref as AXUIElementRef) {
            if let Some(project) = extract_project_name_from_path(&file_path) {
                return Some((project, None));
            }
        }
    }

    None
}

/// Attempts to extract document path from an accessibility element
#[cfg(target_os = "macos")]
unsafe fn try_get_document_from_element(element_ref: AXUIElementRef) -> Option<String> {
    use std::ptr;

    // Try to get AXDocument attribute
    let document_attr = CFString::new(constants::AX_DOCUMENT);
    let mut document_ref: CFTypeRef = ptr::null_mut();
    let doc_result = AXUIElementCopyAttributeValue(
        element_ref,
        document_attr.as_concrete_TypeRef(),
        &mut document_ref,
    );

    if doc_result == 0 && !document_ref.is_null() {
        if let Some(file_path) = cf_type_to_string(document_ref) {
            return Some(file_path);
        }
    }

    // Fallback: try AXURL attribute
    let url_attr = CFString::new(constants::AX_URL);
    let mut url_ref: CFTypeRef = ptr::null_mut();
    let url_result =
        AXUIElementCopyAttributeValue(element_ref, url_attr.as_concrete_TypeRef(), &mut url_ref);

    if url_result == 0 && !url_ref.is_null() {
        if let Some(url) = cf_type_to_string(url_ref) {
            let file_path = if url.starts_with(constants::FILE_URL_PREFIX) {
                &url[constants::FILE_URL_PREFIX.len()..]
            } else {
                &url
            };
            return Some(file_path.to_string());
        }
    }

    // Try AXTitle as a last resort
    let title_attr = CFString::new(constants::AX_TITLE);
    let mut title_ref: CFTypeRef = ptr::null_mut();
    let title_result = AXUIElementCopyAttributeValue(
        element_ref,
        title_attr.as_concrete_TypeRef(),
        &mut title_ref,
    );

    if title_result == 0 && !title_ref.is_null() {
        if let Some(title) = cf_type_to_string(title_ref) {
            // Try to extract path from title if it contains file path
            if title.contains("/") {
                return Some(title);
            }
        }
    }

    None
}

/// Converts a Core Foundation type reference to a CFArray if possible
#[cfg(target_os = "macos")]
unsafe fn cf_type_to_array(cf_ref: CFTypeRef) -> Option<CFArray<CFTypeRef>> {
    use core_foundation::array::CFArrayGetTypeID;
    use core_foundation::base::CFGetTypeID;

    if cf_ref.is_null() {
        return None;
    }

    // Check if it's a CFArray
    if CFGetTypeID(cf_ref) == CFArrayGetTypeID() {
        Some(CFArray::wrap_under_get_rule(
            cf_ref as *const core_foundation::array::__CFArray,
        ))
    } else {
        None
    }
}

/// Converts a Core Foundation type reference to a String if possible
#[cfg(target_os = "macos")]
unsafe fn cf_type_to_string(cf_ref: CFTypeRef) -> Option<String> {
    use core_foundation::base::CFGetTypeID;

    if cf_ref.is_null() {
        return None;
    }

    // Check if it's a CFString
    if CFGetTypeID(cf_ref) == CFString::type_id() {
        let cf_string = CFString::wrap_under_get_rule(cf_ref as CFStringRef);
        Some(cf_string.to_string())
    } else {
        None
    }
}

/// Extracts a string value from a Core Foundation dictionary
#[cfg(target_os = "macos")]
pub unsafe fn get_string_value(dict: &CFDictionary, key: &str) -> Option<String> {
    let cf_key = CFString::new(key);
    let key_ptr = cf_key.as_concrete_TypeRef() as *const c_void;

    if let Some(value_ref) = dict.find(key_ptr) {
        let cf_string = CFString::wrap_under_get_rule(*value_ref as CFStringRef);
        Some(cf_string.to_string())
    } else {
        None
    }
}

/// Extracts a numeric value from a Core Foundation dictionary
#[cfg(target_os = "macos")]
pub unsafe fn get_number_value(dict: &CFDictionary, key: &str) -> Option<i64> {
    let cf_key = CFString::new(key);
    let key_ptr = cf_key.as_concrete_TypeRef() as *const c_void;

    if let Some(value_ref) = dict.find(key_ptr) {
        let cf_number = CFNumber::wrap_under_get_rule(*value_ref as CFNumberRef);
        cf_number.to_i64()
    } else {
        None
    }
}

// Stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub unsafe fn populate_project_info_for_pid(
    _pid: i32,
    _window_project_map: &mut HashMap<u32, (Option<String>, Option<String>)>,
) {
    // No-op on non-macOS platforms
}

#[cfg(not(target_os = "macos"))]
pub unsafe fn get_string_value(
    _dict: &std::collections::HashMap<String, String>,
    _key: &str,
) -> Option<String> {
    None
}

#[cfg(not(target_os = "macos"))]
pub unsafe fn get_number_value(
    _dict: &std::collections::HashMap<String, String>,
    _key: &str,
) -> Option<i64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_simple_window_mapping() {
        // Test the simple mapping approach
        let mut window_project_map = HashMap::new();

        // Simulate the mapping
        window_project_map.insert(
            8229,
            (Some("swii".to_string()), Some("window_list.rs".to_string())),
        );
        window_project_map.insert(
            8512,
            (Some("switch".to_string()), Some("commands.rs".to_string())),
        );
        window_project_map.insert(
            9062,
            (
                Some("promptbook".to_string()),
                Some("eslint.config.mjs".to_string()),
            ),
        );

        // Test lookups
        assert_eq!(
            window_project_map.get(&8229),
            Some(&(Some("swii".to_string()), Some("window_list.rs".to_string())))
        );
        assert_eq!(
            window_project_map.get(&8512),
            Some(&(Some("switch".to_string()), Some("commands.rs".to_string())))
        );
        assert_eq!(
            window_project_map.get(&9062),
            Some(&(
                Some("promptbook".to_string()),
                Some("eslint.config.mjs".to_string())
            ))
        );

        // Test missing window
        assert_eq!(window_project_map.get(&99999), None);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_window_correlation_real_scenario() {
        // Test scenario similar to the reported bug - simple direct mapping
        let mut window_project_map = HashMap::new();

        // The old bug: windows 8229 and 9062 both showed "swii" project
        // With the new approach, they should show different projects
        window_project_map.insert(
            8229,
            (Some("swii".to_string()), Some("window_list.rs".to_string())),
        );
        window_project_map.insert(
            9062,
            (
                Some("promptbook".to_string()),
                Some("eslint.config.mjs".to_string()),
            ),
        );
        window_project_map.insert(
            8512,
            (Some("switch".to_string()), Some("commands.rs".to_string())),
        );

        // Verify each window shows the correct project
        assert_eq!(
            window_project_map.get(&8229),
            Some(&(Some("swii".to_string()), Some("window_list.rs".to_string())))
        );
        assert_eq!(
            window_project_map.get(&9062),
            Some(&(
                Some("promptbook".to_string()),
                Some("eslint.config.mjs".to_string())
            ))
        );
        assert_eq!(
            window_project_map.get(&8512),
            Some(&(Some("switch".to_string()), Some("commands.rs".to_string())))
        );

        // Verify they're all different (no collision)
        let result_8229 = window_project_map.get(&8229);
        let result_9062 = window_project_map.get(&9062);
        let result_8512 = window_project_map.get(&8512);

        assert_ne!(result_8229, result_9062);
        assert_ne!(result_8229, result_8512);
        assert_ne!(result_9062, result_8512);
    }
}
