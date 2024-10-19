use crate::daemon::renderer::app::{register_widget, AppContext};
use crate::daemon::renderer::config::AppConf;
use crate::daemon::structs::{DaemonCmd, DaemonEvt, DaemonRes, Dvoty};
use crate::utils::{DaemonErr, DisplayBackend};
use gtk4::{prelude::*, Viewport};
use gtk4::{
    Application, ApplicationWindow, Box, Entry, Label, ListBox, ListBoxRow, ScrolledWindow, Window,
};
use serde::{Deserialize, Serialize};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use super::{math, search, url};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DvotyEntry {
    Empty,
    Instruction,
    Math {
        expression: String,
        result: String,
    },
    Launch {
        name: String,
        exec: String,
        icon: String,
    },
    Command {
        exec: String,
    },
    Search {
        keyword: String,
    },
    Url {
        url: String,
    },
}

#[derive(Clone)]
pub enum DvotyUIEntry {
    Instruction,
    Math { result: String },
    Launch { exec: String },
    Command { exec: String },
    Search { keyword: String },
    Url { url: String },
}

impl DvotyUIEntry {
    pub fn run(self) {
        match self {
            DvotyUIEntry::Math { result } => {
                math::set_clipboard_text(&result);
            }
            DvotyUIEntry::Search { keyword } => {
                search::spawn_keyword(keyword);
            }
            DvotyUIEntry::Url { url } => {
                url::spawn_url(url);
            }
            _ => {}
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum DvotyTaskType {
    ProcessInput,
}

#[derive(Default)]
pub struct DvotyContext {
    pub dvoty_tasks: HashMap<DvotyTaskType, JoinHandle<()>>,
    pub dvoty_list: Option<ListBox>,
    pub dvoty_scroll: Option<ScrolledWindow>,
    pub dvoty_entries: Vec<(DvotyUIEntry, ListBoxRow)>,
    pub cur_ind: usize,
}

fn get_list(window: &Window) -> Result<ListBox, ()> {
    if let Some(outer_box) = window.child().and_downcast_ref::<Box>() {
        if let Some(inner_box) = outer_box.last_child() {
            if let Some(scroll) = inner_box.first_child() {
                if let Some(scroll_inner) = scroll.first_child() {
                    if let Some(result) = scroll_inner.first_child().and_downcast::<ListBox>() {
                        return Ok(result);
                    }
                }
            }
        }
    }

    Err(())
}

fn get_scrolled_window(window: &Window) -> Result<ScrolledWindow, ()> {
    if let Some(outer_box) = window.child().and_downcast_ref::<Box>() {
        if let Some(inner_box) = outer_box.last_child() {
            if let Some(scroll) = inner_box.first_child().and_downcast::<ScrolledWindow>() {
                return Ok(scroll);
            }
        }
    }

    Err(())
}

fn process_input_str(input: &str, sender: UnboundedSender<DaemonEvt>) {
    if input.is_empty() {
        if let Err(e) = sender.send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Empty)),
            sender: None,
        }) {
            println!("Dvoty: Failed to send entry: {}, ignoring...", e);
        };
        return;
    }

    match input.chars().nth(0).unwrap() {
        '=' => {
            super::math::eval_math(input, sender);
        }
        '@' => {}
        '$' => {}
        ':' => {
            super::url::send_url(input.chars().skip(1).collect::<String>(), sender);
        }
        '/' => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Search {
                        keyword: input.chars().skip(1).collect::<String>(),
                    })),
                    sender: None,
                })
                .unwrap_or_else(|e| {
                    println!("Dvoty: Error adding search entry: {}", e);
                });
        }
        _ => {}
    }
}

fn process_input(
    input: String,
    context: Rc<RefCell<AppContext>>,
    sender: UnboundedSender<DaemonEvt>,
    window: &Window,
) -> Result<(), DaemonErr> {
    let context_ref = &mut context.borrow_mut();
    context_ref.dvoty.dvoty_entries.clear();
    context_ref.dvoty.cur_ind = 0;

    let list = if let Some(l) = &context_ref.dvoty.dvoty_list {
        l
    } else {
        if let Ok(res) = get_list(window) {
            context_ref.dvoty.dvoty_list = Some(res);
            &context_ref.dvoty.dvoty_list.as_ref().unwrap()
        } else {
            println!("Dvoty: can't find list");
            return Err(DaemonErr::CannotFindWidget);
        }
    };

    list.remove_all();

    let task_map = &mut context_ref.dvoty.dvoty_tasks;

    if let Some(handle) = task_map.get(&DvotyTaskType::ProcessInput) {
        handle.abort();
        task_map.remove(&DvotyTaskType::ProcessInput);
    }

    let handle = tokio::spawn(async move {
        process_input_str(&input, sender.clone());
    });

    task_map.insert(DvotyTaskType::ProcessInput, handle);

    Ok(())
}

