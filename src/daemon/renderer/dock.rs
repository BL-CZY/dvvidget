use gtk4::{prelude::*, EventControllerMotion};
use gtk4::{Application, ApplicationWindow, Box, Button, Orientation};
use std::cell::{RefCell, RefMut};
use std::cmp::max;
use std::rc::Rc;

use super::window::{self, WindowDescriptor};

pub struct DockDescriptor {
    window: WindowDescriptor,
    items: Vec<String>,
}

impl DockDescriptor {
    pub fn new() -> Self {
        DockDescriptor {
            window: WindowDescriptor::new(),
            items: vec![],
        }
    }
}

pub fn adjust_btn(list: RefMut<Vec<Button>>, cursor_x: f64) {
    for btn in list.iter() {
        let location = btn.allocation().x() as f64;
        let width = btn.allocation().width() as f64;
        let size = (150f64 - (cursor_x - (location + width / 2f64)).abs() * 0.3f64).max(75f64);
        println!("{}, {}, {}", cursor_x, location, size);
        btn.set_height_request(size as i32);
        btn.set_width_request(size as i32);
    }
}

pub fn create_dock(app: &Application) -> ApplicationWindow {
    let mut descriptor = WindowDescriptor::new();
    descriptor.anchor_bottom = true;
    descriptor.anchor_left = false;
    descriptor.anchor_right = false;
    let result = window::create_window(app, descriptor);

    let list_box = Box::new(Orientation::Horizontal, 10);
    let btns: Rc<RefCell<Vec<Button>>> = Rc::new(RefCell::new(vec![]));

    for num in 0..10 {
        let btn = Button::with_label(&num.to_string());
        btn.set_height_request(75);
        btn.set_width_request(75);
        btn.set_halign(gtk4::Align::End);
        btn.set_valign(gtk4::Align::End);
        list_box.append(&btn);
        btns.borrow_mut().push(btn);
    }

    list_box.set_margin_start(10);
    list_box.set_margin_end(10);

    let motion = EventControllerMotion::new();

    let btns_clone = btns.clone();
    motion.connect_motion(move |_, x, _| {
        let list_btn = btns_clone.borrow_mut();
        adjust_btn(list_btn, x);
    });

    let btns_clone = btns.clone();
    motion.connect_leave(move |_| {
        let list_btn = btns_clone.borrow_mut();
        for btn in list_btn.iter() {
            btn.set_width_request(75);
            btn.set_height_request(75);
        }
    });

    result.add_controller(motion);

    result.set_child(Some(&list_box));

    result
}
