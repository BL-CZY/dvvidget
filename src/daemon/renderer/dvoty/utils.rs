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

#[derive(Debug)]
pub struct DesktopEntry {
    pub name: String,
    pub description: String,
    pub exec: String,
}

fn read_desktop_file(path: &std::path::Path) -> Vec<DesktopEntry> {
    let mut result = vec![];
    let content = if let Ok(res) = std::fs::read_to_string(path) {
        res
    } else {
        return vec![];
    };

    result
}

pub fn get_desktop_entries() -> Vec<DesktopEntry> {
    let paths = match std::env::var("XDG_DATA_DIRS") {
        Ok(v) => v.split(":").map(String::from).collect::<Vec<String>>(),
        Err(e) => {
            println!("Can't get deskfile paths: {}", e);
            return vec![];
        }
    };

    use walkdir::WalkDir;

    let mut result = vec![];

    for path in paths {
        for entry in WalkDir::new(path) {
            if let Ok(res) = entry {
                let name = res.file_name();
                if let Some(str) = name.to_str() {
                    if str.ends_with(".desktop") {
                        result.extend(read_desktop_file(res.path()));
                    }
                } else {
                    println!("Dvoty: can't access OsStr: {:?}", name);
                }
            }
        }
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_desktop_files() {
        let files = get_desktop_entries();
        println!("{:?}", files);
    }
}
