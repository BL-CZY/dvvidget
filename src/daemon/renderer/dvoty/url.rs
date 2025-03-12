use std::sync::Arc;

use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::daemon::{
    renderer::config::AppConf,
    structs::{DaemonCmdType, DaemonEvt, Dvoty},
};

use super::class::adjust_class;
use super::entry::{DvotyEntry, DvotyUIEntry};
use super::DvotyContext;

pub fn send_url(url: String, sender: UnboundedSender<DaemonEvt>, id: &Uuid, monitor: usize) {
    let send_url = if !(url.starts_with("https://") || url.starts_with("http://")) {
        let mut res: String = String::from("https://");
        res.push_str(&url);
        res
    } else {
        url
    };

    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Url {
                url: send_url,
                title: None,
            })),
            sender: None,
            uuid: Some(*id),
            monitors: vec![monitor],
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
    keyword: &str,
    url: String,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) {
    let row = super::entry::create_base_entry(
        &config.dvoty.url_icon,
        keyword,
        "Click to open",
        sender,
        config.clone(),
        monitor,
    );

    context.dvoty_entries[monitor].push((DvotyUIEntry::Url { url }, row.clone()));

    if context.dvoty_entries[monitor].len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty_entries[monitor]);
    }

    list.append(&row);
}
