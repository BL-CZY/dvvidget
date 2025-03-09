use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;

use crate::daemon::renderer::dvoty::event::CURRENT_ID;
use crate::daemon::structs::DaemonCmd;
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
    pub dvoty: Vec<DvotyContext>,
    pub monitor_count: usize,
}

pub static IS_GUI_SHUT: AtomicBool = AtomicBool::new(false);

impl AppContext {
    pub fn from_config(config: &Arc<AppConf>, monitor_count: usize) -> Self {
        let vol = VolContext::from_config(config);
        let bri = BriContext::from_config(config);
        let mut dvoty=  vec![];

        for _ in 0..monitor_count {
            dvoty.push(DvotyContext::default());
        }

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

fn get_window_id(widget: Widget, monitor: usize, map: &HashMap<Widget, Vec<u32>>) -> Result<u32, DaemonErr> {
    match map.get(&widget) {
        Some(list) => {
            match list.get(monitor) {
                Some(num) => Ok(*num),
                None => Err(DaemonErr::CannotFindWidget)
            }
        }
        None => Err(DaemonErr::CannotFindWidget)
    }    
}

fn get_windows(widget: Widget, guard: &HashMap<Widget, Vec<u32>>, app: &Rc<Application>) -> Vec<Window> {
    guard.get(&widget).unwrap().iter().map(|id|{
        app.window_by_id(*id).unwrap()
    }).collect::<Vec<Window>>()
}

fn process_evt(
    evt: DaemonCmd,
    app: Rc<Application>,
    sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
    app_context: Rc<RefCell<AppContext>>,
    monitor: usize,
) -> Result<DaemonRes, DaemonErr> {
    match evt {
        DaemonCmd::ShutDown => {
            app.quit();
        }

        DaemonCmd::Vol(evt) => {
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
                monitor
            )?;

            return Ok(result);
        }

        DaemonCmd::Bri(evt) => {
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
                monitor
            )?;

            return Ok(result);
        }

        DaemonCmd::Dvoty(evt) => {
            let guard = match WINDOWS.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let result = handle_dvoty_cmd(
                evt,
                &app.window_by_id(get_window_id(Widget::Dvoty, monitor, &guard)?).unwrap(),
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
    monitor_count: usize,
) -> Result<(), DaemonErr> {
    let context = Rc::new(RefCell::new(AppContext::from_config(&config, monitor_count)));

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
                    if let Some(id) = evt.uuid {
                        if id == *CURRENT_ID.lock().unwrap(){
                            match process_evt(evt.evt, app.clone(), evt_sender.clone(), config.clone(), context.clone(), evt.monitor) {
                                Err(e) => send_res(evt.sender, DaemonRes::Failure(format!("{:?}", e))),
                                Ok(res) => send_res(evt.sender, res),
                            }
                        }
                    } else {
                        match process_evt(evt.evt, app.clone(), evt_sender.clone(), config.clone(), context.clone(), evt.monitor) {
                            Err(e) => send_res(evt.sender, DaemonRes::Failure(format!("{:?}", e))),
                            Ok(res) => send_res(evt.sender, res),
                        }
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
        &gdk::Display::default().expect("Cannot open display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_SETTINGS,
    );
    
    let monitors = gdk::Display::default().expect("Cannot open display").monitors();

    for monitor in &monitors {
        if let Ok(_mon) = monitor {
            create_sound_osd(backend, app, config.clone());
        }
    }

    create_bri_osd(backend, app, config.clone());
    create_dvoty(backend, app, config.clone(), sender.clone());
}

pub fn start_app(
    backend: DisplayBackend,
    evt_receiver: UnboundedReceiver<DaemonEvt>,
    evt_sender: UnboundedSender<DaemonEvt>,
    config: Arc<AppConf>,
) {
    super::dvoty::app_launcher::DESKTOP_FILES
        .set(Arc::new(Mutex::new(vec![])))
        .unwrap();

    gtk4::init().unwrap();

    let display = gtk4::gdk::Display::default().unwrap();
    let monitors = display.monitors();
    
    let mut count: usize = 0;

    for monitor in &monitors {
        if let Ok(monitor) = monitor {
            if monitor.downcast::<gtk4::gdk::Monitor>().is_ok() {
                count += 1;
            }
        }
    }

    //let recent_manager = gtk4::RecentManager::default();
    //let recent_items = recent_manager.items();

    //let recent_file_names: Vec<String> = recent_items
    //    .iter()
    //    .filter_map(|item| {
    //        let uri = item.uri();
    //        gio::File::for_uri(&uri)
    //            .path()
    //            .map(|path| path.to_string_lossy().into_owned())
    //            .or_else(|| Some(uri.to_string()))
    //    })
    //    .collect();
    //
    //println!("{:?}", recent_file_names);
    
    let app = Rc::new(gtk4::Application::new(
        #[cfg(not(debug_assertions))]
        Some("org.dvida.dvvidgets"),
        #[cfg(debug_assertions)]
        Some("org.dvida.dvvidgets.debug"),
        ApplicationFlags::default(),
    ));

    if let Err(e) = init_gtk_async(
        evt_receiver,
        evt_sender.clone(),
        app.clone(),
        config.clone(),
        count
    ) {
        println!("Err handling command: {:?}", e);
    }

    app.connect_activate(move |app| activate(backend, app, config.clone(), evt_sender.clone()));

    app.run_with_args(&[""]);
}
