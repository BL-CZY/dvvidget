use std::{cell::RefCell, rc::Rc, sync::Arc};

use gtk4::{prelude::EditableExt, prelude::WidgetExt, Window};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    daemon::{
        renderer::{app::AppContext, config::AppConf},
        structs::{DaemonCmd, DaemonEvt, DaemonRes, Dvoty},
    },
    utils::DaemonErr,
};

use super::utils::get_input;

pub fn handle_dvoty_cmd(
    cmd: Dvoty,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
    app_context: Rc<RefCell<AppContext>>,
    config: Arc<AppConf>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Dvoty::Update(str) => {
            super::input::process_input(str, app_context, sender.clone(), window)?;
        }

        Dvoty::AddEntry(entry) => {
            super::entry::add_entry(entry, window, app_context, config, sender.clone())?;
        }

        Dvoty::IncEntryIndex => {
            let mut context_ref = app_context.borrow_mut();

            if !context_ref.dvoty.dvoty_entries.is_empty() {
                let old = context_ref.dvoty.cur_ind;
                let max = context_ref.dvoty.dvoty_entries.len() - 1;
                context_ref.dvoty.cur_ind += 1;
                if context_ref.dvoty.cur_ind > max {
                    context_ref.dvoty.cur_ind = 0;
                }
                let new = context_ref.dvoty.cur_ind;
                super::class::adjust_class(old, new, &mut context_ref.dvoty.dvoty_entries.clone());
                super::row::ensure_row_in_viewport(&mut context_ref, window, sender.clone())?;
            }
        }

        Dvoty::DecEntryIndex => {
            let mut context_ref = app_context.borrow_mut();

            if !context_ref.dvoty.dvoty_entries.is_empty() {
                let old = context_ref.dvoty.cur_ind;
                let max = context_ref.dvoty.dvoty_entries.len() - 1;
                if context_ref.dvoty.cur_ind == 0 {
                    context_ref.dvoty.cur_ind = max;
                } else {
                    context_ref.dvoty.cur_ind -= 1;
                }
                let new = context_ref.dvoty.cur_ind;
                super::class::adjust_class(old, new, &mut context_ref.dvoty.dvoty_entries.clone());
                super::row::ensure_row_in_viewport(&mut context_ref, window, sender.clone())?;
            }
        }

        Dvoty::TriggerEntry => {
            let context_ref = app_context.borrow();
            if !context_ref.dvoty.dvoty_entries.is_empty() {
                context_ref.dvoty.dvoty_entries[context_ref.dvoty.cur_ind]
                    .0
                    .clone()
                    .run(config);
            }
            window.set_visible(false);
        }

        Dvoty::Open => {
            window.set_visible(true);
            if let Ok(input) = get_input(window) {
                input.select_region(0, -1);
            }
        }

        Dvoty::Close => {
            window.set_visible(false);
        }

        Dvoty::SetScroll(val) => {
            super::row::set_scroll(&mut app_context.borrow_mut(), window, val)?;
        }
    }

    Ok(DaemonRes::Success)
}

pub fn send_inc(sender: UnboundedSender<DaemonEvt>) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::IncEntryIndex),
            sender: None,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send inc index: {}", e));
}

pub fn send_dec(sender: UnboundedSender<DaemonEvt>) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::DecEntryIndex),
            sender: None,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send dec index: {}", e));
}
