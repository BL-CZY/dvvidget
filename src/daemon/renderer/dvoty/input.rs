use std::{cell::RefCell, rc::Rc};

use gtk4::Window;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::{
        renderer::app::AppContext,
        structs::{DaemonCmd, DaemonEvt, Dvoty},
    },
    utils::DaemonErr,
};

use super::{DvotyEntry, DvotyTaskType};

fn process_input_str(input: &str, sender: UnboundedSender<DaemonEvt>) {
    if input.is_empty() {
        if let Err(e) = sender.send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Empty)),
            sender: None,
        }) {
            println!("Dvoty: Failed to send entry: {}, ignoring...", e);
        };
        return;
    }

    match input.chars().next().unwrap() {
        '=' => {
            super::math::eval_math(input, sender);
        }
        '@' => {
            super::app_launcher::process_apps({
                if input.len() == 1 {
                    ""
                } else {
                    &input[1..]
                }
            });
        }
        '$' => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Command {
                        exec: input.chars().skip(1).collect::<String>(),
                    })),
                    sender: None,
                })
                .unwrap_or_else(|e| {
                    println!("Dvoty: Failed to send command: {}", e);
                });
        }
        ':' => {
            super::url::send_url(input.chars().skip(1).collect::<String>(), sender);
        }
        '/' => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Search {
                        keyword: input.chars().skip(1).collect::<String>(),
                    })),
                    sender: None,
                })
                .unwrap_or_else(|e| {
                    println!("Dvoty: Error adding search entry: {}", e);
                });
        }
        _ => {}
    }
}

pub fn process_input(
    input: String,
    context: Rc<RefCell<AppContext>>,
    sender: UnboundedSender<DaemonEvt>,
    window: &Window,
) -> Result<(), DaemonErr> {
    let context_ref = &mut context.borrow_mut();
    context_ref.dvoty.dvoty_entries.clear();
    context_ref.dvoty.cur_ind = 0;
    context_ref.dvoty.target_scroll = 0.0f64;

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

    let task_map = &mut context_ref.dvoty.dvoty_tasks;

    if let Some(handle) = task_map.get(&DvotyTaskType::ProcessInput) {
        handle.abort();
        task_map.remove(&DvotyTaskType::ProcessInput);
    }

    let handle = tokio::spawn(async move {
        process_input_str(&input, sender.clone());
    });

    task_map.insert(DvotyTaskType::ProcessInput, handle);

    Ok(())
}
