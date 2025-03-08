use std::cell::RefMut;
use std::path::PathBuf;
use std::sync::Arc;

use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::daemon::renderer::app::AppContext;
use crate::daemon::renderer::config::AppConf;
use crate::daemon::renderer::config::SearchEngine;
use crate::daemon::renderer::dvoty::event::CURRENT_ID;
use crate::daemon::structs::DaemonCmd;
use crate::daemon::structs::DaemonEvt;
use crate::daemon::structs::Dvoty;
use crate::utils;

use super::class::adjust_class;
use super::entry::create_base_entry;
use super::entry::DvotyUIEntry;
use super::DvotyEntry;

fn process_history(
    keyword: &str,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
    id: &Uuid,
) {
    let mut path = PathBuf::from(&config.dvoty.firefox_path);
    path.push("profiles.ini");
    let config = match ini::Ini::load_from_file(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("Cannot get firefox profile: {}", e);
            return;
        }
    };

    let sections = config.sections();

    let mut found = false;
    let mut name = "";
    for section in sections {
        if let Some(str) = section {
            if str.starts_with("Install") {
                name = str;
                found = true;
            }
        }
    }

    if !found {
        println!("Cannot locate firefox profile folder");
        return;
    }

    if let Some(section) = config.section(Some(name)) {
        if let Some(val) = section.get("Default") {
            path.pop();
            path.push(val);
            path.push("places.sqlite");

            let mut copy_path = utils::cache_dir();
            copy_path.push("places.sqlite");
            if let Err(e) = std::fs::copy(&path, &copy_path) {
                println!("Dvoty: cannot copy file: {}", e);
                return;
            }

            let conn = match rusqlite::Connection::open(&copy_path) {
                Ok(c) => c,
                Err(e) => {
                    println!("Cannot open connection: {}", e);
                    return;
                }
            };

            #[derive(Debug)]
            struct History {
                url: String,
                title: String,
            }

            let command = format!(
                "SELECT url, title FROM moz_places
WHERE 
  (LOWER(title) LIKE LOWER('%{}%')
  OR LOWER(url) LIKE LOWER('%{}%'))
  AND last_visit_date >= strftime('%s', 'now', '-30 days') * 1000000
ORDER BY id DESC
LIMIT 30;",
                keyword, keyword
            );

            let mut stmt = match conn.prepare(&command) {
                Ok(r) => r,
                Err(e) => {
                    println!("Dvoty: cannot query history: {}", e);
                    return;
                }
            };

            let result = match stmt.query_map([], |r| {
                Ok(History {
                    url: r.get(0)?,
                    title: r.get(1)?,
                })
            }) {
                Ok(r) => r,
                Err(e) => {
                    println!("Dvoty: cannot parse query result: {}", e);
                    return;
                }
            };

            for row in result {
                if *id != *CURRENT_ID.lock().unwrap_or_else(|p| p.into_inner()) {
                    break;
                }

                if let Ok(his) = row {
                    sender
                        .send(DaemonEvt {
                            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Url {
                                url: his.url.trim().to_string(),
                                title: Some(his.title.trim().to_string()),
                            })),
                            sender: None,
                            uuid: Some(*id),
                        })
                        .unwrap_or_else(|e| {
                            println!("Dvoty: Failed to send url: {}", e);
                        });
                }
            }
        }
    }
}

pub fn handle_search(
    sender: UnboundedSender<DaemonEvt>,
    keyword: String,
    id: &Uuid,
    config: Arc<AppConf>,
) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Search {
                keyword: keyword.clone(),
            })),
            sender: None,
            uuid: Some(*id),
        })
        .unwrap_or_else(|e| {
            println!("Dvoty: Error adding search entry: {}", e);
        });

    process_history(&keyword, config, sender.clone(), id);
}

pub fn spawn_keyword(keyword: String, config: Arc<AppConf>) {
    let search_url = match &config.dvoty.search_engine {
        SearchEngine::Google => format!("https://www.google.com/search?q={}", keyword),
        SearchEngine::Duckduckgo => format!("https://duckduckgo.com/?q={}", keyword),
        SearchEngine::Bing => format!("https://www.bing.com/search?q={}", keyword),
        SearchEngine::Wikipedia(lang) => format!("https://{}.wikipedia.org/wiki/{}", lang, keyword),
    };

    tokio::spawn(async move {
        open::that(search_url).unwrap_or_else(|e| println!("Dvoty: Can't perform search: {}", e));
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
        &config.dvoty.search_icon,
        &keyword,
        "Click to search",
        sender,
    );

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Search { keyword }, row.clone()));

    if context.dvoty.dvoty_entries.len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty.dvoty_entries);
    }

    list.append(&row);
}
