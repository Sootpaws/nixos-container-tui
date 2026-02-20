use cursive::view::ViewWrapper;
use cursive::views::{Button, LinearLayout};

/// Wrapper for the contols of an individual container
pub struct ContainerControls {
    inner: LinearLayout,
}

impl ContainerControls {
    pub fn new() -> Self {
        // Button that displays the container status and brings it up/down
        let status_button = Button::new("[Unknown]", |_| {});
        // Create inner view
        let inner = LinearLayout::horizontal().child(status_button);
        Self { inner }
    }

    pub fn get_state_button(&mut self) -> &mut Button {
        self.inner
            .get_child_mut(0)
            .expect("Container state button should be present")
            .downcast_mut::<Button>()
            .expect("Container state button should be expected type")
    }
}

impl ViewWrapper for ContainerControls {
    cursive::wrap_impl!(self.inner: LinearLayout);
}
