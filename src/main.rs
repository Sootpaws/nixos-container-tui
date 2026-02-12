use cursive::views::{ListView, ScrollView, Button};

fn main() {
    let mut r = cursive::default();
    r.add_global_callback('q', |s| s.quit());
    r.add_layer(
        ScrollView::new(ListView::new()
            .child("a", Button::new("meow", |_|{}))
            .child("b", Button::new("mrrp", |_|{}))
            .child("c", Button::new("nyaa", |_|{}))
            .child("d", Button::new("meow", |_|{}))
    ));
    r.run()
}
