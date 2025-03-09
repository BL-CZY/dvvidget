use gtk4::{prelude::*, Box, Entry, ScrolledWindow};
use gtk4::{ListBox, Window};

pub fn create_list_of<T: Default>(count: usize) -> Vec<T> {
    let mut res = vec![];
    for _ in 0..count {
        res.push(T::default());
    }

    res
}

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
