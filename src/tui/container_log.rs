use cursive::view::{Nameable, ViewWrapper};
use cursive::views::stack_view::{Fullscreen, NoShadow};
use cursive::views::{
    HideableView, LayerPosition, LinearLayout, NamedView, Panel, ScrollView, StackView, TextView,
};

pub struct ContainerLog {
    inner: StackView,
}

impl ContainerLog {
    pub fn new(containers: &Vec<String>) -> Self {
        let mut inner = StackView::new();
        for container in containers {
            inner.add_layer(Fullscreen(NoShadow(
                HideableView::new(
                    Panel::new(ScrollView::new(LinearLayout::vertical()))
                        .title(format!("Logs - {container}"))
                        .with_name(container),
                )
                .hidden(),
            )));
        }
        let mut out = Self { inner };
        out.show(&containers[0]);
        out
    }

    pub fn log(&mut self, container: &str, log: String) {
        let layer = self.inner.find_layer_from_name(container).unwrap();
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
        let layer = self.inner.find_layer_from_name(container).unwrap();
        self.get(layer).unhide();
    }

    fn get(
        &mut self,
        pos: LayerPosition,
    ) -> &mut HideableView<NamedView<Panel<ScrollView<LinearLayout>>>> {
        self.inner.get_mut(pos).unwrap().downcast_mut().unwrap()
    }
}

impl ViewWrapper for ContainerLog {
    cursive::wrap_impl!(self.inner: StackView);
}
