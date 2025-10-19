//! macOS window management functionality including window enumeration,
//! icon extraction, and editor detection.
//!
//! ## Examples
//!
//! ```rust,ignore
//! use swii_lib::macos_window::get_editor_windows;
//!
//! let windows = get_editor_windows();
//! for window in windows {
//!     println!("Found editor: {} - {}", window.app_name, window.window_name.unwrap_or_default());
//! }
//! ```

use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, warn};

use crate::editor_config;

use base64::Engine;
use core_foundation::{array::CFArray, base::TCFType, dictionary::CFDictionary};
use core_graphics::window::{
    kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};

use crate::macos_accessibility::{
    get_number_value, get_string_value, populate_project_info_for_pid,
};
use crate::types::WindowInfo;

/// Errors that can occur during window management operations
#[derive(Debug, Error)]
pub enum WindowError {
    #[error("Failed to access window list: {message}")]
    WindowListAccess { message: String },

    #[error("Failed to read window information: {message}")]
    WindowInfoExtraction { message: String },

    #[error("Failed to extract app icon: {message}")]
    IconExtraction { message: String },

    #[error("Failed to read bundle information: {message}")]
    BundleAccess { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Base64 encoding error: {0}")]
    Base64(#[from] base64::DecodeError),
}

// Core Graphics window dictionary keys
pub const CG_WINDOW_OWNER_NAME: &str = "kCGWindowOwnerName";
pub const CG_WINDOW_OWNER_PID: &str = "kCGWindowOwnerPID";
pub const CG_WINDOW_NUMBER: &str = "kCGWindowNumber";
pub const CG_WINDOW_NAME: &str = "kCGWindowName";

/// Gets all editor windows on macOS
///
/// This function enumerates all visible windows on macOS and filters them
/// to return only windows belonging to known code editors.
///
/// # Returns
///
/// A vector of `WindowInfo` structs containing information about editor windows.
///
/// # Examples
///
/// ```rust,ignore
/// use swii_lib::macos_window::get_editor_windows;
///
/// let editor_windows = get_editor_windows();
/// println!("Found {} editor windows", editor_windows.len());
/// ```
pub fn get_editor_windows() -> Result<Vec<WindowInfo>, WindowError> {
    debug!("Starting enumeration of macOS editor windows");

    // SAFETY: This function is safe to call as it only reads window information
    // and doesn't modify any system state. The Core Graphics API is thread-safe
    // for read operations.
    let window_list_info = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0, // kCGNullWindowID - get all windows
        )
    };

    if window_list_info.is_null() {
        warn!("Failed to retrieve window list from Core Graphics");
        return Err(WindowError::WindowListAccess {
            message: "Core Graphics returned null window list".to_string(),
        });
    }

    // SAFETY: window_list_info is guaranteed to be non-null at this point
    let window_list: CFArray<CFDictionary> =
        unsafe { CFArray::wrap_under_create_rule(window_list_info) };

    let mut windows = Vec::new();
    let mut window_project_map = HashMap::new();
    let mut processed_pids = std::collections::HashSet::new();

    // First pass: collect all editor app PIDs and their project info
    for i in 0..window_list.len() {
        if let Some(window_dict) = window_list.get(i) {
            // SAFETY: get_string_value and get_number_value are safe to call with valid
            // CFDictionary references obtained from Core Graphics APIs. The window_dict
            // comes from CGWindowListCopyWindowInfo which returns valid dictionaries.
            let app_name = unsafe { get_string_value(&window_dict, CG_WINDOW_OWNER_NAME) };
            if let Some(app_name) = app_name {
                if is_editor_window(&app_name) {
                    let pid = unsafe { get_number_value(&window_dict, CG_WINDOW_OWNER_PID) };
                    if let Some(pid) = pid {
                        let pid = pid as i32;
                        // Get all project info for this PID once
                        if !processed_pids.contains(&pid) {
                            // SAFETY: populate_project_info_for_pid is safe to call with valid PIDs
                            // obtained from Core Graphics. The function handles invalid PIDs gracefully.
                            unsafe { populate_project_info_for_pid(pid, &mut window_project_map) };
                            processed_pids.insert(pid);
                        }
                    }
                }
            }
        }
    }

