use std::path::PathBuf;
use std::sync::Arc;

use super::renderer::config::default_config_path;
use super::renderer::window::KeyboardModeWrapper;
use super::renderer::{app::start_app, config::read_config};
use super::server;
use super::structs::DaemonEvt;
use crate::utils::{detect_display, DaemonErr};
use glib::object::Cast;
use gtk4::prelude::DisplayExt;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub fn start_daemon(
    config_path: Option<String>,
    socket_path: Option<String>,
) -> Result<(), DaemonErr> {
    // init cache
    let mut cache_dir = PathBuf::from(std::env::var("HOME").expect("Cannot find home dir"));
    cache_dir.push(".cache/dvvidget");

    if std::fs::read_dir(&cache_dir).is_err() {
        std::fs::create_dir_all(&cache_dir).expect("Cannot create cache directory");
    }

    println!("Cache directory at {:?}", cache_dir);

    cache_dir.push("histfile");

    if std::fs::read(&cache_dir).is_err() {
        std::fs::write(&cache_dir, "")
            .unwrap_or_else(|_| println!("Cannot create histfile in cache directory"));
    }

    println!("Dvoty histfile at {:?}", cache_dir);

    let backend = detect_display();

    let config_path = if let Some(p) = config_path {
        p.into()
    } else {
        default_config_path()
    };

    let config = Arc::new({
        let mut c = read_config(&config_path);
        c.dvoty.window.keyboard_mode = KeyboardModeWrapper {
            inner: gtk4_layer_shell::KeyboardMode::OnDemand,
        };
        c
    });

    let (evt_sender, evt_receiver): (UnboundedSender<DaemonEvt>, UnboundedReceiver<DaemonEvt>) =
        mpsc::unbounded_channel();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_name("dvvidget server main")
        .enable_all()
        .build()
        .unwrap();

    let handle = rt.handle().clone();

    simple_signal::set_handler(
        &[simple_signal::Signal::Int, simple_signal::Signal::Term],
        move |_| {
            crate::utils::shutdown("Received int/term signal");
        },
    );

    let _g = handle.enter();

    gtk4::init().unwrap();

    let display = gtk4::gdk::Display::default().unwrap();
    let monitors = display.monitors();

    let mut monitor_list = vec![];

    for monitor in (&monitors).into_iter().flatten() {
        if let Ok(mon) = monitor.downcast::<gtk4::gdk::Monitor>() {
            monitor_list.push(mon);
        }
    }

    // run the server in a different thread
    let evt_sender_clone = evt_sender.clone();
    let len = monitor_list.len();
    std::thread::Builder::new()
        .name("dvvidget server".into())
        .spawn(move || {
            rt.block_on(async {
                if let Err(e) = server::run_server(&config_path, socket_path, evt_sender.clone(), len).await {
                    println!("Error running the IPC server: {:?}. Dvvidget will keep running, but the cli won't work", e);
                }
                // use tokio::spawn if there are more tasks here, such as information puller
            });
        })
        .expect("Failed to start the async thread: thread failed to initialize");

    start_app(
        backend,
        evt_receiver,
        evt_sender_clone.clone(),
        config,
        monitor_list,
    );

    Ok(())
}
