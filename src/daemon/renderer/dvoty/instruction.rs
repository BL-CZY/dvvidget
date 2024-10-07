use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{Box, ListBox};
use gtk4::{Label, ListBoxRow};

use crate::daemon::renderer::config::AppConf;

fn create_instruction(instruction: &str, icon: &str) -> ListBoxRow {
    let label_start = Label::builder()
        .use_markup(true)
        .label(instruction)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let label_end = Label::builder()
        .use_markup(true)
        .label(icon)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::End)
        .hexpand(true)
        .build();

    let result_box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["dvoty-entry"])
        .build();

    result_box.append(&label_start);
    result_box.append(&label_end);

    let result = ListBoxRow::builder()
        .child(&result_box)
        .focusable(false)
        .build();

    result
}

pub fn populate_instructions(list_box: &ListBox, config: Arc<AppConf>) {
    let instructions = vec![
        (format!("input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> for math expressions", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> @ </span> for launching apps", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> $ </span> for running commands", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> / </span> for searching online", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> : </span> for opening url", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
    ];
    for instruction in instructions.iter() {
        list_box.append(&create_instruction(&instruction.0, &instruction.1));
    }
}
