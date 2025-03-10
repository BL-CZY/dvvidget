use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use crate::{
    daemon::{
        renderer::config::AppConf,
        structs::{DaemonCmdType, DaemonEvt, DaemonRes, Dvoty},
    },
    utils::DaemonErr,
};

use super::{math, search, url, DvotyContext};
use gtk4::{
    prelude::{BoxExt, WidgetExt},
    Box, GestureClick, Image, Label, ListBoxRow, ScrolledWindow, Window,
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
        title: Option<String>,
    },
    Letter {
        letter: String,
    },
    File {
        path: PathBuf,
        name: String,
        icon: Option<PathBuf>,
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
    File { path: PathBuf },
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
            DvotyUIEntry::File { path } => {
                tokio::spawn(async move {
                    open::that(path).unwrap_or_else(|e| {
                        println!("Dvoty: Cannot open file: {}", e);
                    });
                });
            }

            DvotyUIEntry::Instruction => {}
        }
    }
}

pub fn create_base_entry<P>(
    icon_path: P,
    content: &str,
    tip: &str,
    sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
    monitor: usize,
) -> ListBoxRow
where
    P: AsRef<Path>,
{
    let icon = Image::from_file(icon_path);
    icon.add_css_class("dvoty-icon");
    icon.set_halign(gtk4::Align::Start);

    let label_begin = Label::builder()
        .use_markup(true)
        .label(content.replace("&", "&amp;"))
        .css_classes(["dvoty-label", "dvoty-label-mid"])
        .halign(gtk4::Align::Start)
        .hexpand(false)
        .build();

    let mid_wrapper = ScrolledWindow::builder()
        .css_classes(["dvoty-mid-scroll"])
        .min_content_width(config.dvoty.max_mid_width)
        .child(&label_begin)
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
    wrapper_box.append(&mid_wrapper);
    wrapper_box.append(&label_end);

    let res = ListBoxRow::builder()
        .css_classes(["dvoty-entry"])
        .child(&wrapper_box)
        .build();

    let gesture_click = GestureClick::new();
    gesture_click.connect_pressed(move |_, _, _, _| {
        sender
            .send(DaemonEvt {
                evt: DaemonCmdType::Dvoty(Dvoty::TriggerEntry),
                sender: None,
                uuid: None,
                monitors: vec![monitor],
            })
            .unwrap_or_else(|e| println!("Dvoty: Failed to send trigger event by clicking: {}", e))
    });

    res.add_controller(gesture_click);

    res
}

pub fn add_entry(
    entry: DvotyEntry,
    windows: &[Window],
    context: &mut DvotyContext,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) -> Result<DaemonRes, DaemonErr> {
    let list = if let Some(l) = &context.dvoty_list[monitor] {
        l.clone()
    } else if let Ok(res) = super::utils::get_list(&windows[monitor]) {
        context.dvoty_list[monitor] = Some(res.clone());
        res
    } else {
        println!("Dvoty: can't find list");
        return Err(DaemonErr::CannotFindWidget);
    };

    match entry {
        DvotyEntry::Empty => {
            super::instruction::populate_instructions(&list, config, context, sender, monitor);
        }
        DvotyEntry::Math { result, .. } => {
            super::math::populate_math_entry(config, &list, result, context, sender, monitor);
        }
        DvotyEntry::Search { keyword } => {
            super::search::populate_search_entry(config, &list, keyword, context, sender, monitor);
        }
        DvotyEntry::Url { url, title } => match title {
            Some(s) => super::url::populate_url_entry(
                config,
                &list,
                &format!("{} <i><span foreground=\"grey\">{}</span></i>", &s, &url),
                url,
                context,
                sender,
                monitor,
            ),
            None => super::url::populate_url_entry(
                config,
                &list,
                &url.clone(),
                url,
                context,
                sender,
                monitor,
            ),
        },
        DvotyEntry::Command { exec } => {
            super::cmd::populate_cmd_entry(config, &list, exec, context, sender, monitor);
        }
        DvotyEntry::Launch {
            terminal,
            name,
            exec,
            icon,
        } => {
            super::app_launcher::populate_launcher_entry(
                config, &list, name, terminal, exec, icon, context, sender, monitor,
            );
        }
        DvotyEntry::Letter { letter } => {
            super::letter::populate_letter_entry(config, &list, letter, context, sender, monitor);
        }

        DvotyEntry::File { path, name, icon } => {
            super::files::populate_search_entry(
                config, &list, path, name, icon, context, sender, monitor,
            );
        }

        _ => {}
    }

    Ok(DaemonRes::Success)
}
