use cursive::view::{Nameable, ViewWrapper};
use cursive::views::{LinearLayout, Panel, StackView, TextView};

pub struct ContainerLog {
    inner: StackView,
}

impl ContainerLog {
    pub fn new(containers: &Vec<String>) -> Self {
        let mut inner = StackView::new();
        for container in containers {
            inner.add_layer(
                Panel::new(LinearLayout::vertical())
                    .title(format!("Logs - {container}"))
                    .with_name(container),
            );
        }
        let mut out = Self { inner };
        out.show(&containers[0]);
        out
    }

    pub fn log(&mut self, container: &str, log: String) {
        let layer = self.inner.find_layer_from_name(container).unwrap();
        self.inner
            .get_mut(layer)
            .unwrap()
            .downcast_mut::<LinearLayout>()
            .unwrap()
            .add_child(TextView::new(log));
    }

    pub fn show(&mut self, container: &str) {
        let layer = self.inner.find_layer_from_name(container).unwrap();
        self.inner.move_to_front(layer);
    }
}

impl ViewWrapper for ContainerLog {
    cursive::wrap_impl!(self.inner: StackView);
}
