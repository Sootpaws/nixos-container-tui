use backend::messages::{ContainerState, NamedUpdate, Update};
use cursive::Cursive;
use cursive::view::{Nameable, View};
use cursive::views::{Button, LinearLayout, ListChild, ListView, ScrollView};
use tokio::task;

/// Backend for communicating with systemd over dbus
mod backend;

#[tokio::main]
async fn main() {
    // Start the backend
    let (mut recv, containers) = backend::start_backend().await.unwrap();

    // Create the list of containers and their controls/info
    let container_list = {
        let mut list = ListView::new();
        for container in &containers {
            let controls = LinearLayout::horizontal().child(Button::new("[Unknown]", |_| {}));
            list.add_child(container, controls);
        }
        list.with_name(CONTAINER_LIST_NAME)
    };

    let mut root = cursive::default();
    root.add_global_callback('q', |s| s.quit());
    root.add_layer(ScrollView::new(container_list));

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
    let mut container_list = root.find_name::<ListView>(CONTAINER_LIST_NAME).unwrap();
    let controls = get_list_child(&mut container_list, &message.container_name)
        .and_then(|v| v.downcast_mut::<LinearLayout>())
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
            let state_button = controls
                .get_child_mut(0)
                .and_then(|v| v.downcast_mut::<Button>())
                .unwrap();
            state_button.set_label(text);
            state_button.set_enabled(enabled);
        }
        // TODO: Handle these
        Update::Log(_) => (),
        Update::Error(_) => (),
    }
}

const CONTAINER_LIST_NAME: &str = "container_list";

/// Get mutable access to a ListView item by label
fn get_list_child<'a>(view: &'a mut ListView, label: &str) -> Option<&'a mut Box<dyn View>> {
    view.children()
        .iter()
        .position(|child| match child {
            ListChild::Row(child_label, _) => child_label == label,
            _ => false,
        })
        .map(|index| match view.row_mut(index) {
            ListChild::Row(_, view) => view,
            _ => unreachable!(),
        })
}
