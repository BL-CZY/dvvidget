use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::daemon::structs::{DaemonEvt, DaemonRes, Vol};
use crate::utils::{self, DisplayBackend};
use crate::{daemon::structs::DaemonCmd, utils::DaemonErr};

use super::app::VolTaskType;
use super::config::{AppConf, VolCmdProvider};
use super::{app::register_widget, window};
use gtk4::{prelude::*, Adjustment, Application, ApplicationWindow, Scale, Window};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

fn murph(
    sender: UnboundedSender<DaemonEvt>,
    mut current: f64,
    vol_tasks: Rc<RefCell<HashMap<VolTaskType, JoinHandle<()>>>>,
    target: f64,
    config: Arc<AppConf>,
) {
    let mut map_ref = vol_tasks.borrow_mut();
    if let Some(handle) = map_ref.get(&VolTaskType::MurphValue) {
        handle.abort();
        map_ref.remove(&VolTaskType::MurphValue);
    }

    match config.vol.run_cmd {
        VolCmdProvider::Wpctl => {
            if let Err(e) = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("wpctl set-volume @DEFAULT_AUDIO_SINK@ {}%", target))
                .output()
            {
                println!("Failed to set volume: {}", e);
            };
        }

        VolCmdProvider::NoCmd => {}
    }

    let handle = tokio::spawn(async move {
        for _ in 0..50 {
            current += (target - current) * 0.1f64;
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Vol(Vol::SetRough(current)),
                    sender: None,
                })
                .unwrap_or_else(|e| println!("failed to update: {}", e));
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        sender
            .send(DaemonEvt {
                evt: DaemonCmd::Vol(Vol::SetRough(target)),
                sender: None,
            })
            .unwrap_or_else(|e| println!("failed to update: {}", e));

        sender
            .send(DaemonEvt {
                evt: DaemonCmd::Vol(Vol::StopCurValTask),
                sender: None,
            })
            .unwrap_or_else(|e| println!("failed to terminate this task: send err: {}", e));
    });

    map_ref.insert(VolTaskType::MurphValue, handle);
}

pub fn handle_vol_cmd(
    cmd: Vol,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
    vol_tasks: Rc<RefCell<HashMap<VolTaskType, JoinHandle<()>>>>,
    config: Arc<AppConf>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Vol::StopCurValTask => {
            let mut map_ref = vol_tasks.borrow_mut();
            map_ref.remove(&VolTaskType::MurphValue);
        }
        Vol::SetRough(val) => {
            window
                .child()
                .and_downcast_ref::<Scale>()
                .unwrap()
                .set_value(val);
        }
        Vol::Set(val) => {
            let current = window.child().and_downcast_ref::<Scale>().unwrap().value();
            let target = utils::vol_round_down(val);
            murph(sender, current, vol_tasks, target, config);
        }
        Vol::Get => {
            return Ok(DaemonRes::VolGet(
                window.child().and_downcast_ref::<Scale>().unwrap().value(),
            ));
        }
        Vol::Inc(val) => {
            let current = window.child().and_downcast_ref::<Scale>().unwrap().value();
            let target = utils::vol_round_up(current + val);
            murph(sender, current, vol_tasks, target, config);
        }
        Vol::Dec(val) => {
            let current = window.child().and_downcast_ref::<Scale>().unwrap().value();
            let target = utils::vol_round_down(current - val);
            murph(sender, current, vol_tasks, target, config);
        }
        Vol::Close => {
            window.hide();
        }
        Vol::Open => {
            window.show();
        }
        Vol::OpenTime(time) => {
            window.show();
            tokio::spawn(async move {
                if let Err(e) = sender.send(DaemonEvt {
                    evt: DaemonCmd::RegVolClose(time),
                    sender: None,
                }) {
                    println!("Failed to register close: {}", e);
                }

                tokio::time::sleep(Duration::from_secs_f64(time)).await;

                if let Err(e) = sender.send(DaemonEvt {
                    evt: DaemonCmd::ExecVolClose(time),
                    sender: None,
                }) {
                    println!("Err closing the openned window: {}", e);
                }
            });
        }
    }

    Ok(DaemonRes::Success)
}

fn get_volume(cmd: VolCmdProvider) -> f64 {
    match cmd {
        VolCmdProvider::Wpctl => {
            let output = if let Ok(out) = std::process::Command::new("wpctl")
                .arg("get-volume")
                .arg("@DEFAULT_AUDIO_SINK@")
                .output()
            {
                out
            } else {
                return 0.0;
            };

            let stdout = String::from_utf8_lossy(&output.stdout);
            let volume_str = stdout.split_whitespace().nth(1).unwrap_or_default();

            volume_str.parse::<f64>().unwrap_or_default() * 100f64
        }

        VolCmdProvider::NoCmd => 0f64,
    }
}

pub fn create_sound_osd(
    backend: DisplayBackend,
    app: &Application,
    config: Arc<AppConf>,
) -> ApplicationWindow {
    let descriptor = config.vol.window.clone();

    let result = window::create_window(backend, app, descriptor);
    result.add_css_class("sound");
    let adjustment = Adjustment::new(
        get_volume(config.vol.run_cmd.clone()),
        0.0,
        config.vol.max_vol,
        0.1,
        0.0,
        0.0,
    );
    let scale = Scale::new(gtk4::Orientation::Horizontal, Some(&adjustment));
    scale.set_width_request(100);
    scale.add_css_class("sound_scale");
    scale.set_sensitive(false);

    result.set_child(Some(scale).as_ref());
    register_widget(super::app::Widget::Volume, result.id());

    result.present();

    if !config.vol.window.visible_on_start {
        result.hide();
    }

    result
}
