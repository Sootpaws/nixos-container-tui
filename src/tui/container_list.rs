use super::utils;
use cursive::view::ViewWrapper;
use cursive::views::{Button, LinearLayout, ListView, ScrollView};

/// Wrapper for the main container list
pub struct ContainerList {
    inner: ScrollView<ListView>,
}

impl ContainerList {
    /// Create a container list TUI from a list of container names
    pub fn new(containers: &Vec<String>) -> Self {
        let mut list = ListView::new();
        for container in containers {
            let controls = LinearLayout::horizontal().child(Button::new("[Unknown]", |_| {}));
            list.add_child(container, controls);
        }
        Self {
            inner: ScrollView::new(list),
        }
    }

    /// Get the container view for a given name
    pub fn get_container(&mut self, name: &str) -> Option<&mut LinearLayout> {
        utils::get_list_child(self.inner.get_inner_mut(), name)
            .and_then(|v| v.downcast_mut::<LinearLayout>())
    }
}

impl ViewWrapper for ContainerList {
    cursive::wrap_impl!(self.inner: ScrollView<ListView>);
}
