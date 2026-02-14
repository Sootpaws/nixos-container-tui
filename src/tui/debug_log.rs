use anyhow::Error;
use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, TextView};

pub struct DebugLog {
    inner: LinearLayout,
}

impl DebugLog {
    pub fn new() -> Self {
        Self {
            inner: LinearLayout::vertical(),
        }
    }

    pub fn log(&mut self, container: &str, log: &str) {
        self.add(format!("[LOG] {container} - {log}"));
    }

    pub fn error(&mut self, container: &str, error: Error) {
        self.add(format!("[ERROR] {container} - {error:#}"));
    }

    fn add(&mut self, line: String) {
        self.inner.add_child(TextView::new(line));
    }
}

impl ViewWrapper for DebugLog {
    cursive::wrap_impl!(self.inner: LinearLayout);
}
