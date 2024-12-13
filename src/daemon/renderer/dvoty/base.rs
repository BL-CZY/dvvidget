use crate::daemon::renderer::app::register_widget;
use crate::daemon::renderer::config::AppConf;
use crate::daemon::structs::{DaemonCmd, DaemonEvt, Dvoty};
use crate::utils::DisplayBackend;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Entry, ListBox, ListBoxRow, ScrolledWindow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use super::entry::DvotyUIEntry;

#[derive(PartialEq, Eq, Hash)]
pub enum DvotyTaskType {
    ProcessInput,
    MurphViewport,
}

#[derive(Default)]
pub struct DvotyContext {
    pub dvoty_tasks: HashMap<DvotyTaskType, JoinHandle<()>>,
    pub dvoty_list: Option<ListBox>,
    pub dvoty_scroll: Option<ScrolledWindow>,
    pub dvoty_entries: Vec<(DvotyUIEntry, ListBoxRow)>,
    pub cur_ind: usize,
    pub target_scroll: f64,
}

fn input(sender: UnboundedSender<DaemonEvt>) -> Entry {
    let input = Entry::builder().css_classes(["dvoty-input"]).build();

    let key_controller = gtk4::EventControllerKey::new();
    let sender_clone = sender.clone();
    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| match keyval {
        gtk4::gdk::Key::Tab => glib::Propagation::Stop,
        gtk4::gdk::Key::Up => {
            super::event::send_dec(sender_clone.clone());
            glib::Propagation::Stop
        }
        gtk4::gdk::Key::Down => {
            super::event::send_inc(sender_clone.clone());
            glib::Propagation::Stop
        }
        gtk4::gdk::Key::Escape => {
            sender_clone
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::Close),
                    sender: None,
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send triggering event: {}", e));
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    });

    let sender_clone = sender.clone();
    key_controller.connect_key_released(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Return || keyval == gtk4::gdk::Key::KP_Enter {
            sender_clone
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::TriggerEntry),
                    sender: None,
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send triggering event: {}", e));
        }
    });

    input.add_controller(key_controller);

    input.connect_changed(move |entry| {
        let content: String = entry.text().into();
        if let Err(e) = sender.send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::Update(content)),
            sender: None,
        }) {
            println!("Can't send message from Dvoty: {}", e);
        };
    });

    input
}

fn list(config: Arc<AppConf>) -> gtk4::Box {
    let list_box = ListBox::builder()
        .css_classes(["dvoty-list"])
        .focusable(false)
        .build();
    let list_wrapper = ScrolledWindow::builder()
        .kinetic_scrolling(false)
        .overlay_scrolling(false)
        .min_content_height(config.dvoty.max_height as i32)
        .child(&list_box)
        .hexpand(true)
        .build();
    let outer_wrapper = Box::builder().css_classes(["dvoty-scroll"]).build();
    outer_wrapper.append(&list_wrapper);

    outer_wrapper
}

pub fn create_dvoty(
    backend: DisplayBackend,
    app: &Application,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
) -> ApplicationWindow {
    let result = crate::daemon::renderer::window::create_window(
        &backend,
        app,
        &config.dvoty.window,
        gtk4_layer_shell::KeyboardMode::OnDemand,
    );
    result.add_css_class("dvoty-window");

    let input = input(sender.clone());
    let outer_wrapper = list(config.clone());

    let wrapper = Box::builder()
        .spacing(20)
        .css_classes(["dvoty-wrapper"])
        .orientation(gtk4::Orientation::Vertical)
        .build();
    wrapper.append(&input);
    wrapper.append(&outer_wrapper);

    result.set_child(Some(&wrapper));
    register_widget(crate::daemon::renderer::app::Widget::Dvoty, result.id());

    input.grab_focus();

    result.present();

    if !config.dvoty.window.visible_on_start {
        result.set_visible(false);
    }

    // update the list after creation
    if let Err(e) = sender.send(DaemonEvt {
        evt: DaemonCmd::Dvoty(Dvoty::Update("".into())),
        sender: None,
    }) {
        println!("Can't send message from Dvoty: {}", e);
    };

    result
}
