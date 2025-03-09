use std::sync::Arc;

use gtk4::{prelude::EditableExt, prelude::WidgetExt, Window};
use lazy_static::lazy_static;
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

lazy_static! {
    pub static ref CURRENT_ID: Arc<Mutex<uuid::Uuid>> = Arc::new(Mutex::new(uuid::Uuid::new_v4()));
}

pub fn handle_dvoty_cmd(
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
                str,
                context,
                sender.clone(),
                windows,
                config.clone(),
                monitor,
            )?;
        }

        Dvoty::AddEntry(entry) => {
            super::entry::add_entry(entry, windows, context, config, sender.clone(), monitor)?;
        }

        Dvoty::IncEntryIndex => {
            if !context.dvoty_entries.is_empty() {
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
            if !context.dvoty_entries.is_empty() {
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
                    .run(config);
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

pub fn send_inc(sender: UnboundedSender<DaemonEvt>, monitor: usize) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::IncEntryIndex),
            sender: None,
            uuid: None,
            monitor,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send inc index: {}", e));
}

pub fn send_dec(sender: UnboundedSender<DaemonEvt>, monitor: usize) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::DecEntryIndex),
            sender: None,
            uuid: None,
            monitor,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send dec index: {}", e));
}
