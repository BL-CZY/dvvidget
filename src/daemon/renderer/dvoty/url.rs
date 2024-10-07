use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{EventControllerKey, GestureClick, ListBox};
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{
    renderer::config::AppConf,
    structs::{DaemonCmd, DaemonEvt, Dvoty},
};

use super::base::DvotyEntry;

pub fn send_url(url: String, sender: UnboundedSender<DaemonEvt>) {
    let send_url = if !(url.starts_with("https://") || url.starts_with("http://")) {
        let mut res: String = String::from("https://");
        res.push_str(&url);
        res
    } else {
        url
    };

    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Url { url: send_url })),
            sender: None,
        })
        .unwrap_or_else(|e| {
            println!("Dvoty: Failed to send url: {}", e);
        });
}

fn spawn_url(keyword: String) {
    let keyword_clone = keyword.clone();
    tokio::spawn(async move {
        open::that(keyword_clone).unwrap_or_else(|e| println!("Dvoty: Can't open url: {}", e));
    });
}

pub fn populate_url_entry(config: Arc<AppConf>, list: &ListBox, keyword: String) {
    let row = super::base::create_base_entry(config, ":", &keyword, "Click to open");

    let gesture_click = GestureClick::new();
    let keyword_clone = keyword.clone();
    gesture_click.connect_pressed(move |_, _, _, _| {
        let keyword_clone = keyword_clone.clone();
        spawn_url(keyword_clone);
    });

    let key_controller = EventControllerKey::new();

    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Return {
            let keyword_clone = keyword.clone();
            spawn_url(keyword_clone);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });

    row.add_controller(gesture_click);
    row.add_controller(key_controller);

    list.append(&row);
}
