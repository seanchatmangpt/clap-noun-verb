//! Logger Plugin - Structured logging with multiple levels
//! See PLUGIN_IMPLEMENTATION_GUIDE.md for full specification

use crate::plugin::{Plugin, PluginCapability, PluginMetadata};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO",
                LogLevel::Warn => "WARN",
                LogLevel::Error => "ERROR",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone)]
pub struct LoggerPlugin {
    logs: Arc<Mutex<Vec<LogEntry>>>,
    level_filter: Arc<Mutex<LogLevel>>,
    loaded: bool,
}

impl LoggerPlugin {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            level_filter: Arc::new(Mutex::new(LogLevel::Info)),
            loaded: false,
        }
    }

    pub fn set_level(&self, level: LogLevel) -> crate::Result<()> {
        let mut filter = self.level_filter.lock().map_err(|_| {
            crate::NounVerbError::MiddlewareError("Log level lock failed".to_string())
        })?;
        *filter = level;
        Ok(())
    }

    pub fn log(&self, message: &str) -> crate::Result<()> {
        self.log_at_level(LogLevel::Info, message)
    }

    pub fn debug(&self, message: &str) -> crate::Result<()> {
        self.log_at_level(LogLevel::Debug, message)
    }

    pub fn info(&self, message: &str) -> crate::Result<()> {
        self.log_at_level(LogLevel::Info, message)
    }

    pub fn warn(&self, message: &str) -> crate::Result<()> {
        self.log_at_level(LogLevel::Warn, message)
    }

    pub fn error(&self, message: &str) -> crate::Result<()> {
        self.log_at_level(LogLevel::Error, message)
    }

    fn log_at_level(&self, level: LogLevel, message: &str) -> crate::Result<()> {
        let mut logs = self
            .logs
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Log lock failed".to_string()))?;

        logs.push(LogEntry { timestamp: SystemTime::now(), level, message: message.to_string() });

        Ok(())
    }

    pub fn get_logs(&self) -> crate::Result<Vec<LogEntry>> {
        let logs = self
            .logs
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Log lock failed".to_string()))?;
        Ok(logs.clone())
    }

    pub fn count_logs(&self) -> crate::Result<usize> {
        let logs = self
            .logs
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Log lock failed".to_string()))?;
        Ok(logs.len())
    }

    pub fn clear_logs(&self) -> crate::Result<()> {
        let mut logs = self
            .logs
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Log lock failed".to_string()))?;
        logs.clear();
        Ok(())
    }

    pub fn get_logs_by_level(&self, level: &str) -> crate::Result<Vec<LogEntry>> {
        let logs = self
            .logs
            .lock()
            .map_err(|_| crate::NounVerbError::MiddlewareError("Log lock failed".to_string()))?;

        let level_str = level.to_uppercase();
        Ok(logs.iter().filter(|entry| entry.level.to_string() == level_str).cloned().collect())
    }
}

impl Default for LoggerPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LoggerPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoggerPlugin").finish()
    }
}

impl Plugin for LoggerPlugin {
    fn name(&self) -> &str {
        "logger"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(self.name(), self.version())
            .with_description("Structured logging with levels")
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Hook]
    }

    fn load(&mut self) -> crate::Result<()> {
        self.loaded = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chicago-TDD: Integration tests with real logger
    #[test]
    fn test_logger_basic_logging_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        plugin.log("Basic message").unwrap();

        let logs = plugin.get_logs().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Basic message");
    }

    #[test]
    fn test_logger_multiple_levels_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        plugin.debug("Debug message").unwrap();
        plugin.info("Info message").unwrap();
        plugin.warn("Warning message").unwrap();
        plugin.error("Error message").unwrap();

        let logs = plugin.get_logs().unwrap();
        assert_eq!(logs.len(), 4);
    }

    #[test]
    fn test_logger_filter_by_level_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        plugin.info("Info 1").unwrap();
        plugin.warn("Warn 1").unwrap();
        plugin.error("Error 1").unwrap();
        plugin.info("Info 2").unwrap();

        let warns = plugin.get_logs_by_level("WARN").unwrap();
        assert_eq!(warns.len(), 1);
        assert_eq!(warns[0].message, "Warn 1");

        let infos = plugin.get_logs_by_level("INFO").unwrap();
        assert_eq!(infos.len(), 2);
    }

    #[test]
    fn test_logger_count_logs_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        for i in 0..5 {
            plugin.log(&format!("Message {}", i)).unwrap();
        }

        let count = plugin.count_logs().unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_logger_clear_logs_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        plugin.log("Message 1").unwrap();
        plugin.log("Message 2").unwrap();

        assert_eq!(plugin.count_logs().unwrap(), 2);

        plugin.clear_logs().unwrap();

        assert_eq!(plugin.count_logs().unwrap(), 0);
    }

    #[test]
    fn test_logger_log_levels_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        plugin.debug("Debug").unwrap();
        plugin.info("Info").unwrap();
        plugin.warn("Warn").unwrap();
        plugin.error("Error").unwrap();

        let all_logs = plugin.get_logs().unwrap();
        assert_eq!(all_logs.len(), 4);

        let errors = plugin.get_logs_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_logger_retrieve_all_logs_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        plugin.info("First").unwrap();
        plugin.info("Second").unwrap();
        plugin.info("Third").unwrap();

        let logs = plugin.get_logs().unwrap();
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].message, "First");
        assert_eq!(logs[1].message, "Second");
        assert_eq!(logs[2].message, "Third");
    }

    #[test]
    fn test_logger_concurrent_logging_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        let plugin = Arc::new(plugin);
        let mut handles = vec![];

        for i in 0..10 {
            let p = Arc::clone(&plugin);
            let handle = std::thread::spawn(move || {
                for j in 0..5 {
                    p.log(&format!("Thread {} Message {}", i, j)).unwrap();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let count = plugin.count_logs().unwrap();
        assert_eq!(count, 50);
    }

    #[test]
    fn test_logger_empty_logs_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        let logs = plugin.get_logs().unwrap();
        assert!(logs.is_empty());

        let count = plugin.count_logs().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_logger_message_preservation_workflow() {
        let mut plugin = LoggerPlugin::new();
        plugin.load().unwrap();

        let messages = vec![
            "Simple message",
            "Message with numbers 12345",
            "Message with special chars !@#$%",
            "Very long message with lots of text and many words",
        ];

        for msg in &messages {
            plugin.log(msg).unwrap();
        }

        let logs = plugin.get_logs().unwrap();
        for (i, log) in logs.iter().enumerate() {
            assert_eq!(log.message, messages[i]);
        }
    }
}
