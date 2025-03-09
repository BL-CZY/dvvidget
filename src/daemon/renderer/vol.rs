use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::daemon::structs::{DaemonEvt, DaemonRes, MonitorClient, Vol};
use crate::utils::{self, DisplayBackend};
use crate::{daemon::structs::DaemonCmdType, utils::DaemonErr};

use super::app::{VolBriTaskType, VolBriTaskTypeWindow};
use super::config::{AppConf, VolCmdProvider};
use super::{app::register_widget, window};
use gtk4::{
    prelude::*, Adjustment, Application, ApplicationWindow, Box, Image, Label, Scale, Window,
};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

pub struct VolContext {
    pub cur_vol: f64,
    pub max_vol: f64,
    pub is_muted: bool,
    pub vol_tasks_window: Vec<HashMap<VolBriTaskTypeWindow, JoinHandle<()>>>,
    pub vol_tasks: HashMap<VolBriTaskType, JoinHandle<()>>,
}

impl VolContext {
    pub fn from_config(config: &Arc<AppConf>, monitor_count: usize) -> Self {
        let (cur_vol, is_muted) = get_volume(&config.vol.run_cmd);
        VolContext {
            cur_vol,
            max_vol: config.vol.max_vol,
            is_muted,
            vol_tasks_window: {
                let mut res = vec![];
                for _ in 0..monitor_count {
                    res.push(HashMap::new());
                }
                res
            },
            vol_tasks: HashMap::new(),
        }
    }

    pub fn set_virtual_volume(&mut self, val: f64) -> f64 {
        if val > self.max_vol {
            self.cur_vol = self.max_vol;
        } else if val < 0f64 {
            self.cur_vol = 0f64;
        } else {
            self.cur_vol = val;
        }

        self.cur_vol
    }
}

