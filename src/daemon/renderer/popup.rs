use super::window::{self, WindowDescriptor};
use gtk4::{prelude::*, Adjustment, Application, ApplicationWindow, Scale};

pub fn create_sound_osd(app: &Application) -> ApplicationWindow {
    let mut descriptor = WindowDescriptor::new();
    descriptor.anchor_bottom = true;
    descriptor.margin_bottom = 130;

    let result = window::create_window(app, descriptor);
    result.add_css_class("sound");
    let adjustment = Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 10.0);
    let scale = Scale::new(gtk4::Orientation::Horizontal, Some(&adjustment));
    scale.set_width_request(100);
    scale.add_css_class("sound_scale");

    scale.connect_value_changed(|scale| {
        println!("{}", scale.value());
    });

    result.set_child(Some(scale).as_ref());

    result
}
