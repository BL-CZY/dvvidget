use gtk4::{prelude::*, Box, Entry, ScrolledWindow};
use gtk4::{ListBox, Window};

pub enum UIErr {
    NotFound,
}

pub fn get_list(window: &Window) -> Result<ListBox, UIErr> {
    if let Some(outer_box) = window.child().and_downcast_ref::<Box>() {
        if let Some(inner_box) = outer_box.last_child() {
            if let Some(scroll) = inner_box.first_child() {
                if let Some(scroll_inner) = scroll.first_child() {
                    if let Some(result) = scroll_inner.first_child().and_downcast::<ListBox>() {
                        return Ok(result);
                    }
                }
            }
        }
    }

    Err(UIErr::NotFound)
}

pub fn get_scrolled_window(window: &Window) -> Result<ScrolledWindow, UIErr> {
    if let Some(outer_box) = window.child().and_downcast_ref::<Box>() {
        if let Some(inner_box) = outer_box.last_child() {
            if let Some(scroll) = inner_box.first_child().and_downcast::<ScrolledWindow>() {
                return Ok(scroll);
            }
        }
    }

    Err(UIErr::NotFound)
}

pub fn get_input(window: &Window) -> Result<Entry, UIErr> {
    if let Some(outer_box) = window.child().and_downcast_ref::<Box>() {
        if let Some(entry) = outer_box.first_child().and_downcast::<Entry>() {
            return Ok(entry);
        }
    }

    Err(UIErr::NotFound)
}

pub struct DesktopEntry {
    pub name: String,
    pub description: String,
    pub exec: String,
}

pub fn get_all_desktop_entries() -> Vec<DesktopEntry> {
    let paths = match std::env::var("XDG_DATA_DIRS") {
        Ok(v) => v.split(":").map(String::from).collect::<Vec<String>>(),
        Err(e) => {
            println!("Can't get deskfile paths: {}", e);
            return vec![];
        }
    };

    for path in paths {
        println!("{}", path);
    }

    vec![]
}
