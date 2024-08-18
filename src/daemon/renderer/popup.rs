use super::window::{self, WindowDescriptor};
use gtk4::{prelude::WidgetExt, Application, ApplicationWindow};

pub fn create_sound_osd(app: &Application) -> ApplicationWindow {
    let mut descriptor = WindowDescriptor::new();
    descriptor.anchor_bottom = true;
    descriptor.margin_bottom = 130;

    let result = window::create_window(app, descriptor);
    result.set_width_request(100);
    result.set_height_request(40);

    result
}
