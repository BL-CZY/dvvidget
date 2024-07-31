use std::sync::Arc;

use crate::daemon::structs::DaemonEvt;
use crate::utils::DaemonErr;
use gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4::Application;
use tokio::sync::mpsc::UnboundedReceiver;

use super::config::read_config;
use super::dock::create_dock;
use super::dock::DockDescriptor;

pub struct AppDescriptor {
    pub dock: DockDescriptor,
}

impl AppDescriptor {
    pub fn new() -> Self {
        AppDescriptor {
            dock: DockDescriptor::new(),
        }
    }
}

pub fn handle_evt(evt: DaemonEvt, app: Arc<Application>) {}

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
                    if let DaemonEvt::ShutDown = evt {
                        app.quit();
                        break;
                    }
                    handle_evt(evt, app.clone());
                }
            }
        }
    });
    Ok(())
}

fn activate(app: &gtk4::Application) {
    create_dock(app).present();
}

pub fn start_app(evt_receiver: UnboundedReceiver<DaemonEvt>, path: Option<String>) {
    gtk4::init().unwrap();

    let app = Arc::new(gtk4::Application::new(
        Some("org.dvida.dvvidgets"),
        ApplicationFlags::default(),
    ));

    if let Err(e) = init_gtk_async(evt_receiver, app.clone()) {
        println!("failed to start app: {:?}", e);
        return;
    };

    let app_descriptor = read_config(path);

    app.connect_activate(|app| activate(&app));

    app.run_with_args(&[""]);
}
