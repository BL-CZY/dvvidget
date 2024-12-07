use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{
    renderer::{app::AppContext, config::AppConf},
    structs::DaemonEvt,
};
use std::{cell::RefMut, sync::Arc};

use super::{class::adjust_class, entry::DvotyUIEntry};

pub fn populate_cmd_entry(
    config: Arc<AppConf>,
    list: &ListBox,
    cmd: String,
    context: &mut RefMut<AppContext>,
    sender: UnboundedSender<DaemonEvt>,
) {
    let row =
        super::entry::create_base_entry(&config.dvoty.cmd_icon, &cmd, "Click to execute", sender);

    let cmd_clone = cmd.clone();

    context
        .dvoty
        .dvoty_entries
        .push((DvotyUIEntry::Command { exec: cmd_clone }, row.clone()));

    adjust_class(0, 0, &mut context.dvoty.dvoty_entries);

    list.append(&row);
}
