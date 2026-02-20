use anyhow::{Context, Result, anyhow};
use messages::{ContainerState, NamedUpdate, Update};
use proxies::{ManagerProxy, UnitProxy};
use std::fs;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task;
use tokio_stream::StreamExt;
use zbus::Connection;

/// Data structures for communicating with the backend
pub mod messages;
/// zbus/dbus proxies for communicating with systemd
// TODO: Strip out all the unused parts
#[allow(clippy::type_complexity)]
mod proxies;

/// Set up backend communication with systemd over dbus
pub async fn start_backend() -> Result<(Receiver, Vec<String>)> {
    // Connect to systemd over dbus
    let connection = Connection::system()
        .await
        .context("Could not connect to DBus")?;
    // Get list of containers to monitor
    let containers = get_containers()?;
    // Create channel for recieving updates from monitors
    let (send, recv) = mpsc::unbounded_channel();
    // Spawn tasks for monitoring each container
    for container in &containers {
        task::spawn(monitor_container_status(
            container.clone(),
            send.clone(),
            connection.clone(),
        ));
        task::spawn(monitor_container_log(container.clone(), send.clone()));
    }
    // Return backend message reciever
    Ok((recv, containers))
}

/// Type of the reciever for messages from the backend
pub type Receiver = mpsc::UnboundedReceiver<NamedUpdate>;

/// Type of senders for transmitting messages out of the backend
type Sender = mpsc::UnboundedSender<NamedUpdate>;

/// Helper for reporting log messages from container monitors
macro_rules! log {
    ($name:expr, $sender:expr, $message:expr) => {
        $sender
            .send(NamedUpdate {
                container_name: $name.clone(),
                inner: Update::Log(format!($message)),
            })
            .unwrap()
    };
}

/// Monitor the status of a container
async fn monitor_container_status(container: String, channel: Sender, connection: Connection) {
    // These will get moved into the inner closure
    let c = container.clone();
    let s = channel.clone();
    // We use an inner async closure here to catch and handle ?-propagated
    // errors during the setup process
    let inner = async move || -> Result<()> {
        let service_name = format!("container@{c}.service");
        // Connect to systemd over dbus
        log!(c, s, "Connecting to systemd");
        let manager = ManagerProxy::new(&connection)
            .await
            .context("Failed to connect to systemd manager")?;
        let path = manager
            .load_unit(&service_name)
            .await
            .context("Failed to get unit path")?;
        let unit = UnitProxy::new(&connection, path)
            .await
            .context("Failed to connect to unit object")?;
        let mut state_stream = unit.receive_active_state_changed().await;
        log!(c, s, "Monitoring");
        // Listen for state changes
        while let Some(state) = state_stream.next().await {
            let state = ContainerState::from_systemd(
                &state.get().await.context("Failed to get updated state")?,
            )?;
            s.send(NamedUpdate {
                container_name: c.clone(),
                inner: Update::State(state),
            })
            .expect("Channel should always be open");
        }
        Ok(())
    };
    // Actually invoke the setup closure, and report any errors
    match inner().await.context("Failed to set up status monitoring") {
        Ok(()) => (),
        Err(error) => channel
            .send(NamedUpdate {
                container_name: container,
                inner: Update::Error(error),
            })
            .expect("Channel should always be open"),
    }
}

/// Monitor logs from a container
async fn monitor_container_log(container: String, channel: Sender) {
    let c = container.clone();
    let s = channel.clone();
    let inner = async move || -> Result<()> {
        log!(c, s, "Requesting logs");
        let mut child = Command::new("journalctl")
            .args([
                "--no-hostname",
                "--follow",
                "--unit",
                &format!("container@{c}.service"),
            ])
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to spawn journalctl")?;
        let mut reader =
            BufReader::new(child.stdout.take().expect("Child stdio should be present")).lines();
        tokio::spawn(async move { child.wait().await });
        log!(c, s, "Reading logs");
        while let Some(line) = reader
            .next_line()
            .await
            .context("Failed to read log line")?
        {
            s.send(NamedUpdate {
                container_name: c.clone(),
                inner: Update::ContainerLog(line),
            })
            .expect("Channel should always be open");
        }
        Ok(())
    };
    match inner().await.context("Failed to set up log monitoring") {
        Ok(()) => (),
        Err(error) => channel
            .send(NamedUpdate {
                container_name: container,
                inner: Update::Error(error),
            })
            .expect("Channel should always be open"),
    }
}

/// Get the list of container names
fn get_containers() -> Result<Vec<String>> {
    let mut configs = fs::read_dir("/etc/nixos-containers")
        .context("Failed to list container configs")?
        .map(|entry| {
            entry
                .context("Failed to get container config")?
                .file_name()
                .into_string()
                .map_err(|failed| {
                    anyhow!(
                        "Container config name contains invalid UTF-8: {}",
                        failed.to_string_lossy()
                    )
                })?
                .rsplit_once('.')
                .map(|(name, _)| name.to_string())
                .ok_or(anyhow!("Container config name is not of expected form"))
        })
        .collect::<Result<Vec<String>>>()?;
    configs.sort();
    Ok(configs)
}