    // Second pass: create WindowInfo objects using the map
    for i in 0..window_list.len() {
        if let Some(window_dict) = window_list.get(i) {
            match extract_editor_window_info(&window_dict, &window_project_map) {
                Ok(Some(info)) => {
                    windows.push(info);
                }
                Ok(None) => {
                    // Not an editor window, skip silently
                }
                Err(e) => {
                    warn!("Failed to extract window info: {}", e);
                    // Continue processing other windows
                }
            }
        }
    }

    debug!("Found {} editor windows", windows.len());
    Ok(windows)
}

/// Extracts window information from a macOS window dictionary
///
/// # Arguments
///
/// * `window_dict` - Core Foundation dictionary containing window information
/// * `window_project_map` - Map of window numbers to project information
///
/// # Returns
///
/// Returns `Some(WindowInfo)` if the window is an editor window and information
/// was successfully extracted, `None` if it's not an editor window, or an error
/// if extraction failed.
///
/// # Safety
///
/// This function is safe to call as it only reads from the provided dictionary
/// and doesn't perform any unsafe operations itself.
fn extract_editor_window_info(
    window_dict: &CFDictionary,
    window_project_map: &HashMap<u32, (Option<String>, Option<String>)>,
) -> Result<Option<WindowInfo>, WindowError> {
    // Extract basic window info
    // SAFETY: get_string_value and get_number_value are safe to call with valid
    // CFDictionary references obtained from Core Graphics APIs. The window_dict
    // comes from CGWindowListCopyWindowInfo which returns valid dictionaries.
    let app_name = unsafe { get_string_value(window_dict, CG_WINDOW_OWNER_NAME) }.ok_or_else(
        || WindowError::WindowInfoExtraction {
            message: "Failed to extract app name".to_string(),
        },
    )?;

    let window_name = unsafe { get_string_value(window_dict, CG_WINDOW_NAME) };
    let pid = unsafe { get_number_value(window_dict, CG_WINDOW_OWNER_PID) }.ok_or_else(|| {
        WindowError::WindowInfoExtraction {
            message: "Failed to extract PID".to_string(),
        }
    })? as i32;

    let window_number = unsafe { get_number_value(window_dict, CG_WINDOW_NUMBER) }.ok_or_else(
        || WindowError::WindowInfoExtraction {
            message: "Failed to extract window number".to_string(),
        },
    )? as u32;

    // Filter out windows that are not editor windows
    if !is_editor_window(&app_name) {
        return Ok(None);
    }

    // Look up project info from our simple map
    let (project, active_editor_tab) = window_project_map
        .get(&window_number)
        .cloned()
        .unwrap_or((None, None));

    // Get app icon - log errors but don't fail the entire operation
    let app_icon = match get_app_icon_for_pid(pid) {
        Ok(icon) => {
            if let Some(ref icon_data) = icon {
                debug!(
                    "Found app icon for {} (PID: {}), length: {}",
                    app_name,
                    pid,
                    icon_data.len()
                );
            } else {
                debug!("No app icon found for {} (PID: {})", app_name, pid);
            }
            icon
        }
        Err(e) => {
            warn!(
                "Failed to extract app icon for {} (PID: {}): {}",
                app_name, pid, e
            );
            None
        }
    };

    Ok(Some(WindowInfo {
        app_name,
        window_name,
        pid,
        window_number,
        project,
        active_editor_tab,
        app_icon,
    }))
}

/// Checks if an application is a code editor
pub fn is_editor_window(app_name: &str) -> bool {
    editor_config::is_editor_application(app_name)
}

