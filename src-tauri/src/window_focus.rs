use std::ffi::c_void;
use std::ptr;

#[cfg(target_os = "macos")]
use core_foundation::{
    array::CFArray,
    base::{CFTypeRef, TCFType},
    dictionary::CFDictionary,
    string::{CFString, CFStringRef},
};

#[cfg(target_os = "macos")]
use core_graphics::window::{
    kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};

use crate::macos_accessibility::get_number_value;

#[cfg(target_os = "macos")]
type AXUIElementRef = *const crate::macos_accessibility::__AXUIElement;

#[cfg(target_os = "macos")]
type AXError = i32;

#[cfg(target_os = "macos")]
extern "C" {
    fn AXUIElementCreateApplication(pid: libc::pid_t) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef) -> AXError;
    fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;
}

#[cfg(target_os = "macos")]
extern "C" {
    fn NSApplicationLoad() -> bool;
    fn objc_getClass(name: *const std::os::raw::c_char) -> *const c_void;
    fn sel_registerName(str: *const std::os::raw::c_char) -> *const c_void;
    fn objc_msgSend(obj: *const c_void, sel: *const c_void, ...) -> *const c_void;
}

mod constants {
    pub const AX_WINDOWS: &str = "AXWindows";
    pub const AX_RAISE: &str = "AXRaise";
    pub const AX_MAIN: &str = "AXMain";
    pub const CG_WINDOW_OWNER_PID: &str = "kCGWindowOwnerPID";
    pub const CG_WINDOW_NUMBER: &str = "kCGWindowNumber";
}

#[derive(Debug, PartialEq)]
pub enum WindowFocusError {
    WindowNotFound,
    ApplicationNotFound,
    PermissionDenied,
    SystemError(String),
}

impl std::fmt::Display for WindowFocusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowFocusError::WindowNotFound => write!(f, "Window not found"),
            WindowFocusError::ApplicationNotFound => write!(f, "Application not found"),
            WindowFocusError::PermissionDenied => write!(f, "Accessibility permissions required"),
            WindowFocusError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

#[tauri::command]
pub async fn bring_window_to_front(
    _app: tauri::AppHandle,
    args: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    println!("bring_window_to_front args: {:?}", args);
    crate::command_wrapper::create_command("bring_window_to_front", args, |ctx| {
        let pid = ctx
            .parameters
            .get("pid")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| "Missing or invalid 'pid' parameter".to_string())?
            as i32;

        let window_number = ctx
            .parameters
            .get("window_number")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "Missing or invalid 'window_number' parameter".to_string())?
            as u32;

        ctx.logger.info(&format!(
            "Attempting to focus window PID: {}, window number: {}",
            pid, window_number
        ));

        #[cfg(target_os = "macos")]
        {
            let result = macos_bring_window_to_front(pid, window_number);
            match result {
                Ok(()) => {
                    ctx.logger.info("Successfully brought window to front");
                    Ok(())
                }
                Err(e) => {
                    ctx.logger
                        .error(&format!("Failed to bring window to front: {}", e));
                    Err(e.to_string())
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            ctx.logger
                .warn("Window focusing is only supported on macOS");
            Err("Window focusing is only supported on macOS".to_string())
        }
    })
    .await
}

#[cfg(target_os = "macos")]
fn macos_bring_window_to_front(pid: i32, window_number: u32) -> Result<(), WindowFocusError> {
    unsafe {
        // Step 1: Verify the window exists
        if !window_exists(pid, window_number)? {
            return Err(WindowFocusError::WindowNotFound);
        }

        // Step 2: Try to bring application to front (but don't fail if it doesn't work)
        let _ = activate_application(pid);

        // Step 3: Focus the specific window
        focus_window_by_number(pid, window_number)?;

        Ok(())
    }
}