pub fn create_base_entry(config: Arc<AppConf>, icon: &str, content: &str, tip: &str) -> ListBoxRow {
    let label_begin = Label::builder()
        .use_markup(true)
        .label(format!(
            "<span show=\"ignorables\" background=\"{}\" foreground=\"{}\" size=\"x-large\"> {} </span> {}",
            config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color, icon, content
        ))
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let label_end = Label::builder()
        .use_markup(true)
        .label(tip)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::End)
        .hexpand(true)
        .build();

    let wrapper_box = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["dvoty-box"])
        .build();

    wrapper_box.append(&label_begin);
    wrapper_box.append(&label_end);

    let res = ListBoxRow::builder()
        .css_classes(["dvoty-entry"])
        .child(&wrapper_box)
        .build();

    return res;
}

fn add_entry(
    entry: DvotyEntry,
    window: &Window,
    context: Rc<RefCell<AppContext>>,
    config: Arc<AppConf>,
) -> Result<DaemonRes, DaemonErr> {
    let context_ref = &mut context.borrow_mut();

    let list = if let Some(l) = &context_ref.dvoty.dvoty_list {
        l.clone()
    } else {
        if let Ok(res) = get_list(window) {
            context_ref.dvoty.dvoty_list = Some(res.clone());
            res
        } else {
            println!("Dvoty: can't find list");
            return Err(DaemonErr::CannotFindWidget);
        }
    };

    match entry {
        DvotyEntry::Empty => {
            super::instruction::populate_instructions(&list, config, context_ref);
        }
        DvotyEntry::Math { result, .. } => {
            super::math::populate_math_entry(config, &list, result, context_ref);
        }
        DvotyEntry::Search { keyword } => {
            super::search::populate_search_entry(config, &list, keyword, context_ref);
        }
        DvotyEntry::Url { url } => {
            super::url::populate_url_entry(config, &list, url, context_ref);
        }
        _ => {}
    }

    Ok(DaemonRes::Success)
}

fn set_class(target: &ListBoxRow, remove_class: &[&str], add_class: &[&str]) {
    for class in remove_class.iter() {
        target.remove_css_class(class);
    }

    for class in add_class.iter() {
        target.add_css_class(class);
    }
}

