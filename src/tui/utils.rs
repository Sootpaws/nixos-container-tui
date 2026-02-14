use cursive::view::View;
use cursive::views::{ListChild, ListView};

/// Get mutable access to a ListView item by label
pub fn get_list_child<'a>(view: &'a mut ListView, label: &str) -> Option<&'a mut Box<dyn View>> {
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
