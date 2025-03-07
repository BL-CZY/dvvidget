use std::{cell::RefCell, rc::Rc, sync::Arc};

use gtk4::Window;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::{
        renderer::{app::AppContext, config::AppConf},
        structs::{DaemonCmd, DaemonEvt, Dvoty},
    },
    utils::DaemonErr,
};

use super::{event::CURRENT_ID, general::process_general, DvotyEntry, DvotyTaskType};

async fn process_input_str(input: &str, sender: UnboundedSender<DaemonEvt>, config: Arc<AppConf>) {
    let id = *CURRENT_ID.lock().unwrap_or_else(|p| p.into_inner());

    if input.is_empty() {
        if let Err(e) = sender.send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Empty)),
            sender: None,
            uuid: None,
        }) {
            println!("Dvoty: Failed to send entry: {}, ignoring...", e);
        };
        return;
    }

    match input.chars().next().unwrap() {
        '=' => {
            super::math::eval_math(input, sender, &id);
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
            );
        }
        '$' => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Command {
                        exec: input.chars().skip(1).collect::<String>(),
                    })),
                    sender: None,
                    uuid: Some(id),
                })
                .unwrap_or_else(|e| {
                    println!("Dvoty: Failed to send command: {}", e);
                });
        }
        ':' => {
            super::url::send_url(input.chars().skip(1).collect::<String>(), sender, &id);
        }
        '/' => {
            super::search::handle_search(
                sender,
                input.chars().skip(1).collect::<String>(),
                &id,
                config,
            )
            .await;
        }
        '^' => {
            super::letter::process_greek_letters(
                input.chars().skip(1).collect::<String>(),
                sender,
                &id,
            );
        }
        _ => {
            process_general(sender, input, &id, config).await;
        }
    }
}

pub fn process_input(
    input: String,
    context: Rc<RefCell<AppContext>>,
    sender: UnboundedSender<DaemonEvt>,
    window: &Window,
    config: Arc<AppConf>,
) -> Result<(), DaemonErr> {
    let id = uuid::Uuid::new_v4();
    {
        *CURRENT_ID.lock().unwrap() = id;
    }

    let context_ref = &mut context.borrow_mut();
    context_ref.dvoty.dvoty_entries.clear();
    context_ref.dvoty.cur_ind = 0;
    context_ref.dvoty.target_scroll = 0.0f64;

    let task_map = &mut context_ref.dvoty.dvoty_tasks;

    if let Some(handle) = task_map.get(&DvotyTaskType::ProcessInput) {
        handle.abort();
        task_map.remove(&DvotyTaskType::ProcessInput);
    }

    let handle = tokio::spawn(async move {
        process_input_str(&input, sender.clone(), config).await;
    });

    task_map.insert(DvotyTaskType::ProcessInput, handle);

    let list = if let Some(l) = &context_ref.dvoty.dvoty_list {
        l
    } else if let Ok(res) = super::utils::get_list(window) {
        context_ref.dvoty.dvoty_list = Some(res);
        context_ref.dvoty.dvoty_list.as_ref().unwrap()
    } else {
        println!("Dvoty: can't find list");
        return Err(DaemonErr::CannotFindWidget);
    };

    list.remove_all();

    Ok(())
}
