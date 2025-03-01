use std::{cell::RefMut, collections::HashSet, ffi::OsString, path::PathBuf, sync::Arc};

use anyhow::Context;
use freedesktop_file_parser::{EntryType, IconString};
use gtk4::ListBox;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::{
    daemon::{
        renderer::{app::AppContext, config::AppConf, dvoty::DvotyEntry},
        structs::{DaemonCmd, DaemonEvt, Dvoty},
    },
    utils::get_paths,
};

use super::{class::adjust_class, entry::DvotyUIEntry, event::CURRENT_ID};

use std::sync::Mutex;

use freedesktop_file_parser::DesktopFile;
use once_cell::sync::OnceCell;

pub static DESKTOP_FILES: OnceCell<Arc<Mutex<Vec<DesktopFile>>>> = OnceCell::new();

fn add_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: user overrides
    let content = std::fs::read_to_string(path)?;

    let desktop_file = freedesktop_file_parser::parse(&content)?;

    if let Some(bool) = desktop_file.entry.no_display {
        if bool {
            return Ok(());
        }
    }

    Ok(DESKTOP_FILES
        .get()
        .unwrap()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .push(desktop_file))
}

fn fill_files(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
        let _ = add_file(p);
    });

    Ok(())
}

pub fn process_paths() {
    DESKTOP_FILES
        .get()
        .unwrap()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clear();

    let paths = get_paths();

    paths.par_iter().for_each(|path| {
        let _ = fill_files(path);
    });
}

fn send(sender: UnboundedSender<DaemonEvt>, name: &str, exec: &str, icon: &IconString, id: &Uuid) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Launch {
                name: name.to_string(),
                exec: exec.to_string(),
                icon: icon.get_icon_path(),
            })),
            sender: None,
            uuid: Some(*id),
        })
        .unwrap_or_else(|e| println!("Dvoty: failed to send: {}", e));
}

fn process_content(
    content: &DesktopFile,
    input: &str,
    sender: UnboundedSender<DaemonEvt>,
    id: &Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    if *id != *CURRENT_ID.lock().unwrap_or_else(|p| p.into_inner()) {
        return Ok(());
    }

    // TODO: handle not show in
    // TODO: add user overrides

    if let Some(ref icon) = content.entry.icon {
        if let EntryType::Application(ref fields) = content.entry.entry_type {
            if let Some(ref exec) = fields.exec {
                let mut keywords: Vec<String> = vec![content.entry.name.default.to_lowercase()];
                if let Some(ref generic_name) = content.entry.generic_name {
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
                            &content.entry.name.default,
                            &exec,
                            &icon,
                            id,
                        );

                        for (_, value) in content.actions.clone() {
                            send(
                                sender.clone(),
                                &format!("{}: {}", content.entry.name.default, value.name.default),
                                &exec,
                                &icon,
                                id,
                            );
                        }

                        return Ok(());
                    }
                }

                for (_, value) in content.actions.clone() {
                    if value.name.default.to_lowercase().contains(input) {
                        send(
                            sender.clone(),
                            &format!("{}: {}", content.entry.name.default, value.name.default),
                            &exec,
                            &icon,
                            id,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn process_apps(input: &str, sender: UnboundedSender<DaemonEvt>, id: &Uuid) {
    DESKTOP_FILES
        .get()
        .unwrap()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .par_iter()
        .for_each(|file| {
            let _ = process_content(file, input, sender.clone(), id);
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
        "Click to launch",
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