pub fn adjust_class(old: usize, new: usize, input: &mut Vec<(DvotyUIEntry, ListBoxRow)>) {
    if old >= input.len() || new >= input.len() {
        return;
    }

    match input[old].0 {
        DvotyUIEntry::Instruction => {
            set_class(
                &input[old].1,
                &["dvoty-entry-instruction-select", "dvoty-entry-select"],
                &["dvoty-entry-instruction", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Math { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-math-select", "dvoty-entry-select"],
                &["dvoty-entry-math", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Search { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-search-select", "dvoty-entry-select"],
                &["dvoty-entry-search", "dvoty-entry"],
            );
        }
        DvotyUIEntry::Url { .. } => {
            set_class(
                &input[old].1,
                &["dvoty-entry-url-select", "dvoty-entry-select"],
                &["dvoty-entry-url", "dvoty-entry"],
            );
        }
        _ => {}
    }

    match input[new].0 {
        DvotyUIEntry::Instruction => {
            set_class(
                &input[new].1,
                &["dvoty-entry-instruction", "dvoty-entry"],
                &["dvoty-entry-instruction-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Math { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-math", "dvoty-entry"],
                &["dvoty-entry-math-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Search { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-search", "dvoty-entry"],
                &["dvoty-entry-search-select", "dvoty-entry-select"],
            );
        }
        DvotyUIEntry::Url { .. } => {
            set_class(
                &input[new].1,
                &["dvoty-entry-url", "dvoty-entry"],
                &["dvoty-entry-url-select", "dvoty-entry-select"],
            );
        }

        _ => {}
    }
}

fn adjust_row(
    row: ListBoxRow,
    viewport: Viewport,
    scroll: ScrolledWindow,
) -> Result<(), DaemonErr> {
    let viewport_bound = if let Some(bound) = viewport.compute_bounds(&viewport) {
        bound
    } else {
        return Err(DaemonErr::CannotFindWidget);
    };

    let row_bound = if let Some(bound) = row.compute_bounds(&viewport) {
        bound
    } else {
        return Err(DaemonErr::CannotFindWidget);
    };

    let adjustment = scroll.vadjustment();

    if row_bound.y() < viewport_bound.y() {
        // if the top of the row is not in the viewport, reduce the adjustment value by the
        // difference
        adjustment.set_value(adjustment.value() - (viewport_bound.y() - row_bound.y()) as f64);
    } else if row_bound.y() + row_bound.height() > viewport_bound.y() + viewport_bound.height() {
        // if the bottom of the row is not in the viewport, increase the adjustment value by the
        // difference
        adjustment.set_value(
            adjustment.value()
                + (row_bound.y() + row_bound.height()
                    - viewport_bound.y()
                    - viewport_bound.height()) as f64,
        );
    }

    Ok(())
}

fn ensure_row_in_viewport(
    context_ref: &mut RefMut<AppContext>,
    window: &Window,
) -> Result<(), DaemonErr> {
    let scroll = if let Some(s) = &context_ref.dvoty.dvoty_scroll {
        s
    } else {
        if let Ok(res) = get_scrolled_window(window) {
            context_ref.dvoty.dvoty_scroll = Some(res);
            &context_ref.dvoty.dvoty_scroll.as_ref().unwrap()
        } else {
            println!("Dvoty: can't find scrolled window");
            return Err(DaemonErr::CannotFindWidget);
        }
    };

    let viewport = if let Some(v) = scroll.first_child().and_downcast::<gtk4::Viewport>() {
        v
    } else {
        println!("Dvoty: can't find viewport");

        return Err(DaemonErr::CannotFindWidget);
    };

    let row = context_ref.dvoty.dvoty_entries[context_ref.dvoty.cur_ind]
        .1
        .clone();

    adjust_row(row, viewport, scroll.clone())?;

    Ok(())
}

pub fn handle_dvoty_cmd(
    cmd: Dvoty,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
    app_context: Rc<RefCell<AppContext>>,
    config: Arc<AppConf>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Dvoty::Update(str) => {
            process_input(str, app_context, sender.clone(), window)?;
        }

        Dvoty::AddEntry(entry) => {
            add_entry(entry, window, app_context, config)?;
        }

        Dvoty::IncEntryIndex => {
            let mut context_ref = app_context.borrow_mut();

            if !context_ref.dvoty.dvoty_entries.is_empty() {
                let old = context_ref.dvoty.cur_ind;
                let max = context_ref.dvoty.dvoty_entries.len() - 1;
                context_ref.dvoty.cur_ind += 1;
                if context_ref.dvoty.cur_ind > max {
                    context_ref.dvoty.cur_ind = 0;
                }
                let new = context_ref.dvoty.cur_ind;
                adjust_class(old, new, &mut context_ref.dvoty.dvoty_entries.clone());
                ensure_row_in_viewport(&mut context_ref, window)?;
            }
        }

        Dvoty::DecEntryIndex => {
            let mut context_ref = app_context.borrow_mut();

            if !context_ref.dvoty.dvoty_entries.is_empty() {
                let old = context_ref.dvoty.cur_ind;
                let max = context_ref.dvoty.dvoty_entries.len() - 1;
                if context_ref.dvoty.cur_ind == 0 {
                    context_ref.dvoty.cur_ind = max;
                } else {
                    context_ref.dvoty.cur_ind -= 1;
                }
                let new = context_ref.dvoty.cur_ind;
                adjust_class(old, new, &mut context_ref.dvoty.dvoty_entries.clone());
                ensure_row_in_viewport(&mut context_ref, window)?;
            }
        }

        Dvoty::TriggerEntry => {
            let context_ref = app_context.borrow();
            if !context_ref.dvoty.dvoty_entries.is_empty() {
                context_ref.dvoty.dvoty_entries[context_ref.dvoty.cur_ind]
                    .0
                    .clone()
                    .run();
            }
            window.set_visible(false);
        }

        Dvoty::Open => {
            window.set_visible(true);
        }

        Dvoty::Close => {
            window.set_visible(false);
        }
    }

    Ok(DaemonRes::Success)
}

fn send_inc(sender: UnboundedSender<DaemonEvt>) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::IncEntryIndex),
            sender: None,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send inc index: {}", e));
}

fn send_dec(sender: UnboundedSender<DaemonEvt>) {
    sender
        .send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::DecEntryIndex),
            sender: None,
        })
        .unwrap_or_else(|e| println!("Dvoty: Failed to send dec index: {}", e));
}

fn input(sender: UnboundedSender<DaemonEvt>) -> Entry {
    let input = Entry::builder().css_classes(["dvoty-input"]).build();

    let key_controller = gtk4::EventControllerKey::new();
    let sender_clone = sender.clone();
    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| match keyval {
        gtk4::gdk::Key::Tab => glib::Propagation::Stop,
        gtk4::gdk::Key::Up => {
            send_dec(sender_clone.clone());
            glib::Propagation::Stop
        }
        gtk4::gdk::Key::Down => {
            send_inc(sender_clone.clone());
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
        .hscrollbar_policy(gtk4::PolicyType::Never)
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
    let result =
        crate::daemon::renderer::window::create_window(&backend, app, &config.dvoty.window);
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
