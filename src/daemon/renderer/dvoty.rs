use crate::daemon::structs::DaemonCmd;
use crate::daemon::structs::DaemonEvt;
use crate::daemon::structs::DaemonRes;
use crate::daemon::structs::Dvoty;
use crate::utils::DaemonErr;
use crate::utils::DisplayBackend;
use evalexpr::context_map;
use glib::object::CastNone;
use gtk4::prelude::ApplicationWindowExt;
use gtk4::prelude::BoxExt;
use gtk4::prelude::ButtonExt;
use gtk4::prelude::DisplayExt;
use gtk4::prelude::EditableExt;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use gtk4::Box;
use gtk4::Button;
use gtk4::Label;
use gtk4::Window;
use gtk4::{Application, ApplicationWindow, Entry, ListBox, ScrolledWindow};
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use super::app::register_widget;
use super::app::AppContext;
use super::{config::AppConf, window};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DvotyEntry {
    Empty,
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

#[derive(PartialEq, Eq, Hash)]
pub enum DvotyTaskType {
    ProcessInput,
}

#[derive(Default)]
pub struct DvotyContext {
    pub dvoty_tasks: HashMap<DvotyTaskType, JoinHandle<()>>,
    pub dvoty_list: Option<ListBox>,
}

fn get_list(window: &Window) -> Result<ListBox, ()> {
    if let Some(outer_list) = window.child().and_downcast_ref::<ListBox>() {
        if let Some(scroll_wrap) = outer_list.last_child() {
            if let Some(scroll) = scroll_wrap.first_child() {
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

fn set_clipboard_text(text: &str) {
    let display = gtk4::gdk::Display::default().expect("Could not get default display");
    let clipboard = display.clipboard();

    clipboard.set_text(text);
}

fn eval_math(input: &str, sender: UnboundedSender<DaemonEvt>) {
    use evalexpr::Value;
    let context = match context_map! {
        "avg" => Function::new(|argument| {
            let arguments = argument.as_tuple()?;

            if arguments.is_empty() {
                return Err(evalexpr::EvalexprError::CustomMessage("Average of empty set is undefined".to_string()));
            }

            let sum: f64 = arguments.iter()
                .map(|arg| arg.as_number())
                .collect::<Result<Vec<f64>, evalexpr::EvalexprError>>()?
                .iter()
                .sum();

            let avg = sum / arguments.len() as f64;

            Ok(Value::Float(avg))
        }),
        "sqrt" => Function::new(|argument| {
            let number = argument.as_number()?;
            if number < 0.0 {
                Err(evalexpr::EvalexprError::CustomMessage("Cannot calculate square root of a negative number".to_string()))
            } else {
                Ok(Value::Float(number.sqrt()))
            }
        })
    } {
        Ok(res) => res,
        Err(e) => {
            println!("Dvoty: Error creating math context: {}", e);
            return;
        }
    };

    let expr = input.chars().skip(1).collect::<String>();
    match evalexpr::eval_with_context(&expr, &context) {
        Ok(res) => {
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Math {
                        expression: expr,
                        result: res.to_string(),
                    })),
                    sender: None,
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send math result: {}", e));
        }
        Err(e) => {
            println!("Dvoty: Failed to evaluate math: {}", e);
            sender
                .send(DaemonEvt {
                    evt: DaemonCmd::Dvoty(Dvoty::AddEntry(DvotyEntry::Math {
                        expression: expr,
                        result: e.to_string(),
                    })),
                    sender: None,
                })
                .unwrap_or_else(|e| println!("Dvoty: Failed to send math result: {}", e));
        }
    }
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
            eval_math(input, sender);
        }
        '@' => {}
        '$' => {}
        ':' => {}
        '/' => {}
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

fn add_entry(
    entry: DvotyEntry,
    window: &Window,
    context: Rc<RefCell<AppContext>>,
    config: Arc<AppConf>,
) -> Result<DaemonRes, DaemonErr> {
    let context_ref = &mut context.borrow_mut();

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

    match entry {
        DvotyEntry::Empty => {
            populate_instructions(list, config);
        }
        DvotyEntry::Math { result, .. } => {
            let label_begin = Label::builder()
                .use_markup(true)
                .label(format!(
                    "<span show=\"ignorables\" background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> {}",
                    config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color, result
                ))
                .css_classes(["dvoty-label"])
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();

            let label_end = Label::builder()
                .use_markup(true)
                .label("Click to copy")
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

            let btn = Button::builder()
                .css_classes(["dvoty-entry"])
                .child(&wrapper_box)
                .build();

            btn.connect_clicked(move |_| {
                set_clipboard_text(&result);
            });

            list.append(&btn);
        }
        _ => {}
    }

    Ok(DaemonRes::Success)
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
    }

    Ok(DaemonRes::Success)
}

fn create_instruction(instruction: &str, icon: &str) -> Box {
    let label_start = Label::builder()
        .use_markup(true)
        .label(instruction)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let label_end = Label::builder()
        .use_markup(true)
        .label(icon)
        .css_classes(["dvoty-label"])
        .halign(gtk4::Align::End)
        .hexpand(true)
        .build();

    let result = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .css_classes(["dvoty-entry"])
        .build();

    result.append(&label_start);
    result.append(&label_end);

    result
}

fn populate_instructions(list_box: &ListBox, config: Arc<AppConf>) {
    let instructions = vec![
        (format!("input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> for math expressions", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> @ </span> for launching apps", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> $ </span> for running commands", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> / </span> for searching online", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> : </span> for opening url", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
    ];
    for instruction in instructions.iter() {
        list_box.append(&create_instruction(&instruction.0, &instruction.1));
    }
}

pub fn create_dvoty(
    backend: DisplayBackend,
    app: &Application,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
) -> ApplicationWindow {
    let result = window::create_window(&backend, app, &config.dvoty.window);
    result.add_css_class("dvoty-window");

    let list_box = ListBox::builder().css_classes(["dvoty-list"]).build();
    let input = Entry::builder().css_classes(["dvoty-input"]).build();

    populate_instructions(&list_box, config.clone());

    let sender_clone = sender.clone();
    input.connect_changed(move |entry| {
        let content: String = entry.text().into();
        if let Err(e) = sender_clone.send(DaemonEvt {
            evt: DaemonCmd::Dvoty(Dvoty::Update(content)),
            sender: None,
        }) {
            println!("Can't send message from Dvoty: {}", e);
        };
    });

    let list_wrapper = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_height(config.dvoty.max_height as i32)
        .child(&list_box)
        .css_classes(["dvoty-scroll"])
        .build();

    let wrapper = ListBox::builder().css_classes(["dvoty-wrapper"]).build();
    wrapper.append(&input);
    wrapper.append(&list_wrapper);

    result.set_child(Some(&wrapper));
    register_widget(super::app::Widget::Dvoty, result.id());

    input.grab_focus();

    result.present();

    if !config.dvoty.window.visible_on_start {
        result.set_visible(false);
    }

    result
}
