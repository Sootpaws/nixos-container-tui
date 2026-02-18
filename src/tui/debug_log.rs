use anyhow::Error;
use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, Panel, TextView};

pub struct DebugLog {
    inner: Panel<LinearLayout>,
}

impl DebugLog {
    pub fn new() -> Self {
        Self {
            inner: Panel::new(LinearLayout::vertical()).title("Internal Logs"),
        }
    }

    pub fn log(&mut self, container: &str, log: &str) {
        self.add(format!("[LOG] {container} - {log}"));
    }

    pub fn error(&mut self, container: &str, error: Error) {
        self.add(format!("[ERROR] {container} - {error:#}"));
    }

    fn add(&mut self, line: String) {
        self.inner.get_inner_mut().add_child(TextView::new(line));
    }
}

impl ViewWrapper for DebugLog {
    cursive::wrap_impl!(self.inner: Panel<LinearLayout>);
}
