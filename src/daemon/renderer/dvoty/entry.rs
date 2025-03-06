use std::{cell::RefCell, path::PathBuf, process::Stdio, rc::Rc, sync::Arc};

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
        terminal: bool,
        name: String,
        exec: String,
        icon: Option<PathBuf>,
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
    Letter {
        letter: String,
    },
}

#[derive(Clone)]
pub enum DvotyUIEntry {
    Instruction,
    Math { result: String },
    Launch { terminal: bool, exec: String },
    Command { exec: String },
    Search { keyword: String },
    Url { url: String },
    Letter { letter: String },
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
                    .arg(format!("{} {}", config.dvoty.terminal_exec, &exec))
                    .stdout(Stdio::null())
                    .spawn()
                {
                    println!("Dvoty: Failed to spawn command: {}", e);
                }
            }
            DvotyUIEntry::Launch { terminal, exec } => {
                let exec = if !terminal {
                    exec
                } else {
                    format!("{} {}", config.dvoty.terminal_exec, exec)
                };

                if let Err(e) = std::process::Command::new("setsid")
                    .arg("/bin/sh")
                    .arg("-c")
                    .arg(&exec)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    println!("Dvoty: Failed to spawn command: {}", e);
                }
            }
            DvotyUIEntry::Letter { letter } => {
                math::set_clipboard_text(&letter);
            }

            DvotyUIEntry::Instruction => {}
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
        .css_classes(["dvoty-label", "dvoty-label-mid"])
        .halign(gtk4::Align::Start)
        .hexpand(false)
        .build();

    let label_end = Label::builder()
        .use_markup(true)
        .label(tip)
        .css_classes(["dvoty-label", "dvoty-label-end"])
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
                uuid: None,
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
        DvotyEntry::Launch {
            terminal,
            name,
            exec,
            icon,
        } => {
            super::app_launcher::populate_launcher_entry(
                config,
                &list,
                name,
                terminal,
                exec,
                icon,
                context_ref,
                sender,
            );
        }
        DvotyEntry::Letter { letter } => {
            super::letter::populate_letter_entry(config, &list, letter, context_ref, sender);
        }
        _ => {}
    }

    Ok(DaemonRes::Success)
}
