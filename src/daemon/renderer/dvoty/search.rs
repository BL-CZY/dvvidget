use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{EventControllerKey, GestureClick, ListBox};

use crate::daemon::renderer::config::AppConf;

use super::base::create_base_entry;

fn spawn_keyword(keyword: String) {
    let keyword_clone = keyword.clone();
    tokio::spawn(async move {
        open::that(format!("https://www.google.com/search?q={}", keyword_clone))
            .unwrap_or_else(|e| println!("Dvoty: Can't perform search: {}", e));
    });
}

pub fn populate_search_entry(config: Arc<AppConf>, list: &ListBox, keyword: String) {
    let row = create_base_entry(config, "/", &keyword, "Click to search");

    let gesture_click = GestureClick::new();
    let keyword_clone = keyword.clone();
    gesture_click.connect_pressed(move |_, _, _, _| {
        let keyword_clone = keyword_clone.clone();
        spawn_keyword(keyword_clone);
    });

    let key_controller = EventControllerKey::new();

    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Return {
            let keyword_clone = keyword.clone();
            spawn_keyword(keyword_clone);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });

    row.add_controller(gesture_click);
    row.add_controller(key_controller);

    list.append(&row);
}
