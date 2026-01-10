//! Console API (re-export from js_engine).

// Console implementation is in js_engine crate
// This module provides types and utilities for console logging

/// Console log level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

/// Console log entry.
#[derive(Clone, Debug)]
pub struct ConsoleLogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub stack_trace: Option<String>,
}

/// Console log buffer.
pub struct ConsoleBuffer {
    entries: Vec<ConsoleLogEntry>,
    max_entries: usize,
}

impl ConsoleBuffer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: String) {
        let entry = ConsoleLogEntry {
            level,
            message,
            timestamp: std::time::SystemTime::now(),
            stack_trace: None,
        };

        self.entries.push(entry);

        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    pub fn entries(&self) -> &[ConsoleLogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for ConsoleBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}
