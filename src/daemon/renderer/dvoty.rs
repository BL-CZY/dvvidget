use gtk4::prelude::ApplicationWindowExt;
use gtk4::prelude::EditableExt;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use gtk4::{Application, ApplicationWindow, Entry, ListBox, ScrolledWindow};
use std::sync::Arc;

use crate::utils::DisplayBackend;

use super::app::register_widget;
use super::{config::AppConf, window};

pub fn create_dvoty(
    backend: DisplayBackend,
    app: &Application,
    config: Arc<AppConf>,
) -> ApplicationWindow {
    let result = window::create_window(&backend, app, &config.dvoty.window);
    result.add_css_class("dvoty-window");

    let list_box = ListBox::builder().css_classes(["dvoty-list"]).build();
    let input = Entry::builder().css_classes(["dvoty-entry"]).build();

    input.connect_changed(|entry| {
        println!("{}", entry.text());
    });

    let list_wrapper = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_width(config.dvoty.max_row as i32)
        .child(&list_box)
        .css_classes(["dvoty-list-wrapper"])
        .build();

    let wrapper = ListBox::builder().css_classes(["dvoty-wrapper"]).build();
    wrapper.append(&input);
    wrapper.append(&list_wrapper);

    result.set_child(Some(&wrapper));
    register_widget(super::app::Widget::Dvoty, result.id());

    result.present();

    if !config.dvoty.window.visible_on_start {
        result.hide();
    }

    result
}
