//! # Command Wrapper Module
//!
//! This module provides a generic command wrapper for Tauri commands that includes:
//! - Automatic logging of command execution
//! - Execution time tracking
//! - Standardized error handling and response format
//! - Structured logging with timestamps
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! #[tauri::command]
//! async fn my_command(args: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
//!     swii_lib::command_wrapper::create_command("my_command", args, |ctx| {
//!         ctx.logger.info("Starting my command");
//!
//!         // Your command logic here
//!         let data = "example data";
//!
//!         ctx.logger.info("Command completed successfully");
//!         Ok(data)
//!     }).await
//! }
//! ```
//!
//! ## Even Simpler Usage
//!
//! For actions that don't return data:
//! ```rust,ignore
//! #[tauri::command]
//! async fn my_action(args: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
//!     swii_lib::command_wrapper::create_command("my_action", args, |ctx| {
//!         ctx.logger.info("Performing action");
//!         // Do something...
//!         Ok(()) // Return unit for actions
//!     }).await
//! }
//! ```
//!
//! ## Benefits
//!
//! - **Consistent Logging**: Every command automatically logs start, completion, and errors
//! - **Performance Monitoring**: Execution time is tracked and included in responses
//! - **Error Handling**: Standardized error format with proper logging
//! - **Debugging**: Easy to trace command execution and identify issues
//! - **Maintainability**: Centralized command behavior management

use serde::Serialize;
use std::time::Instant;

/// Command execution context containing metadata and utilities
pub struct CommandContext {
    pub command_name: String,
    pub start_time: Instant,
    pub parameters: serde_json::Value,
    pub logger: CommandLogger,
}

impl CommandContext {
    /// Create a new command context
    pub fn new(command_name: &str, parameters: serde_json::Value) -> Self {
        Self {
            command_name: command_name.to_string(),
            start_time: Instant::now(),
            parameters,
            logger: CommandLogger::new(command_name),
        }
    }
}

/// Logger for command execution with structured logging
pub struct CommandLogger {
    pub command_name: String,
}

impl CommandLogger {
    pub fn new(command_name: &str) -> Self {
        Self {
            command_name: command_name.to_string(),
        }
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        self.log("INFO", message, None);
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        self.log("ERROR", message, None);
    }

    /// Internal logging method with reduced duplication
    fn log(&self, level: &str, message: &str, params: Option<&serde_json::Value>) {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");

        let log_message = match params {
            Some(params) => format!(
                "[{}] [{}] [{}] {} - Params: {}",
                timestamp, level, self.command_name, message, params
            ),
            None => format!(
                "[{}] [{}] [{}] {}",
                timestamp, level, self.command_name, message
            ),
        };

        println!("{}", log_message);
    }
}

/// Standardized command result wrapper
#[derive(Debug, Serialize)]
pub struct CommandResult<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: String,
    #[serde(skip_serializing)]
    pub command_name: String,
}

impl<T> CommandResult<T>
where
    T: Serialize,
{
    /// Create a successful result
    pub fn success(data: T, execution_time_ms: u64, command_name: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            execution_time_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
            command_name: command_name.to_string(),
        }
    }

    /// Create an error result
    pub fn error(error: String, execution_time_ms: u64, command_name: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            execution_time_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
            command_name: command_name.to_string(),
        }
    }

    /// Get the command name
    pub fn command_name(&self) -> &str {
        &self.command_name
    }

    /// Convert to Tauri response format
    pub fn into_tauri_response(self) -> Result<serde_json::Value, String> {
        serde_json::to_value(self).map_err(|e| e.to_string())
    }
}

