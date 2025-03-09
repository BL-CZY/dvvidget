use std::{collections::HashMap, path::PathBuf, sync::Arc};

use freedesktop_file_parser::{EntryType, IconString};
use gtk4::ListBox;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::{
    daemon::{
        renderer::{config::AppConf, dvoty::DvotyEntry},
        structs::{DaemonCmdType, DaemonEvt, Dvoty},
    },
    utils::get_paths,
};

use super::{class::adjust_class, entry::DvotyUIEntry, event::CURRENT_IDS, DvotyContext};

use std::sync::Mutex;

use freedesktop_file_parser::DesktopFile;
use once_cell::sync::OnceCell;

pub static DESKTOP_FILES: OnceCell<Arc<Mutex<Vec<DesktopFile>>>> = OnceCell::new();

fn add_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    let desktop_file = match freedesktop_file_parser::parse(&content) {
        Err(e) => {
            return Err(Box::new(e));
        }
        Ok(r) => r,
    };

    if let Some(bool) = desktop_file.entry.no_display {
        if bool {
            return Ok(());
        }
    }

    DESKTOP_FILES
        .get()
        .unwrap()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .push(desktop_file);

    Ok(())
}

pub async fn process_paths() -> Result<(), Box<dyn std::error::Error>> {
    DESKTOP_FILES
        .get()
        .unwrap()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clear();

    let paths = get_paths();

    let mut desktop_files_map: HashMap<String, PathBuf> = HashMap::new();

    // Process directories in order (later directories will override earlier ones)
    for dir in paths {
        match tokio::fs::metadata(&dir).await {
            Ok(metadata) if metadata.is_dir() => {}
            _ => continue, // Skip if not a directory or can't access
        }

        // otherwise it will stuck on the first one
        let mut iter = tokio::fs::read_dir(&dir).await?;

        // Read all entries in the directory
        while let Some(entry) = iter.next_entry().await? {
            let path = entry.path();

            // Check if it's a .desktop file
            if path.is_file() && path.extension().map_or(false, |ext| ext == "desktop") {
                // Get the filename as the key
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    // Add or replace in the map
                    desktop_files_map.insert(filename.to_string(), path);
                }
            }
        }
    }

    // Convert the map values to a vector
    let paths: Vec<PathBuf> = desktop_files_map.into_values().collect();

    paths.par_iter().for_each(|path| {
        let _ = add_file(path);
    });

    DESKTOP_FILES
        .get()
        .unwrap()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .par_sort_by(|a, b| {
            a.entry
                .name
                .default
                .to_lowercase()
                .cmp(&b.entry.name.default.to_lowercase())
        });

    Ok(())
}

fn send(
    sender: UnboundedSender<DaemonEvt>,
    name: String,
    exec: String,
    terminal: bool,
    icon: IconString,
    id: Uuid,
    monitor: usize,
) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Launch {
                terminal,
                name: name.to_string(),
                exec: exec.to_string(),
                icon: icon.get_icon_path(),
            })),
            sender: None,
            uuid: Some(id),
            monitors: vec![monitor],
        })
        .unwrap_or_else(|e| println!("Dvoty: failed to send: {}", e));
}

pub fn underline_string(input: &str, str: &str) -> String {
    if input.len() == 0 {
        return str.to_string();
    }

    let str_lower = str.to_lowercase();
    if let Some(i) = str_lower.find(input) {
        let mut result = "".to_string();
        result.push_str(&str[0..i]);
        result.push_str("<u><b>");
        result.push_str(&str[i..i + input.len()]);
        result.push_str("</b></u>");
        result.push_str(&str[i + input.len()..]);
        return result;
    }

    str.to_string()
}

fn process_content(
    content: &DesktopFile,
    input: &str,
    sender: UnboundedSender<DaemonEvt>,
    id: &Uuid,
    config: Arc<AppConf>,
    monitor: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = &input.to_lowercase();
    if *id
        != *CURRENT_IDS.get().unwrap()[monitor]
            .lock()
            .unwrap_or_else(|p| p.into_inner())
    {
        return Ok(());
    }

    // TODO: handle not show in
    // TODO: add user overrides

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

            let default_icon = IconString {
                content: config.dvoty.launch_icon.clone(),
            };
            let icon = if let Some(ref icon) = content.entry.icon {
                icon
            } else {
                &default_icon
            };

            let terminal = fields.terminal.map_or_else(|| false, |v| v);

            for kwd in keywords.iter() {
                if kwd.contains(input) {
                    send(
                        sender.clone(),
                        underline_string(input, &content.entry.name.default),
                        exec.clone(),
                        terminal,
                        icon.clone(),
                        *id,
                        monitor,
                    );

                    for (_, value) in content.actions.clone() {
                        send(
                            sender.clone(),
                            format!(
                                "{}: {}",
                                underline_string(input, &content.entry.name.default),
                                value.name.default
                            ),
                            value.exec.clone().map_or(exec.clone(), |v| v),
                            terminal,
                            icon.clone(),
                            *id,
                            monitor,
                        );
                    }

                    return Ok(());
                }
            }

            for (_, value) in content.actions.clone() {
                if value.name.default.to_lowercase().contains(input) {
                    send(
                        sender.clone(),
                        format!(
                            "{}: {}",
                            content.entry.name.default,
                            underline_string(input, &value.name.default),
                        ),
                        value.exec.clone().map_or(exec.clone(), |v| v),
                        terminal,
                        icon.clone(),
                        *id,
                        monitor,
                    );
                }
            }
        }
    }

    Ok(())
}

pub fn process_apps(
    input: &str,
    sender: UnboundedSender<DaemonEvt>,
    id: &Uuid,
    config: Arc<AppConf>,
    monitor: usize,
) {
    DESKTOP_FILES
        .get()
        .unwrap()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .iter()
        .for_each(|file| {
            let _ = process_content(file, input, sender.clone(), id, config.clone(), monitor);
        });
}

fn clense_cmd(exec: &mut String, str: &str) {
    if exec.ends_with(str) {
        exec.pop();
        exec.pop();
    }
}

// TODO: add terminal apps
pub fn populate_launcher_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    name: String,
    terminal: bool,
    mut exec: String,
    icon: Option<PathBuf>,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
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
        config.clone(),
        monitor,
    );

    clense_cmd(&mut exec, "%u");
    clense_cmd(&mut exec, "%U");
    clense_cmd(&mut exec, "%f");
    clense_cmd(&mut exec, "%F");
    clense_cmd(&mut exec, "%i");
    clense_cmd(&mut exec, "%c");
    clense_cmd(&mut exec, "%k");

    context.dvoty_entries[monitor].push((DvotyUIEntry::Launch { terminal, exec }, row.clone()));

    if context.dvoty_entries[monitor].len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty_entries[monitor]);
    }

    list.append(&row);
}
