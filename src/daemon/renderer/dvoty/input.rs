use std::{path::PathBuf, sync::Arc};

use gtk4::{prelude::EditableExt, Window};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::{
        renderer::config::AppConf,
        structs::{DaemonCmdType, DaemonEvt, Dvoty},
    },
    utils::{cache_dir, DaemonErr},
};

use super::{
    event::CURRENT_IDS, general::process_general, utils::get_input, DvotyContext, DvotyEntry,
    DvotyTaskType,
};

async fn process_input_str(
    input: &str,
    sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
    monitor: usize,
    recent_paths: Vec<PathBuf>,
    id: uuid::Uuid,
) {
    if input.is_empty() {
        if let Err(e) = sender.send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Empty)),
            sender: None,
            uuid: Some(id),
            monitors: vec![monitor],
        }) {
            println!("Dvoty: Failed to send entry: {}, ignoring...", e);
        };
        return;
    }

    match input.chars().next().unwrap() {
        '=' => {
            super::math::eval_math(
                input.chars().skip(1).collect::<String>().to_lowercase(),
                sender,
                &id,
                monitor,
            );
        }
        '@' => {
            super::app_launcher::process_apps(
                {
                    if input.len() == 1 {
                        ""
                    } else {
                        &input[1..]
                    }
                },
                sender,
                &id,
                config.clone(),
                monitor,
            );
        }
        '$' => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Command {
                        exec: input.chars().skip(1).collect::<String>(),
                    })),
                    sender: None,
                    uuid: Some(id),
                    monitors: vec![monitor],
                })
                .unwrap_or_else(|e| {
                    println!("Dvoty: Failed to send command: {}", e);
                });
        }
        ':' => {
            super::url::send_url(
                input.chars().skip(1).collect::<String>(),
                sender,
                &id,
                monitor,
            );
        }
        '/' => {
            super::search::handle_search(
                sender,
                input.chars().skip(1).collect::<String>(),
                &id,
                config,
                monitor,
            )
            .await;
        }
        '^' => {
            super::letter::process_greek_letters(
                input.chars().skip(1).collect::<String>(),
                sender,
                &id,
                monitor,
            );
        }
        '#' => {
            super::files::process_recent_files(
                input.chars().skip(1).collect::<String>(),
                sender,
                &id,
                monitor,
                recent_paths,
            );
        }
        '\\' => {
            process_general(
                sender,
                &input.chars().skip(1).collect::<String>(),
                &id,
                config,
                monitor,
                recent_paths,
            )
            .await;
        }
        _ => {
            process_general(sender, input, &id, config, monitor, recent_paths).await;
        }
    }
}

pub fn process_input(
    input: String,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    windows: &[Window],
    config: Arc<AppConf>,
    monitor: usize,
    recent_paths: Vec<PathBuf>,
) -> Result<(), DaemonErr> {
    let id = uuid::Uuid::new_v4();
    {
        *CURRENT_IDS.get().unwrap()[monitor].lock().unwrap() = id;
    }

    if context.should_autofill[monitor] {
        // find cache
        let mut cache_dir = cache_dir();
        cache_dir.push("histfile");

        if let Ok(content) = std::fs::read_to_string(cache_dir) {
            let histories: Vec<&str> = content.split("\n").filter(|s| !s.is_empty()).collect();
            for ele in histories.iter() {
                if ele.starts_with(&input) && !input.is_empty() && ele.len() > input.len() {
                    if let Ok(input_ui) = get_input(&windows[monitor]) {
                        input_ui.set_text(ele);
                        input_ui.select_region(input.len() as i32, -1);
                    }
                    //sender
                    //    .send(DaemonEvt {
                    //        evt: DaemonCmdType::Dvoty(Dvoty::Update(ele.to_string(), recent_paths)),
                    //        sender: None,
                    //        uuid: None,
                    //        monitors: vec![monitor],
                    //    })
                    //    .unwrap_or_else(|e| {
                    //        println!("Dvoty: Cannot resend input: {}", e);
                    //    });

                    return Ok(());
                }
            }
        }
    }

    context.dvoty_entries[monitor].clear();
    context.cur_ind[monitor] = 0;
    context.target_scroll[monitor] = 0.0f64;

    let list = if let Some(l) = &context.dvoty_list[monitor] {
        l
    } else if let Ok(res) = super::utils::get_list(&windows[monitor]) {
        context.dvoty_list[monitor] = Some(res);
        context.dvoty_list[monitor].as_ref().unwrap()
    } else {
        println!("Dvoty: can't find list");
        return Err(DaemonErr::CannotFindWidget);
    };

    list.remove_all();

    let task_map = &mut context.dvoty_tasks;

    if let Some(handle) = task_map[monitor].get(&DvotyTaskType::ProcessInput) {
        handle.abort();
        task_map[monitor].remove(&DvotyTaskType::ProcessInput);
    }

    let handle = tokio::spawn(async move {
        process_input_str(&input, sender.clone(), config, monitor, recent_paths, id).await;
    });

    task_map[monitor].insert(DvotyTaskType::ProcessInput, handle);

    Ok(())
}
