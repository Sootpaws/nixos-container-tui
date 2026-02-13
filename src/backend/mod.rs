use tokio::sync::mpsc;
use tokio::task;
use tokio_stream::StreamExt;
use zbus::Connection;
use anyhow::{Context, Result, anyhow};
use std::fs;
use messages::{NamedUpdate, Update, ContainerState};
use proxies::{ManagerProxy, UnitProxy};

/// Data structures for communicating with the backend
pub mod messages;

/// zbus/dbus proxies for communicating with systemd
// TODO: Strip out all the unused parts
#[allow(clippy::type_complexity)]
mod proxies;

/// Set up backend communication with systemd over dbus
pub async fn start_backend() -> Result<()> {
    // Connect to systemd over dbus
    let connection = Connection::system().await.unwrap();
    // Get list of containers to monitor
    let containers = get_containers()?;
    // Create channel for recieving updates from monitors
    let (send, mut recv) = mpsc::unbounded_channel();
    // Spawn tasks for monitoring each container
    for container in &containers {
        task::spawn(monitor_container(
            container.clone(),
            send.clone(),
            connection.clone(),
        ));
    }
    println!("Monitors spawned");
    while let Some(msg) = recv.recv().await {
        println!("Update: {:?}", msg);
    }
    Ok(())
}

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
async fn monitor_container(
    container: String,
    channel: mpsc::UnboundedSender<NamedUpdate>,
    connection: Connection,
) {
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
            dbg!(state);
            log!(c, s, "Changed")
        }
        Ok(())
    };
    // Actually invoke the setup closure, and report any errors
    match inner().await.context("Failed to set up monitoring") {
        Ok(()) => (),
        Err(error) => channel
            .send(NamedUpdate {
                container_name: container,
                inner: Update::Error(error),
            })
            .unwrap(),
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
