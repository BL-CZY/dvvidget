use std::cell::RefMut;
use std::sync::Arc;

use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::renderer::app::AppContext;
use crate::daemon::{
    renderer::config::AppConf,
    structs::{DaemonCmd, DaemonEvt, Dvoty},
};

use super::class::adjust_class;
use super::entry::{DvotyEntry, DvotyUIEntry};

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
            uuid: None,
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
    sender: UnboundedSender<DaemonEvt>,
) {
    let row =
        super::entry::create_base_entry(&config.dvoty.url_icon, &keyword, "Click to open", sender);

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Url { url: keyword }, row.clone()));

    if context.dvoty.dvoty_entries.len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty.dvoty_entries);
    }

    list.append(&row);
}
