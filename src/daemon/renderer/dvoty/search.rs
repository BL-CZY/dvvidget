use std::cell::RefMut;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{GestureClick, ListBox};

use crate::daemon::renderer::app::AppContext;
use crate::daemon::renderer::config::AppConf;

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
) {
    let row = create_base_entry(&config.dvoty.serach_icon, &keyword, "Click to search");

    let gesture_click = GestureClick::new();
    let keyword_clone = keyword.clone();
    gesture_click.connect_pressed(move |_, _, _, _| {
        let keyword_clone = keyword_clone.clone();
        spawn_keyword(keyword_clone);
    });

    row.add_controller(gesture_click);

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Search { keyword }, row.clone()));

    context.dvoty.cur_ind = 0;

    adjust_class(0, 0, &mut context.dvoty.dvoty_entries);

    list.append(&row);
}
