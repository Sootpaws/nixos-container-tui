use super::utils;
use super::{ContainerControls, Main};
use cursive::view::ViewWrapper;
use cursive::views::{ListView, Panel, ScrollView};

/// Wrapper for the main container list
pub struct ContainerList {
    inner: Panel<ScrollView<ListView>>,
}

impl ContainerList {
    /// Create a container list TUI from a list of container names
    pub fn new(containers: &Vec<String>) -> Self {
        let mut list = ListView::new().on_select(|root, container| {
            let main = Main::get_self(root);
            main.get_container_log().show(container);
        });
        for container in containers {
            list.add_child(container, ContainerControls::new());
        }
        Self {
            inner: Panel::new(ScrollView::new(list)).title("Containers"),
        }
    }

    /// Get the container view for a given name
    pub fn get_container(&mut self, name: &str) -> Option<&mut ContainerControls> {
        utils::get_list_child(self.inner.get_inner_mut().get_inner_mut(), name)
            .and_then(|v| v.downcast_mut::<ContainerControls>())
    }
}

impl ViewWrapper for ContainerList {
    cursive::wrap_impl!(self.inner: Panel<ScrollView<ListView>>);
}
