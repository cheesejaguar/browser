//! Browser history UI.

use std::time::SystemTime;

/// History entry.
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub visit_time: SystemTime,
    pub visit_count: u32,
    pub favicon: Option<String>,
}

/// History manager.
pub struct HistoryManager {
    entries: Vec<HistoryEntry>,
    id_counter: u64,
    max_entries: usize,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            id_counter: 0,
            max_entries: 10000,
        }
    }

    pub fn add_visit(&mut self, url: &str, title: &str) {
        // Check if URL already exists
        if let Some(entry) = self.entries.iter_mut().find(|e| e.url == url) {
            entry.visit_count += 1;
            entry.visit_time = SystemTime::now();
            entry.title = title.to_string();
            return;
        }

        self.id_counter += 1;
        let entry = HistoryEntry {
            id: self.id_counter,
            url: url.to_string(),
            title: title.to_string(),
            visit_time: SystemTime::now(),
            visit_count: 1,
            favicon: None,
        };

        self.entries.insert(0, entry);

        // Enforce max entries
        if self.entries.len() > self.max_entries {
            self.entries.pop();
        }
    }

    pub fn remove(&mut self, id: u64) {
        self.entries.retain(|e| e.id != id);
    }

    pub fn remove_url(&mut self, url: &str) {
        self.entries.retain(|e| e.url != url);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn clear_range(&mut self, from: SystemTime, to: SystemTime) {
        self.entries.retain(|e| e.visit_time < from || e.visit_time > to);
    }

    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        let query = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.url.to_lowercase().contains(&query)
                    || e.title.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn get_recent(&self, count: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().take(count).collect()
    }

    pub fn get_by_date(&self, date: chrono::NaiveDate) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Ok(duration) = e.visit_time.duration_since(SystemTime::UNIX_EPOCH) {
                    let secs = duration.as_secs() as i64;
                    let entry_date = chrono::DateTime::from_timestamp(secs, 0)
                        .map(|dt| dt.date_naive());
                    entry_date == Some(date)
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}