/// Gets the app icon for a given PID
///
/// This function attempts to extract the application icon for a process
/// by finding its bundle path and extracting the icon from the bundle.
///
/// # Arguments
///
/// * `pid` - Process ID of the application
///
/// # Returns
///
/// Returns the base64-encoded icon data if successful, None if the icon
/// could not be extracted.
///
/// # Examples
///
/// ```rust,ignore
/// use swii_lib::macos_window::get_app_icon_for_pid;
///
/// let icon = get_app_icon_for_pid(12345);
/// match icon {
///     Ok(Some(icon_data)) => println!("Found icon with {} bytes", icon_data.len()),
///     Ok(None) => println!("No icon found"),
///     Err(e) => println!("Error extracting icon: {}", e),
/// }
/// ```
pub fn get_app_icon_for_pid(pid: i32) -> Result<Option<String>, WindowError> {
    debug!("Getting app icon for PID: {}", pid);

    // SAFETY: This function is safe to call as it only reads window information
    // and doesn't modify any system state. The Core Graphics API is thread-safe
    // for read operations.
    let window_list_info = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0,
        )
    };

    if window_list_info.is_null() {
        warn!("Window list info is null for PID {}", pid);
        return Err(WindowError::WindowListAccess {
            message: format!("Failed to get window list for PID {}", pid),
        });
    }

    // SAFETY: window_list_info is guaranteed to be non-null at this point
    let window_list: CFArray<CFDictionary> =
        unsafe { CFArray::wrap_under_create_rule(window_list_info) };

    // Find a window for this PID to get the app bundle path
    for i in 0..window_list.len() {
        if let Some(window_dict) = window_list.get(i) {
            // SAFETY: get_number_value is safe to call with valid CFDictionary references
            // obtained from Core Graphics APIs.
            if let Some(dict_pid) = unsafe { get_number_value(&window_dict, "kCGWindowOwnerPID") } {
                if dict_pid as i32 == pid {
                    // Try to get the app bundle path from the window info
                    match get_app_bundle_path_for_pid(pid) {
                        Ok(Some(bundle_path)) => {
                            debug!("Found bundle path: {}", bundle_path);
                            return get_icon_from_bundle_path(&bundle_path);
                        }
                        Ok(None) => {
                            debug!("No bundle path found for PID: {}", pid);
                        }
                        Err(e) => {
                            warn!("Failed to get bundle path for PID {}: {}", pid, e);
                        }
                    }
                    break;
                }
            }
        }
    }

    debug!("No app icon found for PID: {}", pid);
    Ok(None)
}

