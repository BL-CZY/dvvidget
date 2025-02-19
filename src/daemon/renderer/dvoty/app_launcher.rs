use std::{cell::RefMut, collections::HashSet, ffi::OsString, path::PathBuf, sync::Arc};

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
                let mut keywords: Vec<String> =
                    vec![desktop_file.entry.name.default.to_lowercase()];
                if let Some(ref generic_name) = desktop_file.entry.generic_name {
                    keywords.push(generic_name.default.to_lowercase());
                }

                if let Some(ref kwds) = fields.keywords {
                    let temp: Vec<String> = kwds.default.iter().map(|s| s.to_lowercase()).collect();
                    keywords.extend(temp);
                }

                for kwd in keywords.iter() {
                    if kwd.contains(input) {
                        send(
                            sender.clone(),
                            &desktop_file.entry.name.default,
                            &exec,
                            &icon,
                        );

                        for (_, value) in desktop_file.actions {
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

                        return Ok(());
                    }
                }

                for (_, value) in desktop_file.actions {
                    if value.name.default.to_lowercase().contains(input) {
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

    let mut exist: HashSet<OsString> = HashSet::new();

    let paths = dirs
        .filter_map(|entry| match entry {
            Ok(res) => {
                if !exist.contains(&res.file_name()) {
                    exist.insert(res.file_name());
                    Some(res.path())
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect::<Vec<PathBuf>>();

    paths.par_iter().for_each(|p| {
        let _ = process_content(p, input, sender.clone());
    });

    Ok(())
}

pub fn process_apps(input: &str, sender: UnboundedSender<DaemonEvt>) {
    let input = &input.to_lowercase();
    let paths = if let Ok(v) = std::env::var("XDG_DATA_DIRS") {
        v.split(":")
            .filter_map(|s| {
                let mut res = if let Ok(p) = PathBuf::try_from(s) {
                    p
                } else {
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

fn clense_cmd(exec: &mut String, str: &str) {
    if exec.ends_with(str) {
        exec.pop();
        exec.pop();
    }
}

pub fn populate_launcher_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    name: String,
    mut exec: String,
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

    clense_cmd(&mut exec, "%u");
    clense_cmd(&mut exec, "%U");
    clense_cmd(&mut exec, "%f");
    clense_cmd(&mut exec, "%F");
    clense_cmd(&mut exec, "%i");
    clense_cmd(&mut exec, "%c");
    clense_cmd(&mut exec, "%k");

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Launch { exec }, row.clone()));

    if context.dvoty.dvoty_entries.len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty.dvoty_entries);
    }

    list.append(&row);
}
