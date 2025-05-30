use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use crate::daemon::structs::{Bri, DaemonEvt, DaemonRes};
use crate::utils::{self, DisplayBackend};
use crate::{daemon::structs::DaemonCmdType, utils::DaemonErr};

use super::app::{VolBriTaskType, VolBriTaskTypeWindow};
use super::config::{AppConf, BriCmdProvider};
use super::{app::register_widget, window};
use gtk4::{
    prelude::*, Adjustment, Application, ApplicationWindow, Box, Image, Label, Scale, Window,
};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

pub struct BriContext {
    pub cur_bri: f64,
    pub bri_tasks_window: Vec<HashMap<VolBriTaskTypeWindow, JoinHandle<()>>>,
    pub bri_tasks: HashMap<VolBriTaskType, JoinHandle<()>>,
}

impl BriContext {
    pub fn from_config(config: &Arc<AppConf>, monitor_count: usize) -> Self {
        let cur_bri = get_bri(&config.bri.run_cmd);
        BriContext {
            cur_bri,
            bri_tasks_window: {
                let mut res = vec![];
                for _ in 0..monitor_count {
                    res.push(HashMap::new());
                }
                res
            },
            bri_tasks: HashMap::new(),
        }
    }

    pub fn set_virtual_brightness(&mut self, val: f64) -> f64 {
        if val > 100f64 {
            self.cur_bri = 100f64;
        } else if val < 0f64 {
            self.cur_bri = 0f64;
        } else {
            self.cur_bri = val;
        }

        self.cur_bri
    }
}

fn update_display_info(config: Arc<AppConf>, windows: &[Window], val: f64) {
    for window in windows {
        let child = if let Some(w) = window.child() {
            w
        } else {
            println!("Vol: can't find the box");
            return;
        };

        if let Some(widget) = child.first_child() {
            if let Some(label) = widget.downcast_ref::<Label>() {
                set_icon(config.clone(), IconRefHolder::Text(label), val);
            } else if let Some(pic) = widget.downcast_ref::<Image>() {
                set_icon(config.clone(), IconRefHolder::Svg(pic), val);
            }
        }

        if let Some(widget) = child.last_child() {
            if let Some(label) = widget.downcast_ref::<Label>() {
                label.set_text(&(val as i64).to_string());
            }
        }
    }
}

fn murph(
    sender: UnboundedSender<DaemonEvt>,
    mut current: f64,
    context: &mut BriContext,
    target: f64,
    config: Arc<AppConf>,
    window: &[Window],
    monitors: Vec<usize>,
) {
    // shadowing target to adjust it to an appropriate value
    let target = context.set_virtual_brightness(target);
    let task_map = &mut context.bri_tasks;
    if let Some(handle) = task_map.get(&VolBriTaskType::MurphValue) {
        handle.abort();
        task_map.remove(&VolBriTaskType::MurphValue);
    }

    set_bri(&config.bri.run_cmd, target);
    update_display_info(config.clone(), window, target);

    let handle = tokio::spawn(async move {
        for _ in 0..50 {
            current += (target - current) * 0.1f64;
            sender
                .send(DaemonEvt {
                    evt: DaemonCmdType::Bri(Bri::SetRough(current)),
                    sender: None,
                    uuid: None,
                    monitors: monitors.clone(),
                })
                .unwrap_or_else(|e| println!("Bri: failed to update: {}", e));
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        sender
            .send(DaemonEvt {
                evt: DaemonCmdType::Bri(Bri::SetRough(target)),
                sender: None,
                uuid: None,
                monitors,
            })
            .unwrap_or_else(|e| println!("Bri: failed to update: {}", e));
    });

    task_map.insert(VolBriTaskType::MurphValue, handle);
}

fn set_rough(val: f64, windows: &[Window]) {
    for window in windows {
        let child = if let Some(widget) = window
            .child()
            .and_downcast_ref::<Box>()
            .unwrap()
            .first_child()
        {
            widget
        } else {
            println!("Bri: Failed to downcast the box");
            continue;
        };

        if let Some(scale) = child.downcast_ref::<Scale>() {
            scale.set_value(val);
            continue;
        }

        let mut found = false;
        while let Some(widget) = child.next_sibling() {
            if let Some(scale) = widget.downcast_ref::<Scale>() {
                scale.set_value(val);
                found = true;
                break;
            }
        }

        if found {
            continue;
        }

        println!("Bri: Couldn't find the scale, ignoring...");
    }
}

pub fn handle_bri_cmd(
    cmd: Bri,
    windows: &[Window],
    sender: UnboundedSender<DaemonEvt>,
    context: &mut BriContext,
    config: Arc<AppConf>,
    monitors: Vec<usize>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Bri::SetRough(val) => {
            set_rough(val, windows);
        }
        Bri::Set(val) => {
            let current = context.cur_bri;
            let target = utils::round_down(val);
            murph(sender, current, context, target, config, windows, monitors);
        }
        Bri::Get => {
            return Ok(DaemonRes::GetBri(context.cur_bri));
        }
        Bri::Inc(val) => {
            let current = context.cur_bri;
            let target = utils::round_down(current + val);
            murph(sender, current, context, target, config, windows, monitors);
        }
        Bri::Dec(val) => {
            let current = context.cur_bri;
            let target = utils::round_down(current - val);
            murph(sender, current, context, target, config, windows, monitors);
        }
        Bri::Close => {
            for monitor in monitors {
                windows[monitor].set_visible(false);
            }
        }
        Bri::Open => {
            for monitor in monitors {
                windows[monitor].set_visible(true);
            }
        }
        Bri::OpenTimed(time) => {
            for monitor in monitors.iter() {
                windows[*monitor].set_visible(true);
                let map_ref = &mut context.bri_tasks_window;
                if let Some(handle) = map_ref[*monitor].get(&VolBriTaskTypeWindow::AwaitClose) {
                    handle.abort();
                    map_ref[*monitor].remove(&VolBriTaskTypeWindow::AwaitClose);
                }

                let sender_clone = sender.clone();
                let monitors_clone = monitors.clone();

                let handle = tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs_f64(time)).await;

                    if let Err(e) = sender_clone.send(DaemonEvt {
                        evt: DaemonCmdType::Bri(Bri::Close),
                        sender: None,
                        uuid: None,
                        monitors: monitors_clone,
                    }) {
                        println!("Err closing the openned window: {}", e);
                    }
                });

                map_ref[*monitor].insert(VolBriTaskTypeWindow::AwaitClose, handle);
            }
        }
    }

    Ok(DaemonRes::Success)
}

