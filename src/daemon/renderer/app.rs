use std::sync::Arc;

use crate::daemon::structs::DaemonEvt;
use crate::utils::DaemonErr;
use gio::ApplicationFlags;
use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::Scale;
use tokio::sync::mpsc::UnboundedReceiver;

use super::config::read_config;
use super::popup::create_sound_osd;

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
                    match evt {
                        DaemonEvt::ShutDown => {
                            app.quit();
                        },
                        DaemonEvt::AdjustVol(val) => {
                            app.windows().iter().nth(0).unwrap().child().and_downcast_ref::<Scale>().unwrap().set_value(val as f64);
                        },
                        _ => {}
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
        &gdk::Display::default().expect("Ughhhhhhhhhhhhhhh"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_SETTINGS,
    );
    create_sound_osd(app).present();
}

pub fn start_app(evt_receiver: UnboundedReceiver<DaemonEvt>, path: Option<String>) {
    gtk4::init().unwrap();

    let app = Arc::new(gtk4::Application::new(
        Some("org.dvida.dvvidgets"),
        ApplicationFlags::default(),
    ));

    let app_descriptor = read_config(path);

    if let Err(e) = init_gtk_async(evt_receiver, app.clone()) {
        println!("oh no {:?}", e);
    };

    let mut vol_id: u32 = 0;

    app.connect_activate(|app| activate(&app));

    app.run_with_args(&[""]);
}
