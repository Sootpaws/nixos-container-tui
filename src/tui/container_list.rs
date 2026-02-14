use super::container_controls::ContainerControls;
use super::utils;
use cursive::view::ViewWrapper;
use cursive::views::{ListView, ScrollView};

/// Wrapper for the main container list
pub struct ContainerList {
    inner: ScrollView<ListView>,
}

impl ContainerList {
    /// Create a container list TUI from a list of container names
    pub fn new(containers: &Vec<String>) -> Self {
        let mut list = ListView::new();
        for container in containers {
            list.add_child(container,  ContainerControls::new());
        }
        Self {
            inner: ScrollView::new(list),
        }
    }

    /// Get the container view for a given name
    pub fn get_container(&mut self, name: &str) -> Option<&mut ContainerControls> {
        utils::get_list_child(self.inner.get_inner_mut(), name)
            .and_then(|v| v.downcast_mut::<ContainerControls>())
    }
}

impl ViewWrapper for ContainerList {
    cursive::wrap_impl!(self.inner: ScrollView<ListView>);
}
