use super::window::{self, WindowDescriptor};
use gtk4::{prelude::*, Adjustment, Application, ApplicationWindow, Scale};

fn get_volume() -> f64 {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("wpctl get-volume @DEFAULT_AUDIO_SINK@")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let volume_str = stdout.trim().split_whitespace().nth(1).unwrap_or_default();

    volume_str.parse::<f64>().unwrap() * 100f64
}

pub fn create_sound_osd(app: &Application) -> ApplicationWindow {
    let mut descriptor = WindowDescriptor::new();
    descriptor.anchor_bottom = true;
    descriptor.margin_bottom = 130;

    let result = window::create_window(app, descriptor);
    result.add_css_class("sound");
    let adjustment = Adjustment::new(get_volume(), 0.0, 100.0, 1.0, 0.0, 0.0);
    let scale = Scale::new(gtk4::Orientation::Horizontal, Some(&adjustment));
    scale.set_width_request(100);
    scale.add_css_class("sound_scale");

    scale.connect_value_changed(|scale| {
        if let Err(e) = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "wpctl set-volume @DEFAULT_AUDIO_SINK@ {}%",
                scale.value()
            ))
            .output()
        {
            println!("failed: {}", e);
        };
    });

    result.set_child(Some(scale).as_ref());
    result.set_startup_id("vol-popup");

    result
}
