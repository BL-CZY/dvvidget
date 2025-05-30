use gtk4::{prelude::WidgetExt, ListBoxRow};

use super::entry::DvotyUIEntry;

fn set_class(target: &ListBoxRow, remove_class: &[&str], add_class: &[&str]) {
    for class in remove_class.iter() {
        target.remove_css_class(class);
    }

    for class in add_class.iter() {
        target.add_css_class(class);
    }
}

pub fn adjust_class(old: usize, new: usize, input: &mut [(DvotyUIEntry, ListBoxRow)]) {
    if old >= input.len() || new >= input.len() {
        return;
    }

    match input[old].0 {
        DvotyUIEntry::Instruction => {
            set_class(
                &input[old].1,
                &["dvoty-entry-instruction-select", "dvoty-entry-select"],
                &["dvoty-entry-instruction", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Math { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-math-select", "dvoty-entry-select"],
                &["dvoty-entry-math", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Search { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-search-select", "dvoty-entry-select"],
                &["dvoty-entry-search", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Url { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-url-select", "dvoty-entry-select"],
                &["dvoty-entry-url", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Command { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-cmd-select", "dvoty-entry-select"],
                &["dvoty-entry-cmd", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Launch { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-launch-select", "dvoty-entry-select"],
                &["dvoty-entry-launch", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Letter { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-letter-select", "dvoty-entry-select"],
                &["dvoty-entry-letter", "dvoty-entry"],
            );
        }
        DvotyUIEntry::File { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-file-select", "dvoty-entry-select"],
                &["dvoty-entry-file", "dvoty-entry"],
            );
        }
    }

    match input[new].0 {
        DvotyUIEntry::Instruction => {
            set_class(
                &input[new].1,
                &["dvoty-entry-instruction", "dvoty-entry"],
                &["dvoty-entry-instruction-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Math { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-math", "dvoty-entry"],
                &["dvoty-entry-math-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Search { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-search", "dvoty-entry"],
                &["dvoty-entry-search-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Url { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-url", "dvoty-entry"],
                &["dvoty-entry-url-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Command { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-cmd", "dvoty-entry"],
                &["dvoty-entry-cmd-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Launch { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-launch", "dvoty-entry"],
                &["dvoty-entry-launch-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Letter { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-letter", "dvoty-entry"],
                &["dvoty-entry-letter-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::File { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-file", "dvoty-entry"],
                &["dvoty-entry-file-select", "dvoty-entry-select"],
            );
        }
    }
}
