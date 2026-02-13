/// Backend for communicating with systemd over dbus
mod backend;

#[tokio::main]
async fn main() {
    backend::start_backend().await.unwrap();
}

// use cursive::views::{ListView, ScrollView, Button};
/*
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
*/
