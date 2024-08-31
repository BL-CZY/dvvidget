use std::time::Duration;

use crate::daemon::structs::{DaemonEvt, DaemonRes, Vol};
use crate::utils;
use crate::{daemon::structs::DaemonCmd, utils::DaemonErr};

use super::{
    app::register_widget,
    window::{self, WindowDescriptor},
};
use gtk4::{prelude::*, Adjustment, Application, ApplicationWindow, Scale, Window};
use tokio::sync::mpsc::UnboundedSender;

pub fn handle_vol_cmd(
    cmd: Vol,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Vol::Set(val) => {
            window
                .child()
                .and_downcast_ref::<Scale>()
                .unwrap()
                .set_value(utils::vol_round(val as f64));
        }
        Vol::Get => {
            return Ok(DaemonRes::VolGet(
                window.child().and_downcast_ref::<Scale>().unwrap().value(),
            ));
        }
        Vol::Inc(val) => {
            let value = window.child().and_downcast_ref::<Scale>().unwrap().value();

            window
                .child()
                .and_downcast_ref::<Scale>()
                .unwrap()
                .set_value(utils::vol_round(value + val as f64));
        }
        Vol::Dec(val) => {
            let value = window.child().and_downcast_ref::<Scale>().unwrap().value();

            window
                .child()
                .and_downcast_ref::<Scale>()
                .unwrap()
                .set_value(utils::vol_round(value - val as f64));
        }
        Vol::Close => {
            window.hide();
        }
        Vol::Open => {
            window.show();
        }
        Vol::OpenTime(f64) => {
            window.show();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs_f64(f64)).await;
                if let Err(e) = sender.send(DaemonEvt {
                    evt: DaemonCmd::Vol(Vol::Close),
                    sender: None,
                }) {
                    println!("Err closing the openned window: {}", e);
                }
            });
        }
    }

    Ok(DaemonRes::Success)
}

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
    register_widget(super::app::Widget::Volume, result.id());

    result
}
