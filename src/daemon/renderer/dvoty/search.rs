use std::cell::RefMut;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::daemon::renderer::app::AppContext;
use crate::daemon::renderer::config::AppConf;
use crate::daemon::renderer::config::SearchEngine;
use crate::daemon::renderer::dvoty::app_launcher::underline_string;
use crate::daemon::renderer::dvoty::event::CURRENT_ID;
use crate::daemon::structs::DaemonCmdType;
use crate::daemon::structs::DaemonEvt;
use crate::daemon::structs::Dvoty;

use super::class::adjust_class;
use super::entry::create_base_entry;
use super::entry::DvotyUIEntry;
use super::DvotyContext;
use super::DvotyEntry;

pub async fn process_history(
    keyword: &str,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
    id: &Uuid,
    monitor: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let keyword = keyword.to_lowercase();

    let mut path = PathBuf::from(&config.dvoty.firefox_path);
    path.push("profiles.ini");
    let firefox_config =
        ini::Ini::load_from_file(&path).with_context(|| "Dvoty: Cannot read firefox config")?;

    let sections = firefox_config.sections();

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
        return Err(anyhow!("Cannot locate firefox profile folder").into());
    }

    if let Some(section) = firefox_config.section(Some(name)) {
        if let Some(val) = section.get("Default") {
            path.pop();
            path.push(val);
            path.push("places.sqlite");

            // in case it's locked regardless in the future
            //let mut copy_path = utils::cache_dir();
            //copy_path.push("places.sqlite");
            //if let Err(e) = std::fs::copy(&path, &copy_path) {
            //    println!("Dvoty: cannot copy file: {}", e);
            //    return;
            //}

            let pool = sqlx::SqlitePool::connect(&format!(
                "sqlite:{}?immutable=1",
                &path.to_str().unwrap_or_else(|| { "" })
            ))
            .await
            .with_context(|| "Cannot open connection")?;

            #[derive(Debug, sqlx::FromRow)]
            struct BookmarkPlace {
                folder_name: String,
                bookmark_title: String,
                url: String,
            }

            // for bookmarks
            let command = format!(
                "
SELECT
    b.title AS bookmark_title,
    p.url AS url,
    folder.title AS folder_name
FROM
    moz_bookmarks AS b
JOIN
    moz_places AS p ON b.fk = p.id
LEFT JOIN
    moz_bookmarks AS folder ON b.parent = folder.id
WHERE
    b.type = 1
AND
    (LOWER(bookmark_title) LIKE LOWER('%{}%')
    OR LOWER(url) LIKE LOWER('%{}%')
	OR LOWER(folder_name) LIKE LOWER('%{}%'))
LIMIT {};
",
                keyword, keyword, keyword, config.dvoty.bookmark_search_limit
            );

            // Build the query using query_as to map results to your struct
            let places = sqlx::query_as::<_, BookmarkPlace>(&command)
                .fetch_all(&pool)
                .await?;

            for place in places {
                if *id != *CURRENT_ID.lock().unwrap_or_else(|p| p.into_inner()) {
                    return Ok(());
                }

                sender
                    .send(DaemonEvt {
                        evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Url {
                            url: place.url.clone(),
                            title: Some(format!(
                                "<span color=\"{}\"> <u><b>{}:</b></u></span> {}",
                                config.dvoty.highlight_color,
                                underline_string(&keyword, &place.folder_name),
                                underline_string(&keyword, &place.bookmark_title)
                            )),
                        })),
                        sender: None,
                        uuid: Some(*id),
                        monitor,
                    })
                    .unwrap_or_else(|e| {
                        println!("Dvoty: Failed to send url: {}", e);
                    });

                tokio::task::yield_now().await;
            }

            #[derive(Debug, sqlx::FromRow)]
            struct Place {
                url: String,
                title: String,
            }

            // for history
            let command = format!(
                "SELECT url, title FROM moz_places
WHERE 
  (LOWER(title) LIKE LOWER('%{}%')
  OR LOWER(url) LIKE LOWER('%{}%'))
  AND last_visit_date >= strftime('%s', 'now', '-{} days') * 1000000
ORDER BY id DESC
LIMIT {};",
                keyword,
                keyword,
                config.dvoty.past_search_date_limit,
                config.dvoty.past_search_limit
            );

            // Build the query using query_as to map results to your struct
            let places = sqlx::query_as::<_, Place>(&command)
                .fetch_all(&pool)
                .await?;

            places.iter().for_each(|place| {
                if *id != *CURRENT_ID.lock().unwrap_or_else(|p| p.into_inner()) {
                    return;
                }

                sender
                    .send(DaemonEvt {
                        evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Url {
                            url: place.url.clone(),
                            title: Some(underline_string(&keyword, &place.title)),
                        })),
                        sender: None,
                        uuid: Some(*id),
                        monitor,
                    })
                    .unwrap_or_else(|e| {
                        println!("Dvoty: Failed to send url: {}", e);
                    });
            });
        }
    }

    Ok(())
}

pub async fn handle_search(
    sender: UnboundedSender<DaemonEvt>,
    keyword: String,
    id: &Uuid,
    config: Arc<AppConf>,
    monitor: usize,
) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Search {
                keyword: keyword.clone(),
            })),
            sender: None,
            uuid: Some(*id),
            monitor,
        })
        .unwrap_or_else(|e| {
            println!("Dvoty: Error adding search entry: {}", e);
        });

    process_history(&keyword, config, sender.clone(), &id, monitor)
        .await
        .unwrap_or_else(|e| {
            println!("{}", e);
        });
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
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) {
    let row = create_base_entry(
        &config.dvoty.search_icon,
        &keyword,
        "Click to search",
        sender,
        config.clone(),
        monitor,
    );

    context.dvoty_entries[monitor].push((DvotyUIEntry::Search { keyword }, row.clone()));

    if context.dvoty_entries[monitor].len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty_entries[monitor]);
    }

    list.append(&row);
}
