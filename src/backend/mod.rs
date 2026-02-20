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
use utils::log;
use zbus::Connection;

/// Data structures for communicating with the backend
pub mod messages;

/// zbus/dbus proxies for communicating with systemd
// TODO: Strip out all the unused parts
#[allow(clippy::type_complexity)]
mod proxies;

/// Backend helper macros
mod utils;

/// Set up backend communication with systemd over dbus
pub async fn start_backend() -> Result<(Receiver, Vec<&'static str>)> {
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
            container,
            send.clone(),
            connection.clone(),
        ));
        task::spawn(monitor_container_log(container, send.clone()));
    }
    // Return backend message reciever
    Ok((recv, containers))
}

/// Type of the reciever for messages from the backend
pub type Receiver = mpsc::UnboundedReceiver<NamedUpdate>;

/// Type of senders for transmitting messages out of the backend
type Sender = mpsc::UnboundedSender<NamedUpdate>;

utils::report_async! {
    /// Monitor the status of a container
    monitor_container_status[c, s](connection: Connection) {
        let service_name = utils::service_name(c);
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
                container_name: c,
                inner: Update::State(state),
            })
            .expect("Channel should always be open");
        }
        Ok(())
    }
    "Failed to set up status monitoring"
}

utils::report_async! {
    /// Monitor logs from a container
    monitor_container_log[c, s]() {
        log!(c, s, "Requesting logs");
        let mut child = Command::new("journalctl")
            .args([
                "--no-hostname",
                "--follow",
                "--unit",
                &utils::service_name(c),
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
                container_name: c,
                inner: Update::ContainerLog(line),
            })
            .expect("Channel should always be open");
        }
        Ok(())
    }
    "Failed to set up log monitoring"
}

/// Get the list of container names
///
/// This leaks the container name strings to make cheap, copyable identifiers
fn get_containers() -> Result<Vec<&'static str>> {
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
                .map(|(name, _)| name.to_string().leak() as &'static str)
                .ok_or(anyhow!("Container config name is not of expected form"))
        })
        .collect::<Result<Vec<_>>>()?;
    configs.sort();
    Ok(configs)
}