fn update_display_info(config: Arc<AppConf>, window: &[Window], val: f64, is_muted: bool) {
    for window in window.iter() {
        let child = if let Some(w) = window.child() {
            w
        } else {
            println!("Vol: can't find the box");
            return;
        };

        if let Some(widget) = child.first_child() {
            if let Some(label) = widget.downcast_ref::<Label>() {
                set_icon(config.clone(), IconRefHolder::Text(label), val, is_muted);
            } else if let Some(pic) = widget.downcast_ref::<Image>() {
                set_icon(config.clone(), IconRefHolder::Svg(pic), val, is_muted);
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
    context: &mut VolContext,
    target: f64,
    config: Arc<AppConf>,
    window: &[Window],
    monitor: MonitorClient,
) {
    let is_mute = context.is_muted;
    // shadowing target to adjust it to an appropriate value
    let target = context.set_virtual_volume(target);
    let task_map = &mut context.vol_tasks_window;
    if let Some(handle) = task_map[monitor].get(&VolBriTaskTypeWindow::MurphValue) {
        handle.abort();
        task_map[monitor].remove(&VolBriTaskTypeWindow::MurphValue);
    }

    set_volume(&config.vol.run_cmd, target);
    update_display_info(config.clone(), window, target, is_mute);

    let handle = tokio::spawn(async move {
        for _ in 0..50 {
            current += (target - current) * 0.1f64;
            sender
                .send(DaemonEvt {
                    evt: DaemonCmdType::Vol(Vol::SetRough(current)),
                    sender: None,
                    uuid: None,
                    monitor,
                })
                .unwrap_or_else(|e| println!("Vol: failed to update: {}", e));
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        sender
            .send(DaemonEvt {
                evt: DaemonCmdType::Vol(Vol::SetRough(target)),
                sender: None,
                uuid: None,
                monitor,
            })
            .unwrap_or_else(|e| println!("Vol: failed to update: {}", e));
    });

    task_map[monitor].insert(VolBriTaskTypeWindow::MurphValue, handle);
}

fn set_rough(val: f64, windows: &[Window]) {
    for window in windows.iter() {
        let child = if let Some(widget) = window
            .child()
            .and_downcast_ref::<Box>()
            .unwrap()
            .first_child()
        {
            widget
        } else {
            println!("Vol: Failed to downcast the box");
            continue;
        };

        if let Some(scale) = child.downcast_ref::<Scale>() {
            scale.set_value(val);
            continue;
        }

        while let Some(widget) = child.next_sibling() {
            if let Some(scale) = widget.downcast_ref::<Scale>() {
                scale.set_value(val);
                continue;
            }
        }

        println!("Vol: Couldn't find the scale, ignoring...");
    }
}

fn handle_set_mute(context: &mut VolContext, config: Arc<AppConf>, val: bool, windows: &[Window]) {
    for window in windows.iter() {
        let child = if let Some(w) = window.child() {
            w
        } else {
            continue;
        };

        if let Some(label) = child.first_child().and_downcast_ref::<Label>() {
            context.is_muted = val;
            set_mute(&config.vol.run_cmd, val);
            let vol = get_volume(&config.vol.run_cmd).0;
            set_icon(config.clone(), IconRefHolder::Text(label), vol, val);
        } else if let Some(pic) = child.first_child().and_downcast_ref::<Image>() {
            context.is_muted = val;
            set_mute(&config.vol.run_cmd, val);
            let vol = get_volume(&config.vol.run_cmd).0;
            set_icon(config.clone(), IconRefHolder::Svg(pic), vol, val);
        }
    }
}

pub fn handle_vol_cmd(
    cmd: Vol,
    windows: &[Window],
    sender: UnboundedSender<DaemonEvt>,
    context: &mut VolContext,
    config: Arc<AppConf>,
    monitors: Vec<usize>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Vol::SetMute(val) => {
            handle_set_mute(context, config, val, windows);
        }
        Vol::ToggleMute => {
            let mute = !context.is_muted;
            handle_set_mute(context, config, mute, windows);
        }
        Vol::GetMute => {
            return Ok(DaemonRes::GetMute(context.is_muted));
        }
        Vol::SetRough(val) => {
            set_rough(val, windows);
        }
        Vol::Set(val) => {
            let current = context.cur_vol;
            let target = utils::round_down(val);
            murph(sender, current, context, target, config, windows, monitors);
        }
        Vol::Get => {
            return Ok(DaemonRes::GetVol(context.cur_vol));
        }
        Vol::Inc(val) => {
            let current = context.cur_vol;
            let target = utils::round_down(current + val);
            murph(sender, current, context, target, config, windows, monitors);
        }
        Vol::Dec(val) => {
            let current = context.cur_vol;
            let target = utils::round_down(current - val);
            murph(sender, current, context, target, config, windows, monitors);
        }
        Vol::Close => {
            windows[monitors].set_visible(false);
        }
        Vol::Open => {
            windows[monitors].set_visible(true);
        }
        Vol::OpenTimed(time) => {
            windows[monitors].set_visible(true);
            let map_ref = &mut context.vol_tasks_window;
            if let Some(handle) = map_ref[monitors].get(&VolBriTaskTypeWindow::AwaitClose) {
                handle.abort();
                map_ref[monitors].remove(&VolBriTaskTypeWindow::AwaitClose);
            }

            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs_f64(time)).await;

                if let Err(e) = sender.send(DaemonEvt {
                    evt: DaemonCmdType::Vol(Vol::Close),
                    sender: None,
                    uuid: None,
                    monitor: MonitorClient::One(monitors),
                }) {
                    println!("Err closing the openned window: {}", e);
                }
            });

            map_ref[monitors].insert(VolBriTaskTypeWindow::AwaitClose, handle);
        }
    }

    Ok(DaemonRes::Success)
}

