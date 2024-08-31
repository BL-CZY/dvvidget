use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::daemon::structs::DaemonCmd;
use crate::daemon::structs::DaemonEvt;
use crate::daemon::structs::DaemonRes;
use crate::utils::DaemonErr;
use gio::ApplicationFlags;
use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::CssProvider;
use lazy_static::lazy_static;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use super::config::read_config;
use super::popup::create_sound_osd;
use super::popup::handle_vol_cmd;

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub enum Widget {
    Volume = 0,
}

pub fn register_widget(widget: Widget, id: u32) {
    let mut guard = match IDS.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    guard.insert(widget, id);
}

lazy_static! {
    pub static ref IDS: Mutex<HashMap<Widget, u32>> = Mutex::new(HashMap::new());
}

fn process_evt(evt: DaemonCmd, app: Arc<Application>) -> Result<DaemonRes, DaemonErr> {
    match evt {
        DaemonCmd::ShutDown => {
            app.quit();
        }

        DaemonCmd::GetVol | DaemonCmd::SetVol(_) | DaemonCmd::IncVol(_) | DaemonCmd::DecVol(_) => {
            let guard = match IDS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            handle_vol_cmd(
                evt,
                &app.window_by_id(*guard.get(&Widget::Volume).unwrap())
                    .unwrap(),
            )?;
        }

        _ => {}
    }

    Ok(DaemonRes::Success)
}

fn send_res(sender: UnboundedSender<DaemonRes>, res: DaemonRes) {
    if let Err(e) = sender.send(res) {
        println!("Err sending daemon response to the server: {:?}", e);
    }
}

pub fn init_gtk_async(
    mut evt_receiver: UnboundedReceiver<DaemonEvt>,
    app: Arc<Application>,
) -> Result<(), DaemonErr> {
    glib::MainContext::default().spawn_local(async move {
        loop {
            tokio::select! {
                Ok(()) = crate::utils::receive_exit() => {
                    app.quit();
                    break;
                }

                Some(evt) = evt_receiver.recv() => {
                    // TODO do this
                    match process_evt(evt.evt, app.clone()) {
                        Err(e) => send_res(evt.sender, DaemonRes::Failure(format!("{:?}", e))),
                        Ok(res) => send_res(evt.sender, res),
                    }
                }
            }
        }
    });

    Ok(())
}

fn activate(app: &gtk4::Application) {
    let css = CssProvider::new();
    css.load_from_data(include_str!("style.css"));
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("cant get display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_SETTINGS,
    );
    create_sound_osd(app).present();
    app.window_by_id(1).unwrap();
}

pub fn start_app(evt_receiver: UnboundedReceiver<DaemonEvt>, path: Option<String>) {
    gtk4::init().unwrap();

    let app = Arc::new(gtk4::Application::new(
        Some("org.dvida.dvvidgets"),
        ApplicationFlags::default(),
    ));

    let _app_descriptor = read_config(path);

    if let Err(e) = init_gtk_async(evt_receiver, app.clone()) {
        println!("Err handling command: {:?}", e);
    }

    app.connect_activate(|app| activate(&app));

    app.run_with_args(&[""]);
}
