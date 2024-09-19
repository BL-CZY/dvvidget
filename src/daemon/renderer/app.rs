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
use tokio::task::JoinHandle;

use super::config::AppConf;
use super::vol::create_sound_osd;
use super::vol::handle_vol_cmd;
use super::vol::VolContext;

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub enum Widget {
    Volume = 0,
}

pub struct AppContext {
    pub vol: VolContext,
}

impl AppContext {
    pub fn from_config(config: &Arc<AppConf>) -> Self {
        AppContext {
            vol: VolContext::from_config(config),
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
    vol_tasks: Rc<RefCell<HashMap<VolTaskType, JoinHandle<()>>>>,
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
                vol_tasks,
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
pub enum VolTaskType {
    AwaitClose,
    MurphValue,
}

pub fn init_gtk_async(
    mut evt_receiver: UnboundedReceiver<DaemonEvt>,
    evt_sender: UnboundedSender<DaemonEvt>,
    app: Rc<Application>,
    config: Arc<AppConf>,
) -> Result<(), DaemonErr> {
    let vol_tasks: Rc<RefCell<HashMap<VolTaskType, JoinHandle<()>>>> =
        Rc::new(RefCell::new(HashMap::new()));

    let context = Rc::new(RefCell::new(AppContext::from_config(&config)));

    glib::MainContext::default().spawn_local(async move {
        loop {
            tokio::select! {
                Ok(()) = crate::utils::receive_exit() => {
                    app.quit();
                    break;
                }

                Some(evt) = evt_receiver.recv() => {
                    match process_evt(evt.evt, app.clone(), evt_sender.clone(), config.clone(), vol_tasks.clone(), context.clone()) {
                        Err(e) => send_res(evt.sender, DaemonRes::Failure(format!("{:?}", e))),
                        Ok(res) => send_res(evt.sender, res),
                    }
                }
            }
        }
    });

    Ok(())
}

fn activate(backend: DisplayBackend, app: &gtk4::Application, config: Arc<AppConf>) {
    let css = CssProvider::new();
    css.load_from_data(&std::fs::read_to_string(&config.general.css_path).unwrap());
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("cant get display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_SETTINGS,
    );
    create_sound_osd(backend, app, config);
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

    if let Err(e) = init_gtk_async(evt_receiver, evt_sender, app.clone(), config.clone()) {
        println!("Err handling command: {:?}", e);
    }

    app.connect_activate(move |app| activate(backend, app, config.clone()));

    app.run_with_args(&[""]);
}
