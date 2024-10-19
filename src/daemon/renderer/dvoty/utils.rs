use gtk4::{prelude::*, Box, ScrolledWindow};
use gtk4::{ListBox, Window};

pub fn get_list(window: &Window) -> Result<ListBox, ()> {
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

    Err(())
}

pub fn get_scrolled_window(window: &Window) -> Result<ScrolledWindow, ()> {
    if let Some(outer_box) = window.child().and_downcast_ref::<Box>() {
        if let Some(inner_box) = outer_box.last_child() {
            if let Some(scroll) = inner_box.first_child().and_downcast::<ScrolledWindow>() {
                return Ok(scroll);
            }
        }
    }

    Err(())
}
