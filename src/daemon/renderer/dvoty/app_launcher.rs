use std::{cell::RefMut, path::PathBuf, sync::Arc};

use anyhow::Context;
use freedesktop_file_parser::{EntryType, IconString};
use gtk4::ListBox;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{
    renderer::{app::AppContext, config::AppConf, dvoty::DvotyEntry},
    structs::{DaemonCmd, DaemonEvt, Dvoty},
};

use super::{class::adjust_class, entry::DvotyUIEntry};

fn send(sender: UnboundedSender<DaemonEvt>, name: &str, exec: &str, icon: &IconString) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Launch {
                name: name.to_string(),
                exec: exec.to_string(),
                icon: icon.get_icon_path(),
            })),
            sender: None,
        })
        .unwrap_or_else(|e| println!("Dvoty: failed to send: {}", e));
}

fn process_content(
    path: &PathBuf,
    input: &str,
    sender: UnboundedSender<DaemonEvt>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    let desktop_file = freedesktop_file_parser::parse(&content)?;

    if let Some(bool) = desktop_file.entry.no_display {
        if bool {
            return Ok(());
        }
    }

    // TODO: handle not show in

    if let Some(icon) = desktop_file.entry.icon {
        if let EntryType::Application(fields) = desktop_file.entry.entry_type {
            if let Some(exec) = fields.exec {
                let mut keywords: Vec<&str> = vec![&desktop_file.entry.name.default];
                if let Some(ref generic_name) = desktop_file.entry.generic_name {
                    keywords.push(&generic_name.default);
                }

                if let Some(ref kwds) = fields.keywords {
                    let temp: Vec<&str> = kwds.default.iter().map(AsRef::as_ref).collect();
                    keywords.extend(temp);
                }

                println!("{:?}", keywords);

                for kwd in keywords.iter() {
                    if kwd.contains(input) {
                        send(
                            sender.clone(),
                            &desktop_file.entry.name.default,
                            &exec,
                            &icon,
                        );
                    }
                }

                for (_, value) in desktop_file.actions {
                    if value.name.default.contains(input) {
                        send(
                            sender.clone(),
                            &format!(
                                "{}: {}",
                                desktop_file.entry.name.default, value.name.default
                            ),
                            &exec,
                            &icon,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn process_path(
    path: &PathBuf,
    input: &str,
    sender: UnboundedSender<DaemonEvt>,
) -> Result<(), Box<dyn std::error::Error>> {
    let dirs = std::fs::read_dir(path).context("Can't read directory")?;

    let paths = dirs
        .filter_map(|entry| match entry {
            Ok(res) => Some(res.path()),
            Err(_) => None,
        })
        .collect::<Vec<PathBuf>>();

    paths.par_iter().for_each(|p| {
        let _ = process_content(p, input, sender.clone());
    });

    Ok(())
}

pub fn process_apps(input: &str, sender: UnboundedSender<DaemonEvt>) {
    let paths = if let Ok(v) = std::env::var("XDG_DATA_DIRS") {
        v.split(":")
            .filter_map(|s| {
                let mut res = if let Ok(p) = PathBuf::try_from(s) {
                    p
                } else {
                    #[cfg(debug_assertions)]
                    println!("{:?} is not valid path", s);

                    return None;
                };

                res.push("applications/");
                Some(res)
            })
            .collect::<Vec<PathBuf>>()
    } else {
        println!("Dvoty: cannot read XDG_DATA_DIR");
        return;
    };

    paths.par_iter().for_each(|path| {
        let _ = process_path(path, input, sender.clone());
    });
}

pub fn populate_launcher_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    name: String,
    exec: String,
    icon: Option<PathBuf>,
    context: &mut RefMut<AppContext>,
    sender: UnboundedSender<DaemonEvt>,
) {
    let row = super::entry::create_base_entry(
        match icon {
            Some(ref buf) => {
                if let Some(str) = buf.to_str() {
                    str
                } else {
                    &config.dvoty.instruction_icon
                }
            }
            None => &config.dvoty.instruction_icon,
        },
        &name,
        "Click to launch".into(),
        sender,
    );

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Launch { exec }, row.clone()));

    list.append(&row);
}
