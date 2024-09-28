use crate::daemon::structs::DaemonCmd;
use crate::daemon::structs::DaemonEvt;
use crate::daemon::structs::DaemonRes;
use crate::daemon::structs::Dvoty;
use crate::utils::DaemonErr;
use crate::utils::DisplayBackend;
use glib::object::CastNone;
use gtk4::prelude::ApplicationWindowExt;
use gtk4::prelude::BoxExt;
use gtk4::prelude::EditableExt;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use gtk4::Box;
use gtk4::Label;
use gtk4::Window;
use gtk4::{Application, ApplicationWindow, Entry, ListBox, ScrolledWindow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use super::app::register_widget;
use super::app::AppContext;
use super::{config::AppConf, window};

#[derive(PartialEq, Eq, Hash)]
pub enum DvotyTaskType {
    ProcessInput,
}

#[derive(Default)]
pub struct DvotyContext {
    pub dvoty_tasks: HashMap<DvotyTaskType, JoinHandle<()>>,
}

fn get_list(window: &Window) -> Result<ListBox, ()> {
    if let Some(outer_list) = window.child().and_downcast_ref::<ListBox>() {
        println!("outer list: {:?}", outer_list.css_classes());
        if let Some(scroll_wrap) = outer_list.last_child() {
            if let Some(scroll) = scroll_wrap
                .first_child()
                .and_downcast_ref::<ScrolledWindow>()
            {
                println!("scroll: {:?}", scroll.css_classes());
                if let Some(result) = scroll.child().and_downcast::<ListBox>() {
                    return Ok(result);
                }
            }
        }
    }

    Err(())
}

fn process_input(
    input: String,
    context: Rc<RefCell<AppContext>>,
    sender: UnboundedSender<DaemonEvt>,
    window: &Window,
) -> Result<(), DaemonErr> {
    let context_ref = &mut context.borrow_mut();
    let task_map = &mut context_ref.dvoty.dvoty_tasks;

    let list = if let Ok(res) = get_list(window) {
        res
    } else {
        println!("Dvoty: can't find list");
        return Err(DaemonErr::CannotFindWidget);
    };

    list.remove_all();

    if let Some(handle) = task_map.get(&DvotyTaskType::ProcessInput) {
        handle.abort();
        task_map.remove(&DvotyTaskType::ProcessInput);
    }

    let handle = tokio::spawn(async move {
        println!("yes: {}", input);
    });

    task_map.insert(DvotyTaskType::ProcessInput, handle);

    Ok(())
}

pub fn handle_dvoty_cmd(
    cmd: Dvoty,
    window: &Window,
    sender: UnboundedSender<DaemonEvt>,
    app_context: Rc<RefCell<AppContext>>,
    _config: Arc<AppConf>,
) -> Result<DaemonRes, DaemonErr> {
    match cmd {
        Dvoty::Update(str) => {
            process_input(str, app_context, sender.clone(), window)?;
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
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> for math expressions", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> for launching apps", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> for running commands", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> for searching online", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
        (format!("Input <span background=\"{}\" foreground=\"{}\" size=\"x-large\"> = </span> for opening url", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color), format!("<span background=\"{}\" foreground=\"{}\" size=\"x-large\"> ? </span>", config.dvoty.highlight_bg_color, config.dvoty.highlight_fg_color)),
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
    let input = Entry::builder().css_classes(["dvoty-entry"]).build();

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

    result.present();

    if !config.dvoty.window.visible_on_start {
        result.set_visible(false);
    }

    result
}
