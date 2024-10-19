use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    daemon::{
        renderer::{app::AppContext, config::AppConf},
        structs::DaemonRes,
    },
    utils::DaemonErr,
};

use super::{math, search, url};
use gtk4::{prelude::BoxExt, Box, Label, ListBoxRow, Window};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DvotyEntry {
    Empty,
    Instruction,
    Math {
        expression: String,
        result: String,
    },
    Launch {
        name: String,
        exec: String,
        icon: String,
    },
    Command {
        exec: String,
    },
    Search {
        keyword: String,
    },
    Url {
        url: String,
    },
}

#[derive(Clone)]
pub enum DvotyUIEntry {
    Instruction,
    Math { result: String },
    Launch { exec: String },
    Command { exec: String },
    Search { keyword: String },
    Url { url: String },
}

impl DvotyUIEntry {
    pub fn run(self) {
        match self {
            DvotyUIEntry::Math { result } => {
                math::set_clipboard_text(&result);
            }
            DvotyUIEntry::Search { keyword } => {
                search::spawn_keyword(keyword);
            }
            DvotyUIEntry::Url { url } => {
                url::spawn_url(url);
            }
            _ => {}
        }
    }
}

pub fn create_base_entry(config: Arc<AppConf>, icon: &str, content: &str, tip: &str) -> ListBoxRow {
    let label_begin = Label::builder()
        .use_markup(true)
        .label(format!(
            "<span show=\"ignorables\" background=\"{}\" foreground=\"{}\" size=\"x-large\"> {} </span> {}",
            config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color, icon, content
        ))
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let label_end = Label::builder()
        .use_markup(true)
        .label(tip)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::End)
        .hexpand(true)
        .build();

    let wrapper_box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["dvoty-box"])
        .build();

    wrapper_box.append(&label_begin);
    wrapper_box.append(&label_end);

    let res = ListBoxRow::builder()
        .css_classes(["dvoty-entry"])
        .child(&wrapper_box)
        .build();

    return res;
}

pub fn add_entry(
    entry: DvotyEntry,
    window: &Window,
    context: Rc<RefCell<AppContext>>,
    config: Arc<AppConf>,
) -> Result<DaemonRes, DaemonErr> {
    let context_ref = &mut context.borrow_mut();

    let list = if let Some(l) = &context_ref.dvoty.dvoty_list {
        l.clone()
    } else {
        if let Ok(res) = super::utils::get_list(window) {
            context_ref.dvoty.dvoty_list = Some(res.clone());
            res
        } else {
            println!("Dvoty: can't find list");
            return Err(DaemonErr::CannotFindWidget);
        }
    };

    match entry {
        DvotyEntry::Empty => {
            super::instruction::populate_instructions(&list, config, context_ref);
        }
        DvotyEntry::Math { result, .. } => {
            super::math::populate_math_entry(config, &list, result, context_ref);
        }
        DvotyEntry::Search { keyword } => {
            super::search::populate_search_entry(config, &list, keyword, context_ref);
        }
        DvotyEntry::Url { url } => {
            super::url::populate_url_entry(config, &list, url, context_ref);
        }
        _ => {}
    }

    Ok(DaemonRes::Success)
}