// returns the current volume, if it's muted, return true, if it's not, return false
fn get_volume(cmd: &VolCmdProvider) -> (f64, bool) {
    match cmd {
        VolCmdProvider::Wpctl => {
            let output = if let Ok(out) = std::process::Command::new("wpctl")
                .arg("get-volume")
                .arg("@DEFAULT_AUDIO_SINK@")
                .output()
            {
                out
            } else {
                return (0f64, false);
            };

            let stdout = String::from_utf8_lossy(&output.stdout);
            let volume_str = stdout.split_whitespace().nth(1).unwrap_or_default();
            let mute_str = stdout.split_whitespace().nth(2).unwrap_or_default();
            (
                volume_str.parse::<f64>().unwrap_or_default() * 100f64,
                mute_str == "[MUTED]",
            )
        }

        VolCmdProvider::NoCmd => (0f64, false),
    }
}

fn set_volume(cmd: &VolCmdProvider, val: f64) {
    match cmd {
        VolCmdProvider::Wpctl => {
            if let Err(e) = std::process::Command::new("wpctl")
                .arg("set-volume")
                .arg("@DEFAULT_AUDIO_SINK@")
                .arg(format!("{}%", val))
                .output()
            {
                println!("Vol: Failed to set volume: {}", e);
            };
        }

        VolCmdProvider::NoCmd => {}
    }
}

fn set_mute(cmd: &VolCmdProvider, val: bool) {
    match cmd {
        VolCmdProvider::Wpctl => {
            if let Err(e) = std::process::Command::new("wpctl")
                .arg("set-mute")
                .arg("@DEFAULT_AUDIO_SINK@")
                .arg(format!("{}", val as i32))
                .output()
            {
                println!("Vol: Failed to set mute: {}", e);
            }
        }

        VolCmdProvider::NoCmd => {}
    }
}

fn set_icon(config: Arc<AppConf>, icon: IconRefHolder, cur_vol: f64, is_muted: bool) {
    if is_muted {
        match icon {
            IconRefHolder::Text(label) => label.set_text(&config.vol.mute_icon),
            IconRefHolder::Svg(pic) => {
                if let Err(e) = utils::set_svg(pic, &config.vol.mute_icon) {
                    println!("Vol: Failed to set icon for mute due to SVG error: {}", e);
                }
            }
        }
        return;
    }

    for icon_descriptor in config.vol.icons.iter() {
        if cur_vol >= icon_descriptor.lower && cur_vol < icon_descriptor.upper {
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

pub fn create_sound_osd(
    backend: DisplayBackend,
    app: &Application,
    config: Arc<AppConf>,
) -> ApplicationWindow {
    let result = window::create_window(
        &backend,
        app,
        &config.vol.window,
        gtk4_layer_shell::KeyboardMode::None,
    );
    result.add_css_class("sound-window");

    let (cur_vol, is_muted) = get_volume(&config.vol.run_cmd);

    let adjustment = Adjustment::new(cur_vol, 0.0, config.vol.max_vol, 0.1, 0.0, 0.0);

    let wrapper: Box = Box::new(gtk4::Orientation::Horizontal, 10);
    wrapper.set_halign(gtk4::Align::Center);
    wrapper.add_css_class("sound-box");

    let text_icon = Label::new(Some(""));
    text_icon.add_css_class("sound-icon");

    let svg_icon = Image::new();
    svg_icon.add_css_class("sound-icon");

    if config.vol.use_svg {
        set_icon(
            config.clone(),
            IconRefHolder::Svg(&svg_icon),
            cur_vol,
            is_muted,
        );
    } else {
        set_icon(
            config.clone(),
            IconRefHolder::Text(&text_icon),
            cur_vol,
            is_muted,
        );
    }

    let label = Label::new(Some(&(cur_vol as i64).to_string()));
    label.add_css_class("sound-label");

    if config.vol.use_svg {
        wrapper.append(&svg_icon);
    } else {
        wrapper.append(&text_icon);
    }

    let scale = Scale::new(gtk4::Orientation::Horizontal, Some(&adjustment));
    scale.set_width_request(100);
    scale.add_css_class("sound-scale");
    scale.set_sensitive(false);
    wrapper.append(&scale);
    wrapper.append(&label);

    result.set_child(Some(wrapper).as_ref());
    register_widget(super::app::Widget::Volume, result.id());

    result.present();

    if !config.vol.window.visible_on_start {
        result.set_visible(false);
    }

    result
}
