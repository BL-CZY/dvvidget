use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    daemon::{
        renderer::{app::AppContext, config::AppConf},
        structs::{DaemonCmd, DaemonEvt, DaemonRes, Dvoty},
    },
    utils::DaemonErr,
};

use super::{math, search, url};
use gtk4::{
    prelude::{BoxExt, WidgetExt},
    Box, GestureClick, Image, Label, ListBoxRow, Window,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

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
    pub fn run(self, config: Arc<AppConf>) {
        match self {
            DvotyUIEntry::Math { result } => {
                math::set_clipboard_text(&result);
            }
            DvotyUIEntry::Search { keyword } => {
                search::spawn_keyword(keyword, config);
            }
            DvotyUIEntry::Url { url } => {
                url::spawn_url(url);
            }
            DvotyUIEntry::Command { exec } => {
                if let Err(e) = std::process::Command::new("setsid")
                    .arg("/bin/sh")
                    .arg("-c")
                    .arg(&exec)
                    .spawn()
                {
                    println!("Dvoty: Failed to spawn command: {}", e);
                }
            }
            _ => {}
        }
    }
}

pub fn create_base_entry(
    icon_path: &str,
    content: &str,
    tip: &str,
    sender: UnboundedSender<DaemonEvt>,
) -> ListBoxRow {
    let icon = Image::from_file(icon_path);
    icon.add_css_class("dvoty-icon");
    icon.set_halign(gtk4::Align::Start);

    let label_begin = Label::builder()
        .use_markup(true)
        .label(content)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::Start)
        .hexpand(false)
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

    wrapper_box.append(&icon);
    wrapper_box.append(&label_begin);
    wrapper_box.append(&label_end);

    let res = ListBoxRow::builder()
        .css_classes(["dvoty-entry"])
        .child(&wrapper_box)
        .build();

    let gesture_click = GestureClick::new();
    gesture_click.connect_pressed(move |_, _, _, _| {
        sender
            .send(DaemonEvt {
                evt: DaemonCmd::Dvoty(Dvoty::TriggerEntry),
                sender: None,
            })
            .unwrap_or_else(|e| println!("Dvoty: Failed to send trigger event by clicking: {}", e))
    });

    res.add_controller(gesture_click);

    res
}

pub fn add_entry(
    entry: DvotyEntry,
    window: &Window,
    context: Rc<RefCell<AppContext>>,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
) -> Result<DaemonRes, DaemonErr> {
    let context_ref = &mut context.borrow_mut();

    let list = if let Some(l) = &context_ref.dvoty.dvoty_list {
        l.clone()
    } else if let Ok(res) = super::utils::get_list(window) {
        context_ref.dvoty.dvoty_list = Some(res.clone());
        res
    } else {
        println!("Dvoty: can't find list");
        return Err(DaemonErr::CannotFindWidget);
    };

    match entry {
        DvotyEntry::Empty => {
            super::instruction::populate_instructions(&list, config, context_ref, sender);
        }
        DvotyEntry::Math { result, .. } => {
            super::math::populate_math_entry(config, &list, result, context_ref, sender);
        }
        DvotyEntry::Search { keyword } => {
            super::search::populate_search_entry(config, &list, keyword, context_ref, sender);
        }
        DvotyEntry::Url { url } => {
            super::url::populate_url_entry(config, &list, url, context_ref, sender);
        }
        DvotyEntry::Command { exec } => {
            super::cmd::populate_cmd_entry(config, &list, exec, context_ref, sender);
        }
        _ => {}
    }

    Ok(DaemonRes::Success)
}
