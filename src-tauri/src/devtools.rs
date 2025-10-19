//! DevTools management for development mode

use tauri::Manager;

#[tauri::command]
pub async fn open_devtools(
    app: tauri::AppHandle,
    _args: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    crate::command_wrapper::create_command("open_devtools", _args, |ctx| {
        ctx.logger.info("Opening DevTools");
        
        #[cfg(debug_assertions)]
        {
            if let Some(window) = app.get_webview_window("swii") {
                window.open_devtools();
                ctx.logger.info("DevTools opened successfully");
            } else {
                ctx.logger.error("Could not find 'swii' window");
                return Err("Could not find 'swii' window".to_string());
            }
        }
        
        #[cfg(not(debug_assertions))]
        {
            ctx.logger.info("DevTools not available in release mode");
        }
        
        Ok(())
    })
    .await
}
