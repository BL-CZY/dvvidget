use std::cell::RefMut;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{GestureClick, ListBox};
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::renderer::app::AppContext;
use crate::daemon::{
    renderer::config::AppConf,
    structs::{DaemonCmd, DaemonEvt, Dvoty},
};

use super::base::{adjust_class, DvotyEntry, DvotyUIEntry};

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

pub fn spawn_url(keyword: String) {
    let keyword_clone = keyword.clone();
    tokio::spawn(async move {
        open::that(keyword_clone).unwrap_or_else(|e| println!("Dvoty: Can't open url: {}", e));
    });
}

pub fn populate_url_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    keyword: String,
    context: &mut RefMut<AppContext>,
) {
    let row = super::base::create_base_entry(config, ":", &keyword, "Click to open");

    let gesture_click = GestureClick::new();
    let keyword_clone = keyword.clone();
    gesture_click.connect_pressed(move |_, _, _, _| {
        let keyword_clone = keyword_clone.clone();
        spawn_url(keyword_clone);
    });

    row.add_controller(gesture_click);

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Url { url: keyword }, row.clone()));

    context.dvoty.cur_ind = 0;

    adjust_class(0, 0, &mut context.dvoty.dvoty_entries);

    list.append(&row);
}
