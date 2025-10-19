//! Common types used across the application.

use serde::Serialize;

/// Window information structure
#[derive(Debug, Serialize)]
pub struct WindowInfo {
    pub app_name: String,
    pub window_name: Option<String>,
    pub pid: i32,
    pub window_number: u32,
    pub project: Option<String>,
    pub active_editor_tab: Option<String>,
    pub app_icon: Option<String>,
}

