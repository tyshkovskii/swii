use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[tauri::command]
pub fn log_from_frontend(level: LogLevel, tag: String, message: String) {
    let prefix = match level {
        LogLevel::Debug => "🔍 [DEBUG]",
        LogLevel::Info => "ℹ️  [INFO]",
        LogLevel::Warn => "⚠️  [WARN]",
        LogLevel::Error => "❌ [ERROR]",
    };
    
    println!("{} [{}] {}", prefix, tag, message);
}

#[tauri::command]
pub fn log_from_frontend_with_data(level: LogLevel, tag: String, message: String, data: serde_json::Value) {
    let prefix = match level {
        LogLevel::Debug => "🔍 [DEBUG]",
        LogLevel::Info => "ℹ️  [INFO]",
        LogLevel::Warn => "⚠️  [WARN]",
        LogLevel::Error => "❌ [ERROR]",
    };
    
    println!("{} [{}] {} | Data: {}", prefix, tag, message, serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string()));
}

