use cursive::view::{Nameable, ViewWrapper};
use cursive::views::stack_view::{Fullscreen, NoShadow};
use cursive::views::{
    FocusTracker, HideableView, LayerPosition, LinearLayout, NamedView, Panel, ScrollView,
    StackView, TextView,
};

pub struct ContainerLog {
    inner: FocusTracker<StackView>,
}

impl ContainerLog {
    pub fn new(containers: &Vec<&'static str>) -> Self {
        let mut inner = StackView::new();
        for container in containers {
            inner.add_layer(Fullscreen(NoShadow(
                HideableView::new(
                    Panel::new(ScrollView::new(LinearLayout::vertical()))
                        .title(format!("Logs - {container}"))
                        .with_name(*container),
                )
                .hidden(),
            )));
        }
        let mut out = Self {
            inner: FocusTracker::new(inner),
        };
        out.show(containers[0]);
        out
    }

    pub fn log(&mut self, container: &str, log: String) {
        let layer = self.get_by_name(container);
        let mut inner = self.get(layer).get_inner_mut().get_mut();
        let scroll = inner.get_inner_mut();
        let follow = scroll.is_at_bottom();
        scroll.get_inner_mut().add_child(TextView::new(log));
        if follow {
            scroll.scroll_to_bottom();
        }
    }

    pub fn show(&mut self, container: &str) {
        self.get(LayerPosition::FromFront(0)).hide();
        let layer = self.get_by_name(container);
        self.get(layer).unhide();
        self.inner.get_inner_mut().move_to_front(layer);
    }

    fn get_by_name(&mut self, container: &str) -> LayerPosition {
        self.inner
            .get_inner_mut()
            .find_layer_from_name(container)
            .expect("Container name should be valid")
    }

    fn get(
        &mut self,
        pos: LayerPosition,
    ) -> &mut HideableView<NamedView<Panel<ScrollView<LinearLayout>>>> {
        self.inner
            .get_inner_mut()
            .get_mut(pos)
            .expect("Passed possition should be valid")
            .downcast_mut()
            .expect("Child view should be expected type")
    }
}

impl ViewWrapper for ContainerLog {
    cursive::wrap_impl!(self.inner: FocusTracker<StackView>);
}
