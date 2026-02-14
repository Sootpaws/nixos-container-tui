use backend::messages::{ContainerState, NamedUpdate, Update};
use cursive::Cursive;
use cursive::views::LayerPosition;
use tokio::task;
use tui::container_list::ContainerList;

/// Backend for communicating with systemd over dbus
mod backend;

/// TUI creation and direct modification
mod tui;

#[tokio::main]
async fn main() {
    // Start the backend
    let (mut recv, containers) = backend::start_backend().await.unwrap();

    // Create the TUI
    let mut root = cursive::default();
    create_tui(&mut root, &containers);

    // Run the Cursive event loop, which checking for and
    // handling backend messages
    let mut runner = root.runner();
    loop {
        // Cursive event loop
        let cursive_refresh = runner.step();
        if !runner.is_running() {
            break;
        }

        // Backend messages
        let backend_refresh = if let Ok(msg) = recv.try_recv() {
            handle_message(&mut runner, msg);
            true
        } else {
            false
        };

        // Refresh if anything happened
        if cursive_refresh || backend_refresh {
            runner.refresh();
        }

        // Give backend tasks a chance to run
        task::yield_now().await;
    }
}

/// Create the TUI
fn create_tui(root: &mut Cursive, containers: &Vec<String>) {
    // Set up the main TUI
    root.add_global_callback('q', |s| s.quit());
    root.add_layer(ContainerList::new(containers));
}

/// Update the TUI given a backend message
fn handle_message(root: &mut Cursive, message: NamedUpdate) {
    let container_list = root
        .screen_mut()
        .get_mut(LayerPosition::FromFront(0))
        .and_then(|v| v.downcast_mut::<ContainerList>())
        .unwrap();
    let controls = container_list
        .get_container(&message.container_name)
        .unwrap();
    match message.inner {
        Update::State(state) => {
            // Get updated settings for state button
            let (text, enabled) = match state {
                ContainerState::Up => ("UP", true),
                ContainerState::Down => ("DOWN", true),
                ContainerState::Starting => ("STARTING", false),
                ContainerState::Stopping => ("STOPPING", false),
                ContainerState::Reloading => ("RELOADING", false),
                ContainerState::Refreshing => ("REFRESHING", false),
                ContainerState::Failed => ("FAILED", true),
                ContainerState::Maintenance => ("MAINTENANCE", true),
            };
            // Update button
            let state_button = controls.get_state_button();
            state_button.set_label(text);
            state_button.set_enabled(enabled);
        }
        // TODO: Handle these
        Update::Log(_) => (),
        Update::Error(_) => (),
    }
}
