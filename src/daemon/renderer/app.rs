use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

use crate::daemon::notification::denote::Notification;
use crate::daemon::structs::DaemonCmdType;
use crate::daemon::structs::DaemonEvt;
use crate::daemon::structs::DaemonRes;
use crate::utils::DaemonErr;
use crate::utils::DisplayBackend;
use crate::utils::ExitType;
use gio::ApplicationFlags;
use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::Window;
use lazy_static::lazy_static;
use std::process::Command;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use super::bri::create_bri_osd;
use super::bri::handle_bri_cmd;
use super::bri::BriContext;
use super::config::AppConf;
use super::dvoty::create_dvoty;
use super::dvoty::event::CURRENT_IDS;
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
    pub monitor_count: usize,
}

pub static IS_GUI_SHUT: AtomicBool = AtomicBool::new(false);

impl AppContext {
    pub fn from_config(config: &Arc<AppConf>, monitor_count: usize) -> Self {
        let vol = VolContext::from_config(config, monitor_count);
        let bri = BriContext::from_config(config, monitor_count);
        let dvoty = DvotyContext::from_config(config, monitor_count);

        AppContext {
            vol,
            bri,
            dvoty,
            monitor_count,
        }
    }
}

pub fn register_widget(widget: Widget, id: u32) {
    let mut guard = match WINDOWS.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    match guard.get_mut(&widget) {
        Some(v) => {
            v.push(id);
        }
        None => {
            guard.insert(widget, vec![id]);
        }
    }
}

lazy_static! {
    pub static ref WINDOWS: Mutex<HashMap<Widget, Vec<u32>>> = Mutex::new(HashMap::new());
}

fn get_windows(
    widget: Widget,
    guard: &HashMap<Widget, Vec<u32>>,
    app: &Rc<Application>,
) -> Vec<Window> {
    guard
        .get(&widget)
        .unwrap()
        .iter()
        .map(|id| app.window_by_id(*id).unwrap())
        .collect::<Vec<Window>>()
}

fn process_evt(
    evt: DaemonCmdType,
    app: Rc<Application>,
    sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
    app_context: Rc<RefCell<AppContext>>,
    monitors: Vec<usize>,
    id: Option<uuid::Uuid>,
) -> Result<DaemonRes, DaemonErr> {
    match evt {
        DaemonCmdType::ShutDown => {
            app.quit();
        }

        DaemonCmdType::Vol(evt) => {
            let guard = match WINDOWS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let vol_context = &mut app_context.borrow_mut().vol;

            let result = handle_vol_cmd(
                evt,
                &get_windows(Widget::Volume, &guard, &app),
                sender,
                vol_context,
                config,
                monitors,
            )?;

            return Ok(result);
        }

        DaemonCmdType::Bri(evt) => {
            let guard = match WINDOWS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let bri_context = &mut app_context.borrow_mut().bri;

            let result = handle_bri_cmd(
                evt,
                &get_windows(Widget::Brightness, &guard, &app),
                sender,
                bri_context,
                config,
                monitors,
            )?;

            return Ok(result);
        }

        DaemonCmdType::Dvoty(evt) => {
            let guard = match WINDOWS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let dvoty_context = &mut app_context.borrow_mut().dvoty;

            let result = handle_dvoty_cmd(
                evt,
                &get_windows(Widget::Dvoty, &guard, &app),
                sender,
                dvoty_context,
                config,
                monitors,
                id,
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
pub enum VolBriTaskTypeWindow {
    AwaitClose,
}

#[derive(Hash, PartialEq, Eq)]
pub enum VolBriTaskType {
    MurphValue,
}

fn activate(
    backend: DisplayBackend,
    app: &gtk4::Application,
    config: Arc<AppConf>,
    sender: UnboundedSender<DaemonEvt>,
    monitors: Vec<gdk::Monitor>,
    app_context: Rc<RefCell<AppContext>>,
) {
    let css = CssProvider::new();
    css.load_from_path(&config.general.css_path);
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Cannot open display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );

    for (ind, monitor) in monitors.iter().enumerate() {
        create_sound_osd(backend, app, config.clone(), monitor);
        create_bri_osd(backend, app, config.clone(), monitor);
        create_dvoty(
            backend,
            app,
            config.clone(),
            sender.clone(),
            ind,
            monitor,
            app_context.clone(),
        );
    }
}

pub fn start_app(
    backend: DisplayBackend,
    evt_receiver: UnboundedReceiver<DaemonEvt>,
    evt_sender: UnboundedSender<DaemonEvt>,
    notification_receiver: UnboundedReceiver<Notification>,
    config: Arc<AppConf>,
    monitor_list: Vec<gdk::Monitor>,
) {
    super::dvoty::app_launcher::DESKTOP_FILES
        .set(Arc::new(Mutex::new(vec![])))
        .unwrap();

    let mut ids = vec![];
    for _ in 0..monitor_list.len() {
        ids.push(Arc::new(Mutex::new(uuid::Uuid::new_v4())));
    }
    CURRENT_IDS.set(ids).unwrap();

    #[cfg(not(debug_assertions))]
    let name = Some("org.dvida.dvvidgets");

    #[cfg(debug_assertions)]
    let name = Some("org.dvida.dvvidgets.debug");

    let app = Rc::new(gtk4::Application::new(
        name,
        ApplicationFlags::NON_UNIQUE | ApplicationFlags::ALLOW_REPLACEMENT,
    ));

    let context = Rc::new(RefCell::new(AppContext::from_config(
        &config,
        monitor_list.len(),
    )));

    if let Err(e) = init_gtk_async(
        evt_receiver,
        evt_sender.clone(),
        app.clone(),
        config.clone(),
        &monitor_list,
        context.clone(),
        notification_receiver,
    ) {
        println!("Err handling command: {:?}", e);
    }

    app.connect_activate(move |app| {
        activate(
            backend,
            app,
            config.clone(),
            evt_sender.clone(),
            monitor_list.clone(),
            context.clone(),
        )
    });

    app.run_with_args(&[""]);
}

fn handle_notification(notification: Notification) {
    println!("{:?}", notification);
}

pub fn init_gtk_async(
    mut evt_receiver: UnboundedReceiver<DaemonEvt>,
    evt_sender: UnboundedSender<DaemonEvt>,
    app: Rc<Application>,
    config: Arc<AppConf>,
    _monitor_list: &[gdk::Monitor],
    app_context: Rc<RefCell<AppContext>>,
    mut notification_receiver: UnboundedReceiver<Notification>,
) -> Result<(), DaemonErr> {
    glib::MainContext::default().spawn_local(async move {
        loop {
            tokio::select! {
                Ok(t) = crate::utils::receive_exit() => {
                    println!("Shutting down the GUI...");
                    app.quit();

                    if let ExitType::Restart = t {
                        let args: Vec<String> = std::env::args().collect();
                        let executable = &args[0];

                        Command::new(executable)
                            .args(&args[1..])
                            .spawn()
                            .expect("Failed to restart");

                        std::process::exit(0);
                    }

                    break;
                }

                Some(evt) = evt_receiver.recv() => {
                    match process_evt(evt.evt, app.clone(), evt_sender.clone(), config.clone(), app_context.clone(), evt.monitors, evt.uuid) {
                        Err(e) => send_res(evt.sender, DaemonRes::Failure(format!("{:?}", e))),
                        Ok(res) => send_res(evt.sender, res),
                    }
                }

                Some(notification) = notification_receiver.recv() => {
                    handle_notification(notification);
                }
            }
        }
    });

    Ok(())
}
