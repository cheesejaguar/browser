//! Downloads management.

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

/// Download manager.
pub struct DownloadManager {
    downloads: Vec<Download>,
    id_counter: u64,
    download_directory: PathBuf,
}

impl DownloadManager {
    pub fn new(download_directory: PathBuf) -> Self {
        Self {
            downloads: Vec::new(),
            id_counter: 0,
            download_directory,
        }
    }

    pub fn start_download(&mut self, url: &str, filename: &str) -> u64 {
        self.id_counter += 1;
        let id = self.id_counter;

        let download = Download {
            id,
            url: url.to_string(),
            filename: filename.to_string(),
            path: self.download_directory.join(filename),
            state: DownloadState::InProgress,
            bytes_received: 0,
            total_bytes: None,
            start_time: Instant::now(),
            end_time: None,
            error: None,
            mime_type: None,
        };

        self.downloads.push(download);
        id
    }

    pub fn update_progress(&mut self, id: u64, received: u64, total: Option<u64>) {
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == id) {
            download.bytes_received = received;
            download.total_bytes = total;
        }
    }

    pub fn complete(&mut self, id: u64) {
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == id) {
            download.state = DownloadState::Complete;
            download.end_time = Some(Instant::now());
        }
    }

    pub fn fail(&mut self, id: u64, error: &str) {
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == id) {
            download.state = DownloadState::Failed;
            download.error = Some(error.to_string());
            download.end_time = Some(Instant::now());
        }
    }

    pub fn cancel(&mut self, id: u64) {
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == id) {
            download.state = DownloadState::Cancelled;
            download.end_time = Some(Instant::now());
        }
    }

    pub fn pause(&mut self, id: u64) {
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == id) {
            if download.state == DownloadState::InProgress {
                download.state = DownloadState::Paused;
            }
        }
    }

    pub fn resume(&mut self, id: u64) {
        if let Some(download) = self.downloads.iter_mut().find(|d| d.id == id) {
            if download.state == DownloadState::Paused {
                download.state = DownloadState::InProgress;
            }
        }
    }

    pub fn remove(&mut self, id: u64) {
        self.downloads.retain(|d| d.id != id);
    }

    pub fn get(&self, id: u64) -> Option<&Download> {
        self.downloads.iter().find(|d| d.id == id)
    }

    pub fn all(&self) -> &[Download] {
        &self.downloads
    }

    pub fn in_progress(&self) -> impl Iterator<Item = &Download> {
        self.downloads.iter().filter(|d| d.state == DownloadState::InProgress)
    }

    pub fn in_progress_count(&self) -> usize {
        self.in_progress().count()
    }
}

/// A download.
#[derive(Clone, Debug)]
pub struct Download {
    pub id: u64,
    pub url: String,
    pub filename: String,
    pub path: PathBuf,
    pub state: DownloadState,
    pub bytes_received: u64,
    pub total_bytes: Option<u64>,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub error: Option<String>,
    pub mime_type: Option<String>,
}

impl Download {
    pub fn progress(&self) -> f64 {
        self.total_bytes
            .map(|total| self.bytes_received as f64 / total as f64)
            .unwrap_or(0.0)
    }

    pub fn speed(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.bytes_received as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        self.total_bytes.map(|total| {
            let remaining = total.saturating_sub(self.bytes_received);
            let speed = self.speed();
            if speed > 0.0 {
                Duration::from_secs_f64(remaining as f64 / speed)
            } else {
                Duration::MAX
            }
        })
    }
}

/// Download state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadState {
    InProgress,
    Paused,
    Complete,
    Failed,
    Cancelled,
}
