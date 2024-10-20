use std::cell::RefMut;
use std::sync::Arc;

use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::renderer::app::AppContext;
use crate::daemon::renderer::config::AppConf;
use crate::daemon::structs::DaemonEvt;

use super::class::adjust_class;
use super::entry::create_base_entry;
use super::entry::DvotyUIEntry;

pub fn spawn_keyword(keyword: String) {
    let keyword_clone = keyword.clone();
    tokio::spawn(async move {
        open::that(format!("https://www.google.com/search?q={}", keyword_clone))
            .unwrap_or_else(|e| println!("Dvoty: Can't perform search: {}", e));
    });
}

pub fn populate_search_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    keyword: String,
    context: &mut RefMut<AppContext>,
    sender: UnboundedSender<DaemonEvt>,
) {
    let row = create_base_entry(
        &config.dvoty.serach_icon,
        &keyword,
        "Click to search",
        sender,
    );

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Search { keyword }, row.clone()));

    adjust_class(0, 0, &mut context.dvoty.dvoty_entries);

    list.append(&row);
}
