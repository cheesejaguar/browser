//! Web Workers API stub.

/// Worker type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerType {
    Classic,
    Module,
}

/// Worker options.
#[derive(Clone, Debug)]
pub struct WorkerOptions {
    pub worker_type: WorkerType,
    pub credentials: RequestCredentials,
    pub name: String,
}

impl Default for WorkerOptions {
    fn default() -> Self {
        Self {
            worker_type: WorkerType::Classic,
            credentials: RequestCredentials::SameOrigin,
            name: String::new(),
        }
    }
}

/// Request credentials mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestCredentials {
    Omit,
    SameOrigin,
    Include,
}

/// Dedicated Worker stub.
pub struct Worker {
    /// Worker script URL.
    pub url: String,
    /// Worker options.
    pub options: WorkerOptions,
    /// Whether terminated.
    pub terminated: bool,
}

impl Worker {
    pub fn new(url: &str, options: Option<WorkerOptions>) -> Self {
        Self {
            url: url.to_string(),
            options: options.unwrap_or_default(),
            terminated: false,
        }
    }

    pub fn terminate(&mut self) {
        self.terminated = true;
    }

    pub fn post_message(&self, _message: &[u8]) {
        // Would send message to worker
    }
}

/// Shared Worker stub.
pub struct SharedWorker {
    /// Worker script URL.
    pub url: String,
    /// Worker name.
    pub name: String,
}

impl SharedWorker {
    pub fn new(url: &str, name: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            name: name.unwrap_or_default().to_string(),
        }
    }
}

/// Service Worker stub.
pub struct ServiceWorker {
    /// Script URL.
    pub script_url: String,
    /// State.
    pub state: ServiceWorkerState,
}

/// Service Worker state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceWorkerState {
    Parsed,
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

/// Service Worker registration.
pub struct ServiceWorkerRegistration {
    /// Scope.
    pub scope: String,
    /// Installing worker.
    pub installing: Option<ServiceWorker>,
    /// Waiting worker.
    pub waiting: Option<ServiceWorker>,
    /// Active worker.
    pub active: Option<ServiceWorker>,
}
