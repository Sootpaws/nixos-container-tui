use anyhow::{Error, Result, anyhow};

/// An update from the backend associated with a container name
#[derive(Debug)]
pub struct NamedUpdate {
    /// The name of the associated container
    pub container_name: &'static str,
    /// The update from that container
    pub inner: Update,
}

/// An update message from the backend
#[derive(Debug)]
pub enum Update {
    /// Log messages from the monitor (not the container service)
    Log(String),
    /// Errors from the monitor (not the container service)
    Error(Error),
    /// State of the container status
    State(ContainerState),
    /// Log message from the container
    ContainerLog(String),
}

/// The state of a container service
#[derive(Debug)]
pub enum ContainerState {
    Up,
    Down,
    Starting,
    Stopping,
    Reloading,
    Refreshing,
    Failed,
    Maintenance,
}

impl ContainerState {
    /// Parse a state from a systemd message
    pub fn from_systemd(state: &str) -> Result<Self> {
        match state {
            "active" => Ok(Self::Up),
            "inactive" => Ok(Self::Down),
            "failed" => Ok(Self::Failed),
            "activating" => Ok(Self::Starting),
            "deactivating" => Ok(Self::Stopping),
            "maintenance" => Ok(Self::Maintenance),
            "reloading" => Ok(Self::Reloading),
            "refreshing" => Ok(Self::Refreshing),
            _ => Err(anyhow!("Unrecognized unit status {state}")),
        }
    }
}
