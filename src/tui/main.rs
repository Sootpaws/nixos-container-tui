use super::ContainerList;
use cursive::Cursive;
use cursive::view::ViewWrapper;
use cursive::views::{LayerPosition, LinearLayout};

/// Wrapper around the entire TUI
pub struct Main {
    inner: LinearLayout,
}

impl Main {
    /// Create the TUI
    pub fn create(root: &mut Cursive, containers: &Vec<String>) {
        root.add_global_callback('q', |s| s.quit());
        root.add_layer(Self::new(containers));
    }

    /// Get the main TUI wrapper from the cursive root
    pub fn get_self(root: &mut Cursive) -> &mut Self {
        root.screen_mut()
            .get_mut(LayerPosition::FromFront(0))
            .and_then(|v| v.downcast_mut::<Self>())
            .unwrap()
    }

    /// Get the container list
    pub fn get_container_list(&mut self) -> &mut ContainerList {
        self.inner
            .get_child_mut(0)
            .and_then(|v| v.downcast_mut::<ContainerList>())
            .unwrap()
    }

    /// Create the TUI with a given list of containers
    fn new(containers: &Vec<String>) -> Self {
        let container_list = ContainerList::new(containers);
        let inner = LinearLayout::horizontal().child(container_list);
        Self { inner }
    }
}

impl ViewWrapper for Main {
    cursive::wrap_impl!(self.inner: LinearLayout);
}
