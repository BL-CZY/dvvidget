use std::cell::RefCell;
use std::collections::HashMap;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::daemon::structs::{Bri, DaemonEvt, DaemonRes};
use crate::utils::{self, DisplayBackend};
use crate::{daemon::structs::DaemonCmd, utils::DaemonErr};

use super::app::{AppContext, VolBriTaskType};
use super::config::{AppConf, BriCmdProvider};
use super::{app::register_widget, window};
use gtk4::{
    prelude::*, Adjustment, Application, ApplicationWindow, Box, Image, Label, Scale, Window,
};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

pub struct BriContext {
    pub cur_bri: f64,
    pub bri_tasks: HashMap<VolBriTaskType, JoinHandle<()>>,
}

impl BriContext {
    pub fn from_config(config: &Arc<AppConf>) -> Self {
        let cur_bri = get_bri(&config.bri.run_cmd);
        BriContext {
            cur_bri,
            bri_tasks: HashMap::new(),
        }
    }
}

fn update_display_info(config: Arc<AppConf>, window: &Window, val: f64) {
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

fn murph(
    sender: UnboundedSender<DaemonEvt>,
    mut current: f64,
    context: Rc<RefCell<AppContext>>,
    target: f64,
    config: Arc<AppConf>,
    window: &Window,
    monitor: usize,
) {
    let context_ref = &mut context.borrow_mut();
    // shadowing target to adjust it to an appropriate value
    let target = context_ref.set_virtual_brightness(target);
    let task_map = &mut context_ref.bri.bri_tasks;
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
                    evt: DaemonCmd::Bri(Bri::SetRough(current)),
                    sender: None,
                    uuid: None,
                    monitor,
                })
                .unwrap_or_else(|e| println!("Bri: failed to update: {}", e));
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        sender
            .send(DaemonEvt {
                evt: DaemonCmd::Bri(Bri::SetRough(target)),
                sender: None,
                uuid: None,
                monitor,
            })
            .unwrap_or_else(|e| println!("Bri: failed to update: {}", e));
    });

    task_map.insert(VolBriTaskType::MurphValue, handle);
}

fn set_rough(val: f64, window: &Window) {
    let child = if let Some(widget) = window
        .child()
        .and_downcast_ref::<Box>()
        .unwrap()
        .first_child()
    {
        widget
    } else {
        println!("Bri: Failed to downcast the box");
        return;
    };

    if let Some(scale) = child.downcast_ref::<Scale>() {
        scale.set_value(val);
        return;
    }

    while let Some(widget) = child.next_sibling() {
        if let Some(scale) = widget.downcast_ref::<Scale>() {
            scale.set_value(val);
            return;
        }
    }

    println!("Bri: Couldn't find the scale, ignoring...");
}

pub fn handle_bri_cmd(
    cmd: Bri,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
    context: Rc<RefCell<AppContext>>,
    config: Arc<AppConf>,
    monitor: usize,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Bri::SetRough(val) => {
            set_rough(val, window);
        }
        Bri::Set(val) => {
            let current = context.borrow_mut().bri.cur_bri;
            let target = utils::round_down(val);
            murph(sender, current, context, target, config, window, monitor);
        }
        Bri::Get => {
            return Ok(DaemonRes::GetBri(context.borrow_mut().bri.cur_bri));
        }
        Bri::Inc(val) => {
            let current = context.borrow_mut().bri.cur_bri;
            let target = utils::round_down(current + val);
            murph(sender, current, context, target, config, window, monitor);
        }
        Bri::Dec(val) => {
            let current = context.borrow_mut().bri.cur_bri;
            let target = utils::round_down(current - val);
            murph(sender, current, context, target, config, window, monitor);
        }
        Bri::Close => {
            window.set_visible(false);
        }
        Bri::Open => {
            window.set_visible(true);
        }
        Bri::OpenTimed(time) => {
            window.set_visible(true);
            let map_ref = &mut context.borrow_mut().bri.bri_tasks;
            if let Some(handle) = map_ref.get(&VolBriTaskType::AwaitClose) {
                handle.abort();
                map_ref.remove(&VolBriTaskType::AwaitClose);
            }

            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs_f64(time)).await;

                if let Err(e) = sender.send(DaemonEvt {
                    evt: DaemonCmd::Bri(Bri::Close),
                    sender: None,
                    uuid: None,
                    monitor,
                }) {
                    println!("Err closing the openned window: {}", e);
                }
            });

            map_ref.insert(VolBriTaskType::AwaitClose, handle);
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
) -> ApplicationWindow {
    let result = window::create_window(
        &backend,
        app,
        &config.bri.window,
        gtk4_layer_shell::KeyboardMode::None,
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
