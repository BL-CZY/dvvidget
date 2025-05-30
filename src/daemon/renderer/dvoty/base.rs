use crate::daemon::renderer::app::{register_widget, AppContext};
use crate::daemon::renderer::config::AppConf;
use crate::daemon::structs::{DaemonCmdType, DaemonEvt, Dvoty};
use crate::utils::DisplayBackend;
use gtk4::gdk::ModifierType;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Entry, ListBox, ListBoxRow, ScrolledWindow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use super::entry::DvotyUIEntry;
use super::utils::create_list_of;

#[derive(PartialEq, Eq, Hash)]
pub enum DvotyTaskType {
    ProcessInput,
    MurphViewport,
}

#[derive(Default)]
pub struct DvotyContext {
    pub dvoty_tasks: Vec<HashMap<DvotyTaskType, JoinHandle<()>>>,
    pub dvoty_list: Vec<Option<ListBox>>,
    pub dvoty_scroll: Vec<Option<ScrolledWindow>>,
    pub dvoty_entries: Vec<Vec<(DvotyUIEntry, ListBoxRow)>>,
    pub cur_ind: Vec<usize>,
    pub target_scroll: Vec<f64>,
    pub should_autofill: Vec<bool>,
}

impl DvotyContext {
    pub fn from_config(_config: &Arc<AppConf>, monitor_count: usize) -> Self {
        DvotyContext {
            dvoty_tasks: create_list_of(monitor_count),
            dvoty_list: create_list_of(monitor_count),
            dvoty_scroll: create_list_of(monitor_count),
            dvoty_entries: create_list_of(monitor_count),
            cur_ind: create_list_of(monitor_count),
            target_scroll: create_list_of(monitor_count),
            should_autofill: vec![true; monitor_count],
        }
    }
}

fn input(
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
    context: Rc<RefCell<AppContext>>,
) -> Entry {
    let input = Entry::builder().css_classes(["dvoty-input"]).build();

    let key_controller = gtk4::EventControllerKey::new();
    let sender_clone = sender.clone();

    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);

    key_controller.connect_key_pressed(
        move |_controller, keyval, _keycode, state: ModifierType| match keyval {
            gtk4::gdk::Key::Tab => glib::Propagation::Stop,
            gtk4::gdk::Key::Up => {
                super::event::send_dec(sender_clone.clone(), vec![monitor]);

                if state.contains(ModifierType::SHIFT_MASK) {
                    super::event::send_dec(sender_clone.clone(), vec![monitor]);
                    super::event::send_dec(sender_clone.clone(), vec![monitor]);
                    super::event::send_dec(sender_clone.clone(), vec![monitor]);
                    super::event::send_dec(sender_clone.clone(), vec![monitor]);
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Down => {
                super::event::send_inc(sender_clone.clone(), vec![monitor]);

                if state.contains(ModifierType::SHIFT_MASK) {
                    super::event::send_inc(sender_clone.clone(), vec![monitor]);
                    super::event::send_inc(sender_clone.clone(), vec![monitor]);
                    super::event::send_inc(sender_clone.clone(), vec![monitor]);
                    super::event::send_inc(sender_clone.clone(), vec![monitor]);
                }
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Page_Down => {
                sender_clone
                    .send(DaemonEvt {
                        evt: DaemonCmdType::Dvoty(Dvoty::ScrollEnd),
                        sender: None,
                        uuid: None,
                        monitors: vec![monitor],
                    })
                    .unwrap();
                glib::Propagation::Stop
            }
            gtk4::gdk::Key::Page_Up => {
                sender_clone
                    .send(DaemonEvt {
                        evt: DaemonCmdType::Dvoty(Dvoty::ScrollStart),
                        sender: None,
                        uuid: None,
                        monitors: vec![monitor],
                    })
                    .unwrap();

                glib::Propagation::Stop
            }

            gtk4::gdk::Key::Escape => {
                sender_clone
                    .send(DaemonEvt {
                        evt: DaemonCmdType::Dvoty(Dvoty::Close),
                        sender: None,
                        uuid: None,
                        monitors: vec![monitor],
                    })
                    .unwrap_or_else(|e| println!("Dvoty: Failed to send triggering event: {}", e));
                glib::Propagation::Stop
            }

            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                let mut context_ref = context.borrow_mut();
                context_ref.dvoty.should_autofill[monitor] = false;

                sender_clone
                    .send(DaemonEvt {
                        evt: DaemonCmdType::Dvoty(Dvoty::TriggerEntry),
                        sender: None,
                        uuid: None,
                        monitors: vec![monitor],
                    })
                    .unwrap_or_else(|e| println!("Dvoty: Failed to send triggering event: {}", e));

                glib::Propagation::Proceed
            }

            gtk4::gdk::Key::BackSpace | gtk4::gdk::Key::Delete => {
                let mut context_ref = context.borrow_mut();
                context_ref.dvoty.should_autofill[monitor] = false;
                glib::Propagation::Proceed
            }

            gtk4::gdk::Key::Left | gtk4::gdk::Key::Right => glib::Propagation::Proceed,

            _ => {
                let mut context_ref = context.borrow_mut();
                context_ref.dvoty.should_autofill[monitor] = true;
                glib::Propagation::Proceed
            }
        },
    );

    input.add_controller(key_controller);

    input.connect_changed(move |entry| {
        let recent_manager = gtk4::RecentManager::default();
        let recent_items = recent_manager.items();

        let recent_file_paths: Vec<PathBuf> = recent_items
            .iter()
            .filter_map(|item| {
                let uri = item.uri();
                gio::File::for_uri(&uri).path()
            })
            .collect();

        let content: String = entry.text().into();
        if let Err(e) = sender.send(DaemonEvt {
            evt: DaemonCmdType::Dvoty(Dvoty::Update(content, recent_file_paths)),
            sender: None,
            uuid: None,
            monitors: vec![monitor],
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
    monitor_ind: usize,
    monitor: &gtk4::gdk::Monitor,
    context: Rc<RefCell<AppContext>>,
) -> ApplicationWindow {
    let result = crate::daemon::renderer::window::create_window(
        &backend,
        app,
        &config.dvoty.window,
        gtk4_layer_shell::KeyboardMode::OnDemand,
        monitor,
    );
    result.add_css_class("dvoty-window");

    let input = input(sender.clone(), monitor_ind, context);
    let outer_wrapper = list(config.clone());

    let wrapper = Box::builder()
        .spacing(config.dvoty.spacing as i32)
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
        evt: DaemonCmdType::Dvoty(Dvoty::Update("".into(), vec![])),
        sender: None,
        uuid: None,
        monitors: vec![monitor_ind],
    }) {
        println!("Can't send message from Dvoty: {}", e);
    };

    result
}
