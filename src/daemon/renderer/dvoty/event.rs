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
    monitors: Vec<usize>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Dvoty::Update(str) => {
            super::input::process_input(
                str,
                context,
                sender.clone(),
                windows,
                config.clone(),
                monitors,
            )?;
        }

        Dvoty::AddEntry(entry) => {
            super::entry::add_entry(entry, windows, context, config, sender.clone(), monitors)?;
        }

        Dvoty::IncEntryIndex => {
            if !context.dvoty_entries.is_empty() {
                let old = context.cur_ind[monitors];
                let max = context.dvoty_entries[monitors].len() - 1;
                context.cur_ind[monitors] += 1;
                if context.cur_ind[monitors] > max {
                    context.cur_ind[monitors] = 0;
                }
                let new = context.cur_ind[monitors];
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitors]);
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitors],
                    sender.clone(),
                    monitors,
                )?;
            }
        }

        Dvoty::DecEntryIndex => {
            if !context.dvoty_entries.is_empty() {
                let old = context.cur_ind[monitors];
                let max = context.dvoty_entries[monitors].len() - 1;
                if context.cur_ind[monitors] == 0 {
                    context.cur_ind[monitors] = max;
                } else {
                    context.cur_ind[monitors] -= 1;
                }
                let new = context.cur_ind[monitors];
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitors]);
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitors],
                    sender.clone(),
                    monitors,
                )?;
            }
        }

        Dvoty::ScrollStart => {
            if !context.dvoty_entries[monitors].is_empty() {
                let old = context.cur_ind[monitors];
                context.cur_ind[monitors] = 0;
                let new = 0;
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitors]);
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitors],
                    sender.clone(),
                    monitors,
                )?;
            }
        }

        Dvoty::ScrollEnd => {
            if !context.dvoty_entries[monitors].is_empty() {
                let old = context.cur_ind[monitors];
                context.cur_ind[monitors] = context.dvoty_entries[monitors].len() - 1;
                let new = context.dvoty_entries[monitors].len() - 1;
                super::class::adjust_class(old, new, &mut context.dvoty_entries[monitors].clone());
                super::row::ensure_row_in_viewport(
                    context,
                    &windows[monitors],
                    sender.clone(),
                    monitors,
                )?;
            }
        }

        Dvoty::TriggerEntry => {
            if !context.dvoty_entries[monitors].is_empty() {
                context.dvoty_entries[monitors][context.cur_ind[monitors]]
                    .0
                    .clone()
                    .run(config);
            }
            windows[monitors].set_visible(false);
        }

        Dvoty::Open => {
            windows[monitors].set_visible(true);
            if let Ok(input) = get_input(&windows[monitors]) {
                input.select_region(0, -1);
            }
        }

        Dvoty::Close => {
            windows[monitors].set_visible(false);
        }

        Dvoty::Toggle => {
            if windows[monitors].is_visible() {
                windows[monitors].set_visible(false);
            } else {
                windows[monitors].set_visible(true);
            }
        }

        Dvoty::SetScroll(val) => {
            super::row::set_scroll(context, &windows[monitors], val, monitors)?;
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
