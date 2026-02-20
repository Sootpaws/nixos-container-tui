use super::{ContainerList, ContainerLog, DebugLog};
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
            .expect("Main view should be present")
            .downcast_mut::<Self>()
            .expect("Main view should be expected type")
    }

    pub fn get_debug_log(&mut self) -> &mut DebugLog {
        self.inner
            .get_child_mut(0)
            .expect("Debug log view should be present")
            .downcast_mut::<DebugLog>()
            .expect("Debug log view should be expected type")
    }

    /// Get the container list
    pub fn get_container_list(&mut self) -> &mut ContainerList {
        self.inner
            .get_child_mut(1)
            .expect("Container list view should be present")
            .downcast_mut::<ContainerList>()
            .expect("Container list view should be expected type")
    }

    pub fn get_container_log(&mut self) -> &mut ContainerLog {
        self.inner
            .get_child_mut(2)
            .expect("Container log view should be present")
            .downcast_mut::<ContainerLog>()
            .expect("Container log view should be expected type")
    }

    /// Create the TUI with a given list of containers
    fn new(containers: &Vec<String>) -> Self {
        let debug_log = DebugLog::new();
        let container_list = ContainerList::new(containers);
        let container_log = ContainerLog::new(containers);
        let inner = LinearLayout::horizontal()
            .child(debug_log)
            .child(container_list)
            .child(container_log);
        Self { inner }
    }
}

impl ViewWrapper for Main {
    cursive::wrap_impl!(self.inner: LinearLayout);
}
