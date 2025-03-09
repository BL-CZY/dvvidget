use std::sync::Arc;

use gtk4::Window;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::{
        renderer::config::AppConf,
        structs::{DaemonCmdType, DaemonEvt, Dvoty},
    },
    utils::DaemonErr,
};

use super::{event::CURRENT_ID, general::process_general, DvotyContext, DvotyEntry, DvotyTaskType};

async fn process_input_str(
    input: &str,
    sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
    monitor: usize,
) {
    let id = *CURRENT_ID.lock().unwrap_or_else(|p| p.into_inner());

    if input.is_empty() {
        if let Err(e) = sender.send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::Empty)),
            sender: None,
            uuid: None,
            monitors: vec![monitor],
        }) {
            println!("Dvoty: Failed to send entry: {}, ignoring...", e);
        };
        return;
    }

    match input.chars().next().unwrap() {
        '=' => {
            super::math::eval_math(input, sender, &id, monitor);
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
        _ => {
            process_general(sender, input, &id, config, monitor).await;
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
) -> Result<(), DaemonErr> {
    let id = uuid::Uuid::new_v4();
    {
        *CURRENT_ID.lock().unwrap() = id;
    }

    context.dvoty_entries[monitor].clear();
    context.cur_ind[monitor] = 0;
    context.target_scroll[monitor] = 0.0f64;

    let task_map = &mut context.dvoty_tasks;

    if let Some(handle) = task_map[monitor].get(&DvotyTaskType::ProcessInput) {
        handle.abort();
        task_map[monitor].remove(&DvotyTaskType::ProcessInput);
    }

    let handle = tokio::spawn(async move {
        process_input_str(&input, sender.clone(), config, monitor).await;
    });

    task_map[monitor].insert(DvotyTaskType::ProcessInput, handle);

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

    Ok(())
}