fn get_bri(cmd: &BriCmdProvider) -> f64 {
    match cmd {
        BriCmdProvider::Builtin => {
            let brightness = backlight::Brightness::default();
            brightness.get_percent().unwrap_or_else(|e| {
                println!("Error trying to get the current brightness: {}", e);
                0
            }) as f64
        }
        BriCmdProvider::BrightnessCtl => {
            let output = if let Ok(child) = Command::new("/bin/sh")
                .arg("-c")
                .arg("brightnessctl")
                .output()
            {
                if let Ok(val) = String::from_utf8(child.stdout) {
                    val
                } else {
                    return 0.0;
                }
            } else {
                return 0.0f64;
            };

            let max = if let Some(l) = output.split(" ").last() {
                if let Ok(v) = l.trim().parse::<u64>() {
                    v
                } else {
                    return 0.0;
                }
            } else {
                return 0.0;
            };

            let cur = if let Ok(child) = Command::new("/bin/sh")
                .arg("-c")
                .arg("brightnessctl get")
                .output()
            {
                if let Ok(val) = String::from_utf8(child.stdout) {
                    if let Ok(v) = val.trim().parse::<u64>() {
                        v
                    } else {
                        return 0.0;
                    }
                } else {
                    return 0.0;
                }
            } else {
                return 0.0f64;
            };

            (cur * 100 / max) as f64
        }
        BriCmdProvider::NoCmd => 0f64,
    }
}

fn set_bri(cmd: &BriCmdProvider, val: f64) {
    match cmd {
        BriCmdProvider::Builtin => {
            let brightness = backlight::Brightness::default();
            brightness.set_percent(val as i32).unwrap_or_else(|e| {
                println!("Error trying to set the current brightness: {}", e);
                false
            });
        }

        BriCmdProvider::BrightnessCtl => {
            let _ = Command::new("/bin/sh")
                .arg("-c")
                .arg(format!("brightnessctl set {}%", val))
                .output();
        }

        BriCmdProvider::NoCmd => {}
    }
}

fn set_icon(config: Arc<AppConf>, icon: IconRefHolder, cur_bri: f64) {
    for icon_descriptor in config.bri.icons.iter() {
        if cur_bri >= icon_descriptor.lower && cur_bri < icon_descriptor.upper {
            match icon {
                IconRefHolder::Text(label) => label.set_text(&icon_descriptor.icon),
                IconRefHolder::Svg(pic) => {
                    if let Err(e) = utils::set_svg(pic, &icon_descriptor.icon) {
                        println!("Vol: Failed to set regular icon due to SVG error: {}", e);
                    }
                }
            }
            return;
        }
    }
}

enum IconRefHolder<'a> {
    Svg(&'a Image),
    Text(&'a Label),
}

pub fn create_bri_osd(
    backend: DisplayBackend,
    app: &Application,
    config: Arc<AppConf>,
    monitor: &gtk4::gdk::Monitor,
) -> ApplicationWindow {
    let result = window::create_window(
        &backend,
        app,
        &config.bri.window,
        gtk4_layer_shell::KeyboardMode::None,
        monitor,
    );
    result.add_css_class("bri-window");

    let cur_bri = get_bri(&config.bri.run_cmd);

    let adjustment = Adjustment::new(cur_bri, 0.0, 100f64, 0.1, 0.0, 0.0);

    let wrapper: Box = Box::new(gtk4::Orientation::Horizontal, 10);
    wrapper.set_halign(gtk4::Align::Center);
    wrapper.add_css_class("bri-box");

    let text_icon = Label::new(Some(""));
    text_icon.add_css_class("bri-icon");

    let svg_icon = Image::new();
    svg_icon.add_css_class("bri-icon");

    if config.bri.use_svg {
        set_icon(config.clone(), IconRefHolder::Svg(&svg_icon), cur_bri);
    } else {
        set_icon(config.clone(), IconRefHolder::Text(&text_icon), cur_bri);
    }

    let label = Label::new(Some(&(cur_bri as i64).to_string()));
    label.add_css_class("bri-label");

    if config.vol.use_svg {
        wrapper.append(&svg_icon);
    } else {
        wrapper.append(&text_icon);
    }

    let scale = Scale::new(gtk4::Orientation::Horizontal, Some(&adjustment));
    scale.set_width_request(100);
    scale.add_css_class("bri-scale");
    scale.set_sensitive(false);
    wrapper.append(&scale);
    wrapper.append(&label);

    result.set_child(Some(wrapper).as_ref());
    register_widget(super::app::Widget::Brightness, result.id());

    result.present();

    if !config.bri.window.visible_on_start {
        result.set_visible(false);
    }

    result
}
