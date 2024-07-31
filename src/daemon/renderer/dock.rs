use gtk4::{Application, ApplicationWindow};

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

pub fn create_dock(app: &Application) -> ApplicationWindow {
    let mut descriptor = WindowDescriptor::new();
    descriptor.anchor_bottom = true;
    descriptor.anchor_left = true;
    descriptor.anchor_right = true;
    descriptor.margin_left = 40;
    descriptor.margin_right = 40;
    let result = window::create_window(app, descriptor);
    result
}