/// Command executor that wraps command execution with logging
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a command with automatic logging and error handling
    pub async fn execute<F, T>(ctx: CommandContext, handler: F) -> CommandResult<T>
    where
        F: FnOnce(CommandContext) -> Result<T, String>,
        T: Serialize,
    {
        Self::log_command_start(&ctx);
        let result = Self::execute_handler(ctx, handler);
        Self::log_command_completion(&result);
        result
    }

    /// Log the start of command execution
    fn log_command_start(ctx: &CommandContext) {
        ctx.logger.info("Command started");
        if !ctx.parameters.is_null() {
            ctx.logger.info("Command has parameters");
        }
    }

    /// Execute the command handler and measure execution time
    fn execute_handler<F, T>(ctx: CommandContext, handler: F) -> CommandResult<T>
    where
        F: FnOnce(CommandContext) -> Result<T, String>,
        T: Serialize,
    {
        let start_time = ctx.start_time;
        let command_name = ctx.command_name.clone();
        let result = handler(ctx);
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(data) => CommandResult::success(data, execution_time_ms, &command_name),
            Err(error) => CommandResult::<T>::error(error, execution_time_ms, &command_name),
        }
    }

    /// Log the completion of command execution
    fn log_command_completion<T>(result: &CommandResult<T>)
    where
        T: Serialize,
    {
        let logger = CommandLogger::new(&result.command_name());

        if result.success {
            logger.info(&format!(
                "Command completed successfully in {}ms",
                result.execution_time_ms
            ));
        } else if let Some(ref error) = result.error {
            logger.error(&format!("Command failed: {}", error));
        }
    }
}

/// Helper function to create a Tauri command with minimal boilerplate
pub async fn create_command<F, T>(
    command_name: &str,
    args: Option<serde_json::Value>,
    handler: F,
) -> Result<serde_json::Value, String>
where
    F: FnOnce(CommandContext) -> Result<T, String>,
    T: Serialize,
{
    let args = args.unwrap_or(serde_json::Value::Null);
    let ctx = CommandContext::new(command_name, args);
    let result = CommandExecutor::execute(ctx, handler).await;
    result.into_tauri_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_context_creation() {
        // Note: We can't easily test with real AppHandle in unit tests
        // This test focuses on the parts we can test without Tauri dependencies

        // Test that we can create the logger
        let logger = CommandLogger::new("test_command");
        assert_eq!(logger.command_name, "test_command");
    }

    #[test]
    fn test_command_context_timing() {
        use std::thread;
        use std::time::Duration;

        // We can't create a full CommandContext without AppHandle, but we can test
        // the timing logic indirectly through the logger which has similar structure
        let start = std::time::Instant::now();
        thread::sleep(Duration::from_millis(10));
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() >= 10);
        assert!(elapsed.as_millis() < 50); // Should be reasonably quick
    }

    #[test]
    fn test_command_logger() {
        let logger = CommandLogger::new("test_logger");
        logger.info("Test info message");
        logger.error("Test error message");
    }

    #[test]
    fn test_command_result_success() {
        let data = "test data";
        let result = CommandResult::success(data, 100, "test_command");

        assert!(result.success);
        assert_eq!(result.data, Some("test data"));
        assert_eq!(result.error, None);
        assert_eq!(result.execution_time_ms, 100);
        assert!(!result.timestamp.is_empty());
        assert_eq!(result.command_name(), "test_command");
    }

    #[test]
    fn test_command_result_error() {
        let error_msg = "test error";
        let result = CommandResult::<String>::error(error_msg.to_string(), 50, "test_command");

        assert!(!result.success);
        assert_eq!(result.data, None);
        assert_eq!(result.error, Some("test error".to_string()));
        assert_eq!(result.execution_time_ms, 50);
        assert!(!result.timestamp.is_empty());
        assert_eq!(result.command_name(), "test_command");
    }

    #[test]
    fn test_command_result_serialization() {
        let result = CommandResult::success("test", 100, "test_command");
        let serialized = serde_json::to_string(&result).unwrap();

        assert!(serialized.contains("success"));
        assert!(serialized.contains("test"));
        assert!(serialized.contains("100"));
        // command_name should not be in serialized output due to #[serde(skip_serializing)]
        assert!(!serialized.contains("command_name"));
    }

    #[test]
    fn test_command_result_into_tauri_response() {
        let result = CommandResult::success("test_data", 150, "test_command");
        let tauri_response = result.into_tauri_response().unwrap();

        let response_value: serde_json::Value = serde_json::from_value(tauri_response).unwrap();
        assert_eq!(response_value["success"], true);
        assert_eq!(response_value["data"], "test_data");
        assert_eq!(response_value["execution_time_ms"], 150);
    }
}
