use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::daemon::structs::DaemonCmd;
use crate::daemon::structs::DaemonEvt;
use crate::daemon::structs::DaemonRes;
use crate::utils::DaemonErr;
use crate::utils::DisplayBackend;
use gio::ApplicationFlags;
use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::CssProvider;
use lazy_static::lazy_static;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use super::bri::create_bri_osd;
use super::bri::handle_bri_cmd;
use super::bri::BriContext;
use super::config::AppConf;
use super::dvoty::create_dvoty;
use super::dvoty::handle_dvoty_cmd;
use super::dvoty::DvotyContext;
use super::vol::create_sound_osd;
use super::vol::handle_vol_cmd;
use super::vol::VolContext;

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub enum Widget {
    Volume = 0,
    Brightness = 1,
    Dvoty = 2,
}

pub struct AppContext {
    pub vol: VolContext,
    pub bri: BriContext,
    pub dvoty: DvotyContext,
}

impl AppContext {
    pub fn from_config(config: &Arc<AppConf>) -> Self {
        AppContext {
            vol: VolContext::from_config(config),
            bri: BriContext::from_config(config),
            dvoty: DvotyContext::default(),
        }
    }

    pub fn set_virtual_volume(&mut self, val: f64) -> f64 {
        if val > self.vol.max_vol {
            self.vol.cur_vol = self.vol.max_vol;
        } else if val < 0f64 {
            self.vol.cur_vol = 0f64;
        } else {
            self.vol.cur_vol = val;
        }

        self.vol.cur_vol
    }

    pub fn set_virtual_brightness(&mut self, val: f64) -> f64 {
        if val > 100f64 {
            self.bri.cur_bri = 100f64;
        } else if val < 0f64 {
            self.bri.cur_bri = 0f64;
        } else {
            self.bri.cur_bri = val;
        }

        self.bri.cur_bri
    }
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

fn process_evt(
    evt: DaemonCmd,
    app: Rc<Application>,
    sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
    app_context: Rc<RefCell<AppContext>>,
) -> Result<DaemonRes, DaemonErr> {
    match evt {
        DaemonCmd::ShutDown => {
            app.quit();
        }

        DaemonCmd::Vol(evt) => {
            let guard = match IDS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let result = handle_vol_cmd(
                evt,
                &app.window_by_id(*guard.get(&Widget::Volume).unwrap())
                    .unwrap(),
                sender,
                app_context,
                config,
            )?;

            return Ok(result);
        }

        DaemonCmd::Bri(evt) => {
            let guard = match IDS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let result = handle_bri_cmd(
                evt,
                &app.window_by_id(*guard.get(&Widget::Brightness).unwrap())
                    .unwrap(),
                sender,
                app_context,
                config,
            )?;

            return Ok(result);
        }

        DaemonCmd::Dvoty(evt) => {
            let guard = match IDS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let result = handle_dvoty_cmd(
                evt,
                &app.window_by_id(*guard.get(&Widget::Dvoty).unwrap())
                    .unwrap(),
                sender,
                app_context,
                config,
            )?;

            return Ok(result);
        }
    }

    Ok(DaemonRes::Success)
}

fn send_res(sender: Option<UnboundedSender<DaemonRes>>, res: DaemonRes) {
    if sender.is_none() {
        return;
    }

    if let Err(e) = sender.unwrap().send(res) {
        println!("Err sending daemon response to the server: {:?}", e);
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum VolBriTaskType {
    AwaitClose,
    MurphValue,
}

pub fn init_gtk_async(
    mut evt_receiver: UnboundedReceiver<DaemonEvt>,
    evt_sender: UnboundedSender<DaemonEvt>,
    app: Rc<Application>,
    config: Arc<AppConf>,
) -> Result<(), DaemonErr> {
    let context = Rc::new(RefCell::new(AppContext::from_config(&config)));

    glib::MainContext::default().spawn_local(async move {
        loop {
            tokio::select! {
                Ok(()) = crate::utils::receive_exit() => {
                    println!("Shutting down the GUI...");
                    app.quit();
                    break;
                }

                Some(evt) = evt_receiver.recv() => {
                    match process_evt(evt.evt, app.clone(), evt_sender.clone(), config.clone(), context.clone()) {
                        Err(e) => send_res(evt.sender, DaemonRes::Failure(format!("{:?}", e))),
                        Ok(res) => send_res(evt.sender, res),
                    }
                }
            }
        }
    });

    Ok(())
}

fn activate(
    backend: DisplayBackend,
    app: &gtk4::Application,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
) {
    let css = CssProvider::new();
    css.load_from_path(&config.general.css_path);
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("cant get display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_SETTINGS,
    );
    create_sound_osd(backend, app, config.clone());
    create_bri_osd(backend, app, config.clone());
    create_dvoty(backend, app, config.clone(), sender.clone());
}

pub fn start_app(
    backend: DisplayBackend,
    evt_receiver: UnboundedReceiver<DaemonEvt>,
    evt_sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
) {
    gtk4::init().unwrap();

    let app = Rc::new(gtk4::Application::new(
        Some("org.dvida.dvvidgets"),
        ApplicationFlags::default(),
    ));

    if let Err(e) = init_gtk_async(
        evt_receiver,
        evt_sender.clone(),
        app.clone(),
        config.clone(),
    ) {
        println!("Err handling command: {:?}", e);
    }

    app.connect_activate(move |app| activate(backend, app, config.clone(), evt_sender.clone()));

    app.run_with_args(&[""]);
}
