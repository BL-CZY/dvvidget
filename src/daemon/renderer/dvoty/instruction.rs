use std::cell::RefMut;
use std::sync::Arc;

use gtk4::{prelude::*, Image};
use gtk4::{Box, ListBox};
use gtk4::{Label, ListBoxRow};

use crate::daemon::renderer::app::AppContext;
use crate::daemon::renderer::config::AppConf;

use super::base::adjust_class;
use super::DvotyEntry;

fn create_instruction(instruction: &str, icon_path: &str) -> ListBoxRow {
    let label_start = Label::builder()
        .use_markup(true)
        .label(instruction)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let icon_end = Image::from_file(icon_path);
    icon_end.set_halign(gtk4::Align::End);
    icon_end.add_css_class("dvoty-icon");

    let result_box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    result_box.append(&label_start);
    result_box.append(&icon_end);

    let result = ListBoxRow::builder()
        .child(&result_box)
        .css_classes(["dvoty-entry"])
        .focusable(false)
        .build();

    result
}

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
        let entry = create_instruction(&instruction.0, &instruction.1);
        context
            .dvoty
            .dvoty_entries
            .push((DvotyEntry::Instruction, entry.clone()));
        list_box.append(&entry);
    }

    context.dvoty.cur_ind = 0;
    adjust_class(0, 0, &mut context.dvoty.dvoty_entries);
}
