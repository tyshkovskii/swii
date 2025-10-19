// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod command_wrapper;
mod devtools;
mod editor_config;
mod logger;
mod macos_accessibility;
mod macos_window;
mod title_parser;
mod types;
mod window_focus;
mod window_list;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            println!("[RUST] Starting Tauri application setup");
            
            #[cfg(target_os = "macos")]
            {
                println!("[RUST] Setting macOS activation policy to Accessory");
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            #[cfg(desktop)]
            {
                println!("[RUST] Initializing global shortcut plugin");
                app.handle()
                    .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
                println!("[RUST] Global shortcut plugin initialized successfully");
            }

            #[cfg(desktop)]
            {
                println!("[RUST] Building tray icon");
                tauri::tray::TrayIconBuilder::new()
                    .on_tray_icon_event(|tray_handle, event| {
                        tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
                    })
                    .build(app)?;
                println!("[RUST] Tray icon built successfully");
            }

            println!("[RUST] Setup completed successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            window_list::list_editor_windows,
            window_focus::bring_window_to_front,
            logger::log_from_frontend,
            logger::log_from_frontend_with_data,
            devtools::open_devtools
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