#[cfg(target_os = "macos")]
unsafe fn window_exists(pid: i32, window_number: u32) -> Result<bool, WindowFocusError> {
    let window_list_info = CGWindowListCopyWindowInfo(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        0,
    );

    if window_list_info.is_null() {
        return Err(WindowFocusError::SystemError(
            "Failed to get window list".to_string(),
        ));
    }

    let window_list: CFArray<CFDictionary> = CFArray::wrap_under_create_rule(window_list_info);

    for i in 0..window_list.len() {
        if let Some(window_dict) = window_list.get(i) {
            if let (Some(dict_pid), Some(dict_window_number)) = (
                get_number_value(&window_dict, constants::CG_WINDOW_OWNER_PID),
                get_number_value(&window_dict, constants::CG_WINDOW_NUMBER),
            ) {
                if dict_pid as i32 == pid && dict_window_number as u32 == window_number {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

#[cfg(target_os = "macos")]
unsafe fn activate_application(pid: i32) -> Result<(), WindowFocusError> {
    // Load NSApplication framework
    if !NSApplicationLoad() {
        return Err(WindowFocusError::SystemError(
            "Failed to load NSApplication".to_string(),
        ));
    }

    // Get NSRunningApplication class
    let ns_running_app_class =
        objc_getClass(b"NSRunningApplication\0".as_ptr() as *const std::os::raw::c_char);
    if ns_running_app_class.is_null() {
        return Err(WindowFocusError::SystemError(
            "NSRunningApplication class not found".to_string(),
        ));
    }

    // Selector for +runningApplicationWithProcessIdentifier:
    let app_with_pid_sel = sel_registerName(
        b"runningApplicationWithProcessIdentifier:\0".as_ptr() as *const std::os::raw::c_char
    );
    if app_with_pid_sel.is_null() {
        return Err(WindowFocusError::SystemError(
            "Selector runningApplicationWithProcessIdentifier: not found".to_string(),
        ));
    }

    // Build a fn pointer for objc_msgSend(id, SEL, pid) -> id
    let msg_send_app: extern "C" fn(*const c_void, *const c_void, libc::pid_t) -> *const c_void =
        std::mem::transmute(objc_msgSend as *const c_void);

    // Call +[NSRunningApplication runningApplicationWithProcessIdentifier:]
    let app = msg_send_app(ns_running_app_class, app_with_pid_sel, pid);
    if app.is_null() {
        return Err(WindowFocusError::ApplicationNotFound);
    }

    // Selector for -activateWithOptions:
    let activate_sel =
        sel_registerName(b"activateWithOptions:\0".as_ptr() as *const std::os::raw::c_char);
    if activate_sel.is_null() {
        return Err(WindowFocusError::SystemError(
            "Selector activateWithOptions: not found".to_string(),
        ));
    }

    // Use only NSApplicationActivateIgnoringOtherApps (2),
    // so we don't bring *all* windows forward—just let our AXRaise call handle the single window.
    let options: usize = 1 << 1; // NSApplicationActivateIgnoringOtherApps

    // Build a fn pointer for objc_msgSend(id, SEL, usize) -> id
    let activate_fn: extern "C" fn(*const c_void, *const c_void, usize) -> *const c_void =
        std::mem::transmute(objc_msgSend as *const c_void);

    // Call -[NSRunningApplication activateWithOptions:]
    activate_fn(app, activate_sel, options);

    Ok(())
}

#[cfg(target_os = "macos")]
unsafe fn focus_window_by_number(pid: i32, window_number: u32) -> Result<(), WindowFocusError> {
    let app_ref = AXUIElementCreateApplication(pid);
    if app_ref.is_null() {
        return Err(WindowFocusError::ApplicationNotFound);
    }

    // Get all windows for the application
    let windows_attr = CFString::new(constants::AX_WINDOWS);
    let mut windows_ref: CFTypeRef = ptr::null_mut();
    let result = AXUIElementCopyAttributeValue(
        app_ref,
        windows_attr.as_concrete_TypeRef(),
        &mut windows_ref,
    );

    if result != 0 {
        return Err(match result {
            -25200 => WindowFocusError::PermissionDenied,
            _ => WindowFocusError::SystemError(format!("AX error: {}", result)),
        });
    }

    if windows_ref.is_null() {
        return Err(WindowFocusError::WindowNotFound);
    }

    let windows_array = cf_type_to_array(windows_ref);
    if windows_array.is_none() {
        return Err(WindowFocusError::SystemError(
            "Failed to convert windows to array".to_string(),
        ));
    }

    let windows = windows_array.unwrap();

    // Find the target window by correlating with Core Graphics windows
    let target_window_ref = find_ax_window_by_cg_number(pid, window_number, &windows)?;

    // Raise the window
    let raise_action = CFString::new(constants::AX_RAISE);
    let raise_result =
        AXUIElementPerformAction(target_window_ref, raise_action.as_concrete_TypeRef());
    if raise_result != 0 {
        return Err(WindowFocusError::SystemError(format!(
            "Failed to raise window: {}",
            raise_result
        )));
    }

    // 2) Small pause so the activation “lands”
    std::thread::sleep(std::time::Duration::from_millis(50));

    // 3) Mark it as the main window
    let main_attr = CFString::new(constants::AX_MAIN);
    let main_value = core_foundation::boolean::CFBoolean::true_value();
    let _ = AXUIElementSetAttributeValue(
        target_window_ref,
        main_attr.as_concrete_TypeRef(),
        main_value.as_CFTypeRef(),
    );

    // 4) Explicitly set the app’s focused window
    let focused_attr = CFString::new("AXFocusedWindow");
    let _ = AXUIElementSetAttributeValue(
        app_ref,
        focused_attr.as_concrete_TypeRef(),
        target_window_ref as CFTypeRef,
    );

    Ok(())
}

#[cfg(target_os = "macos")]
unsafe fn find_ax_window_by_cg_number(
    pid: i32,
    target_window_number: u32,
    ax_windows: &CFArray<CFTypeRef>,
) -> Result<AXUIElementRef, WindowFocusError> {
    // Get Core Graphics windows for this PID to establish correlation
    let window_list_info = CGWindowListCopyWindowInfo(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        0,
    );

    if window_list_info.is_null() {
        return Err(WindowFocusError::SystemError(
            "Failed to get CG window list".to_string(),
        ));
    }

    let window_list: CFArray<CFDictionary> = CFArray::wrap_under_create_rule(window_list_info);
    let mut cg_windows_for_pid = Vec::new();

    // Collect CG windows for this PID in order
    for i in 0..window_list.len() {
        if let Some(window_dict) = window_list.get(i) {
            if let Some(dict_pid) = get_number_value(&window_dict, constants::CG_WINDOW_OWNER_PID) {
                if dict_pid as i32 == pid {
                    if let Some(window_num) =
                        get_number_value(&window_dict, constants::CG_WINDOW_NUMBER)
                    {
                        cg_windows_for_pid.push(window_num as u32);
                    }
                }
            }
        }
    }

    // Find the index of our target window number
    let target_index = cg_windows_for_pid
        .iter()
        .position(|&num| num == target_window_number)
        .ok_or(WindowFocusError::WindowNotFound)?;

    // Return the corresponding AX window (assuming same order)
    let ax_len = ax_windows.len();
    if (target_index as isize) < ax_len {
        if let Some(window_ref_ptr) = ax_windows.get(target_index as isize) {
            return Ok(*window_ref_ptr as AXUIElementRef);
        }
    }

    Err(WindowFocusError::WindowNotFound)
}

#[cfg(target_os = "macos")]
unsafe fn cf_type_to_array(cf_ref: CFTypeRef) -> Option<CFArray<CFTypeRef>> {
    use core_foundation::array::CFArrayGetTypeID;
    use core_foundation::base::CFGetTypeID;

    if cf_ref.is_null() {
        return None;
    }

    if CFGetTypeID(cf_ref) == CFArrayGetTypeID() {
        Some(CFArray::wrap_under_get_rule(
            cf_ref as *const core_foundation::array::__CFArray,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_focus_error_display() {
        assert_eq!(
            WindowFocusError::WindowNotFound.to_string(),
            "Window not found"
        );
        assert_eq!(
            WindowFocusError::ApplicationNotFound.to_string(),
            "Application not found"
        );
        assert_eq!(
            WindowFocusError::PermissionDenied.to_string(),
            "Accessibility permissions required"
        );
        assert_eq!(
            WindowFocusError::SystemError("test".to_string()).to_string(),
            "System error: test"
        );
    }

    #[test]
    fn test_window_focus_error_equality() {
        assert_eq!(
            WindowFocusError::WindowNotFound,
            WindowFocusError::WindowNotFound
        );
        assert_eq!(
            WindowFocusError::ApplicationNotFound,
            WindowFocusError::ApplicationNotFound
        );
        assert_eq!(
            WindowFocusError::PermissionDenied,
            WindowFocusError::PermissionDenied
        );
        assert_eq!(
            WindowFocusError::SystemError("same".to_string()),
            WindowFocusError::SystemError("same".to_string())
        );
        assert_ne!(
            WindowFocusError::SystemError("different".to_string()),
            WindowFocusError::SystemError("other".to_string())
        );
    }

    #[test]
    fn test_bring_window_to_front_signature() {
        // Test function signature and basic error handling
        #[cfg(target_os = "macos")]
        {
            let result = macos_bring_window_to_front(0, 0);
            // On macOS, should return an error for invalid PID/window
            assert!(result.is_err());
        }

        #[cfg(not(target_os = "macos"))]
        {
            // This test doesn't apply on non-macOS since the function is not available
        }
    }

    #[test]
    fn test_bring_window_to_front_invalid_inputs() {
        // Test with clearly invalid inputs
        #[cfg(target_os = "macos")]
        {
            let result = macos_bring_window_to_front(-1, 0);
            assert!(result.is_err());

            let result = macos_bring_window_to_front(0, 0);
            assert!(result.is_err());
        }

        #[cfg(not(target_os = "macos"))]
        {
            // This test doesn't apply on non-macOS since the function is not available
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_window_exists_with_invalid_inputs() {
        unsafe {
            // Test with invalid PID
            let result = window_exists(-1, 12345);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);

            // Test with valid PID but non-existent window
            let result = window_exists(1, 999999999);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_activate_application_invalid_pid() {
        unsafe {
            // Test with invalid PID
            let result = activate_application(-1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), WindowFocusError::ApplicationNotFound);
        }
    }
    #[test]
    fn test_constants_not_empty() {
        assert!(!constants::AX_WINDOWS.is_empty());
        assert!(!constants::AX_RAISE.is_empty());
        assert!(!constants::AX_MAIN.is_empty());
        assert!(!constants::CG_WINDOW_OWNER_PID.is_empty());
        assert!(!constants::CG_WINDOW_NUMBER.is_empty());
    }
}
