use std::sync::Arc;

use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::renderer::config::AppConf;
use crate::daemon::structs::DaemonEvt;

use super::class::adjust_class;
use super::entry::{create_base_entry, DvotyUIEntry};
use super::DvotyContext;

pub fn populate_instructions(
    list_box: &ListBox,
    config: Arc<AppConf>,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) {
    let instructions: Vec<(String, String)> = vec![
        (
            "= for math expressions".into(),
            config.dvoty.instruction_icon.clone(),
        ),
        (
            "@ for launching apps".into(),
            config.dvoty.instruction_icon.clone(),
        ),
        (
            "$ for running commands".into(),
            config.dvoty.instruction_icon.clone(),
        ),
        (
            "/ for searching online".into(),
            config.dvoty.instruction_icon.clone(),
        ),
        (
            ": for opening url".into(),
            config.dvoty.instruction_icon.clone(),
        ),
        (
            "^ for special letters".into(),
            config.dvoty.instruction_icon.clone(),
        ),
    ];

    for instruction in instructions.iter() {
        let entry = create_base_entry(
            &instruction.1,
            &instruction.0,
            "",
            sender.clone(),
            config.clone(),
            monitor,
        );
        context.dvoty_entries[monitor].push((DvotyUIEntry::Instruction, entry.clone()));
        list_box.append(&entry);
    }

    adjust_class(0, 0, &mut context.dvoty_entries[monitor]);
}
