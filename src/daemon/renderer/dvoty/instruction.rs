use std::cell::RefMut;
use std::sync::Arc;

use gtk4::ListBox;

use crate::daemon::renderer::app::AppContext;
use crate::daemon::renderer::config::AppConf;

use super::class::adjust_class;
use super::entry::{create_base_entry, DvotyUIEntry};

pub fn populate_instructions(
    list_box: &ListBox,
    config: Arc<AppConf>,
    context: &mut RefMut<AppContext>,
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
    ];

    for instruction in instructions.iter() {
        let entry = create_base_entry(&instruction.1, &instruction.0, "");
        context
            .dvoty
            .dvoty_entries
            .push((DvotyUIEntry::Instruction, entry.clone()));
        list_box.append(&entry);
    }

    context.dvoty.cur_ind = 0;
    adjust_class(0, 0, &mut context.dvoty.dvoty_entries);
}
