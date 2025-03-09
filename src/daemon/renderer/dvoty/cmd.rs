use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{renderer::config::AppConf, structs::DaemonEvt};
use std::sync::Arc;

use super::{class::adjust_class, entry::DvotyUIEntry, DvotyContext};

pub fn populate_cmd_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    cmd: String,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) {
    let row = super::entry::create_base_entry(
        &config.dvoty.cmd_icon,
        &cmd,
        "Click to execute",
        sender,
        config.clone(),
        monitor,
    );

    let cmd_clone = cmd.clone();

    context.dvoty_entries[monitor].push((DvotyUIEntry::Command { exec: cmd_clone }, row.clone()));

    if context.dvoty_entries[monitor].len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty_entries[monitor]);
    }

    list.append(&row);
}