/// Gets the app bundle path for a given PID
///
/// This function attempts to determine the bundle path of an application
/// given its process ID by looking up the application name and mapping
/// it to known bundle locations.
///
/// # Arguments
///
/// * `pid` - Process ID of the application
///
/// # Returns
///
/// Returns the bundle path as a string if found and the bundle exists,
/// None if the bundle path could not be determined.
///
/// # Safety
///
/// This function is safe to call as it only reads window information
/// and performs filesystem existence checks.
fn get_app_bundle_path_for_pid(pid: i32) -> Result<Option<String>, WindowError> {
    // SAFETY: This function is safe to call as it only reads window information
    // and doesn't modify any system state. The Core Graphics API is thread-safe
    // for read operations.
    let window_list_info = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            0,
        )
    };

    if !window_list_info.is_null() {
        // SAFETY: window_list_info is guaranteed to be non-null at this point
        let window_list: CFArray<CFDictionary> =
            unsafe { CFArray::wrap_under_create_rule(window_list_info) };

        for i in 0..window_list.len() {
            if let Some(window_dict) = window_list.get(i) {
                // SAFETY: get_number_value and get_string_value are safe to call with valid
                // CFDictionary references obtained from Core Graphics APIs.
                if let Some(dict_pid) = unsafe { get_number_value(&window_dict, "kCGWindowOwnerPID") }
                {
                    if dict_pid as i32 == pid {
                        if let Some(app_name) =
                            unsafe { get_string_value(&window_dict, "kCGWindowOwnerName") }
                        {
                            // Look for a matching editor path using the centralized config
                            if let Some(bundle_path) = editor_config::get_editor_path(&app_name) {
                                // Check if the bundle exists
                                if std::path::Path::new(bundle_path).exists() {
                                    return Ok(Some(bundle_path.to_string()));
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Gets the app icon from a bundle path
///
/// This function extracts the application icon from a macOS application bundle
/// by parsing the Info.plist file and locating the icon file.
///
/// # Arguments
///
/// * `bundle_path` - Path to the application bundle (.app directory)
///
/// # Returns
///
/// Returns the base64-encoded icon data if successful, None if extraction fails.
///
/// # Safety
///
/// This function is safe to call as it only performs file system operations
/// and doesn't use any unsafe code internally.
fn get_icon_from_bundle_path(bundle_path: &str) -> Result<Option<String>, WindowError> {
    // Construct the Info.plist path
    let info_plist_path = format!("{}/Contents/Info.plist", bundle_path);

    // Try to get the icon file name from Info.plist
    let plist_data =
        std::fs::read_to_string(&info_plist_path).map_err(|e| WindowError::BundleAccess {
            message: format!("Failed to read Info.plist from {}: {}", info_plist_path, e),
        })?;

    // Simple parsing to find CFBundleIconFile
    if let Some(icon_file) = extract_icon_file_from_plist(&plist_data) {
        let icon_path = if icon_file.ends_with(".icns") {
            format!("{}/Contents/Resources/{}", bundle_path, icon_file)
        } else {
            format!("{}/Contents/Resources/{}.icns", bundle_path, icon_file)
        };

        // Convert icon to base64
        return convert_icon_to_base64(&icon_path);
    }

    // Fallback: try to find any .icns file in Resources
    let resources_path = format!("{}/Contents/Resources", bundle_path);
    if let Ok(entries) = std::fs::read_dir(&resources_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "icns" {
                        if let Some(base64) = convert_icon_to_base64(&path.to_string_lossy())? {
                            return Ok(Some(base64));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Extracts icon file name from Info.plist content
fn extract_icon_file_from_plist(plist_content: &str) -> Option<String> {
    // Simple parsing for CFBundleIconFile
    if let Some(start) = plist_content.find("<key>CFBundleIconFile</key>") {
        if let Some(value_start) = plist_content[start..].find("<string>") {
            let value_start = start + value_start + 8;
            if let Some(value_end) = plist_content[value_start..].find("</string>") {
                let icon_file = plist_content[value_start..value_start + value_end].to_string();
                return Some(icon_file);
            }
        }
    }
    None
}

/// Converts an icon file to base64 string
///
/// This function reads an icon file and converts its contents to a base64-encoded string.
/// If the file is an icns file, it attempts to extract PNG data from it.
///
/// # Arguments
///
/// * `icon_path` - Path to the icon file
///
/// # Returns
///
/// Returns the base64-encoded icon data if successful, None if conversion fails.
///
/// # Examples
///
/// ```rust,ignore
/// use swii_lib::macos_window::convert_icon_to_base64;
///
/// let base64_data = convert_icon_to_base64("/path/to/icon.icns");
/// match base64_data {
///     Ok(Some(data)) => println!("Converted icon: {}", data.len()),
///     Ok(None) => println!("Conversion failed"),
///     Err(e) => println!("Error: {}", e),
/// }
/// ```
fn convert_icon_to_base64(icon_path: &str) -> Result<Option<String>, WindowError> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(icon_path).map_err(|e| WindowError::IconExtraction {
        message: format!("Failed to open icon file {}: {}", icon_path, e),
    })?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| WindowError::IconExtraction {
            message: format!("Failed to read icon file {}: {}", icon_path, e),
        })?;

    // If it's an icns file, we need to convert it to PNG
    if icon_path.ends_with(".icns") {
        // For now, let's try to extract a PNG from the icns file
        // This is a simplified approach - in a production app you'd want a proper icns parser
        if let Some(png_data) = extract_png_from_icns(&buffer)? {
            return Ok(Some(
                base64::engine::general_purpose::STANDARD.encode(&png_data),
            ));
        }
    }

    // Convert to base64 using the new API
    Ok(Some(
        base64::engine::general_purpose::STANDARD.encode(&buffer),
    ))
}

/// Attempts to extract PNG data from an icns file
///
/// This function parses an icns (Apple Icon Image) file and attempts to extract
/// PNG data from it. It looks for various icon types that contain PNG data.
///
/// # Arguments
///
/// * `icns_data` - Raw bytes of the icns file
///
/// # Returns
///
/// Returns PNG data if found, or creates a fallback icon if no PNG is found.
///
/// # Examples
///
/// ```rust,ignore
/// use swii_lib::macos_window::extract_png_from_icns;
///
/// let icns_bytes = std::fs::read("icon.icns").unwrap();
/// let png_data = extract_png_from_icns(&icns_bytes);
/// ```
fn extract_png_from_icns(icns_data: &[u8]) -> Result<Option<Vec<u8>>, WindowError> {
    debug!("Extracting PNG from icns file, size: {}", icns_data.len());

    // icns file format:
    // - 4 bytes: "icns" magic
    // - 4 bytes: total file size (big endian)
    // - For each icon:
    //   - 4 bytes: icon type (e.g., "ic07", "ic08", "ic09", "ic10", "ic11", "ic12", "ic13", "ic14")
    //   - 4 bytes: icon data size (big endian)
    //   - N bytes: icon data (PNG format for newer icons)

    if icns_data.len() < 8 {
        warn!("icns file too small: {} bytes", icns_data.len());
        return Ok(None);
    }

    // Check if it's a valid icns file
    if &icns_data[0..4] != b"icns" {
        warn!("Not a valid icns file - invalid magic bytes");
        return Ok(None);
    }

    let mut offset = 8; // Skip header

    while offset < icns_data.len() - 8 {
        if offset + 8 > icns_data.len() {
            break;
        }

        // Read icon type
        let icon_type = &icns_data[offset..offset + 4];
        offset += 4;

        // Read icon size
        if offset + 4 > icns_data.len() {
            break;
        }
        let icon_size = u32::from_be_bytes([
            icns_data[offset],
            icns_data[offset + 1],
            icns_data[offset + 2],
            icns_data[offset + 3],
        ]) as usize;
        offset += 4;

        debug!(
            "Found icon type: {:?}, size: {}",
            String::from_utf8_lossy(icon_type),
            icon_size
        );

        // Check if we have enough data
        if offset + icon_size > icns_data.len() {
            warn!("Icon data extends beyond file bounds");
            break;
        }

        // Extract icon data
        let icon_data = &icns_data[offset..offset + icon_size];

        // Look for PNG icons (ic07-ic14 are PNG format)
        if icon_type == b"ic07"
            || icon_type == b"ic08"
            || icon_type == b"ic09"
            || icon_type == b"ic10"
            || icon_type == b"ic11"
            || icon_type == b"ic12"
            || icon_type == b"ic13"
            || icon_type == b"ic14"
        {
            // Check if it's actually PNG data
            if icon_data.len() >= 8 && &icon_data[0..8] == b"\x89PNG\r\n\x1a\n" {
                debug!(
                    "Found PNG icon: {:?}, size: {}",
                    String::from_utf8_lossy(icon_type),
                    icon_data.len()
                );
                return Ok(Some(icon_data.to_vec()));
            }
        }

        // Also check for "icp4", "icp5", "icp6" which are PNG icons
        if icon_type == b"icp4" || icon_type == b"icp5" || icon_type == b"icp6" {
            if icon_data.len() >= 8 && &icon_data[0..8] == b"\x89PNG\r\n\x1a\n" {
                debug!(
                    "Found PNG icon: {:?}, size: {}",
                    String::from_utf8_lossy(icon_type),
                    icon_data.len()
                );
                return Ok(Some(icon_data.to_vec()));
            }
        }

        offset += icon_size;
    }

    debug!("No PNG icon found in icns file, creating fallback");
    // If no PNG found, create a fallback icon
    create_fallback_icon().map(Some)
}

/// Creates a simple fallback icon as PNG
///
/// This function generates a simple 32x32 gradient icon as a fallback
/// when no icon can be extracted from an application bundle.
///
/// # Returns
///
/// Returns PNG data for a simple gradient icon.
///
/// # Examples
///
/// ```rust,ignore
/// use swii_lib::macos_window::create_fallback_icon;
///
/// let fallback_png = create_fallback_icon();
/// match fallback_png {
///     Ok(png_data) => println!("Created fallback icon with {} bytes", png_data.len()),
///     Err(e) => println!("Failed to create fallback icon: {}", e),
/// }
/// ```
fn create_fallback_icon() -> Result<Vec<u8>, WindowError> {
    use image::{ImageBuffer, Rgba};
    use std::io::Cursor;

    debug!("Creating fallback icon");

    // Create a simple 32x32 icon with a gradient
    let width = 32;
    let height = 32;
    let mut img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = (x as f32 / width as f32 * 255.0) as u8;
        let g = (y as f32 / height as f32 * 255.0) as u8;
        let b = 128;
        let a = 255;
        *pixel = Rgba([r, g, b, a]);
    }

    // Convert to PNG
    let mut png_data = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| WindowError::IconExtraction {
            message: format!("Failed to create fallback PNG: {}", e),
        })?;

    debug!("Created fallback icon with {} bytes", png_data.len());
    Ok(png_data)
}

