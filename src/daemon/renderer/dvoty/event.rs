use std::sync::Arc;

use gtk4::{prelude::EditableExt, prelude::WidgetExt, Window};
use once_cell::sync::OnceCell;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::{
        renderer::config::AppConf,
        structs::{DaemonCmdType, DaemonEvt, DaemonRes, Dvoty},
    },
    utils::DaemonErr,
};

use std::sync::Mutex;

use super::{utils::get_input, DvotyContext};

pub static CURRENT_IDS: OnceCell<Vec<Arc<Mutex<uuid::Uuid>>>> = OnceCell::new();

fn handle_dvoty_cmd_single(
    cmd: Dvoty,
    windows: &[Window],
    sender: UnboundedSender<DaemonEvt>,
    context: &mut DvotyContext,
    config: Arc<AppConf>,
    monitor: usize,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Dvoty::Update(str) => {
            super::input::process_input(
                str.clone(),
                context,
                sender.clone(),
                windows,
                config.clone(),
                monitor,
            )?;
        }

        Dvoty::AddEntry(entry) => {
            super::entry::add_entry(
                entry.clone(),
                windows,
                context,
                config.clone(),
                sender.clone(),
                monitor,
            )?;
        }

        Dvoty::IncEntryIndex => {
            if !context.dvoty_entries[monitor].is_empty() {
                let old = context.cur_ind[monitor];
                let max = context.dvoty_entries[monitor].len() - 1;
                context.cur_ind[monitor] += 1;
                if context.cur_ind[monitor] > max {
                    context.cur_ind[monitor] = 0;
                }
                let new = context.cur_ind[monitor];
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitor]);
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitor],
                    sender.clone(),
                    monitor,
                )?;
            }
        }

        Dvoty::DecEntryIndex => {
            if !context.dvoty_entries[monitor].is_empty() {
                let old = context.cur_ind[monitor];
                let max = context.dvoty_entries[monitor].len() - 1;
                if context.cur_ind[monitor] == 0 {
                    context.cur_ind[monitor] = max;
                } else {
                    context.cur_ind[monitor] -= 1;
                }
                let new = context.cur_ind[monitor];
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitor]);
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitor],
                    sender.clone(),
                    monitor,
                )?;
            }
        }

        Dvoty::ScrollStart => {
            if !context.dvoty_entries[monitor].is_empty() {
                let old = context.cur_ind[monitor];
                context.cur_ind[monitor] = 0;
                let new = 0;
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitor]);
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitor],
                    sender.clone(),
                    monitor,
                )?;
            }
        }

        Dvoty::ScrollEnd => {
            if !context.dvoty_entries[monitor].is_empty() {
                let old = context.cur_ind[monitor];
                context.cur_ind[monitor] = context.dvoty_entries[monitor].len() - 1;
                let new = context.dvoty_entries[monitor].len() - 1;
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitor].clone());
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitor],
                    sender.clone(),
                    monitor,
                )?;
            }
        }

        Dvoty::TriggerEntry => {
            if !context.dvoty_entries[monitor].is_empty() {
                context.dvoty_entries[monitor][context.cur_ind[monitor]]
                    .0
                    .clone()
                    .run(config.clone());
            }
            windows[monitor].set_visible(false);
        }

        Dvoty::Open => {
            windows[monitor].set_visible(true);
            if let Ok(input) = get_input(&windows[monitor]) {
                input.select_region(0, -1);
            }
        }

        Dvoty::Close => {
            windows[monitor].set_visible(false);
        }

        Dvoty::Toggle => {
            if windows[monitor].is_visible() {
                windows[monitor].set_visible(false);
            } else {
                windows[monitor].set_visible(true);
            }
        }

        Dvoty::SetScroll(val) => {
            super::row::set_scroll(context, &windows[monitor], val, monitor)?;
        }
    }

    Ok(DaemonRes::Success)
}

pub fn handle_dvoty_cmd(
    cmd: Dvoty,
    windows: &[Window],
    sender: UnboundedSender<DaemonEvt>,
    context: &mut DvotyContext,
    config: Arc<AppConf>,
    monitors: Vec<usize>,
    id: Option<uuid::Uuid>,
) -> Result<DaemonRes, DaemonErr> {
    // dvoty events all only have one monitor, so it's fine to have one id

    for monitor in monitors {
        if let Some(uuid) = id {
            if *CURRENT_IDS.get().unwrap()[monitor]
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                != uuid
            {
                continue;
            }
        }

        let _ = handle_dvoty_cmd_single(
            cmd.clone(),
            windows,
            sender.clone(),
            context,
            config.clone(),
            monitor,
        );
    }

    Ok(DaemonRes::Success)
}

pub fn send_inc(sender: UnboundedSender<DaemonEvt>, monitor: Vec<usize>) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::IncEntryIndex),
            sender: None,
            uuid: None,
            monitors: monitor,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send inc index: {}", e));
}

pub fn send_dec(sender: UnboundedSender<DaemonEvt>, monitor: Vec<usize>) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::DecEntryIndex),
            sender: None,
            uuid: None,
            monitors: monitor,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send dec index: {}", e));
}
