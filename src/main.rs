use backend::messages::{ContainerState, NamedUpdate, Update};
use cursive::Cursive;
use tokio::task;
use tui::Main;

/// Backend for communicating with systemd over dbus
mod backend;

/// TUI creation and direct modification
mod tui;

#[tokio::main]
async fn main() {
    // Start the backend
    let (mut recv, containers, send) = backend::start_backend().await.unwrap();

    // Create the TUI
    let mut root = cursive::default();
    root.set_user_data(send);
    Main::create(&mut root, &containers);

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

/// Update the TUI given a backend message
fn handle_message(root: &mut Cursive, message: NamedUpdate) {
    let main = Main::get_self(root);
    let container_list = main.get_container_list();
    let controls = container_list.get_container(message.container_name);
    match message.inner {
        Update::State(state) => {
            // Get updated settings for state button
            let (text, enabled, action): (_, _, &(dyn Fn(&mut _, _) + Sync)) = match state {
                ContainerState::Up => ("UP", true, &stop_container),
                ContainerState::Down => ("DOWN", true, &start_container),
                ContainerState::Starting => ("STARTING", false, &|_, _| {}),
                ContainerState::Stopping => ("STOPPING", false, &|_, _| {}),
                ContainerState::Reloading => ("RELOADING", false, &|_, _| {}),
                ContainerState::Refreshing => ("REFRESHING", false, &|_, _| {}),
                ContainerState::Failed => ("FAILED", true, &start_container),
                ContainerState::Maintenance => ("MAINTENANCE", true, &|_, _| {}),
            };
            // Update button
            let state_button = controls.get_state_button();
            state_button.set_label(text);
            state_button.set_enabled(enabled);
            state_button.set_callback(move |root| action(root, message.container_name));
        }
        Update::ContainerLog(log) => main.get_container_log().log(message.container_name, log),
        Update::Log(log) => main.get_debug_log().log(message.container_name, &log),
        Update::Error(error) => main.get_debug_log().error(message.container_name, error),
    }
}

fn start_container(root: &mut Cursive, container_name: &'static str) {
    task::spawn(backend::start_container(
        container_name,
        get_backend_channel(root),
    ));
}

fn stop_container(root: &mut Cursive, container_name: &'static str) {
    task::spawn(backend::stop_container(
        container_name,
        get_backend_channel(root),
    ));
}

fn get_backend_channel(root: &mut Cursive) -> backend::Sender {
    root.user_data::<backend::Sender>()
        .expect("Backend channel should be in user data")
        .clone()
}
