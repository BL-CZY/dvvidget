use std::sync::Arc;

use super::file::start_file_server;
use super::renderer::{app::start_app, config::read_config};
use super::server;
use super::structs::DaemonEvt;
use crate::utils::{detect_display, DaemonErr};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub fn start_daemon(path: Option<String>) -> Result<(), DaemonErr> {
    let backend = detect_display();

    let config = Arc::new(read_config(path.clone()));

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

    // run the server in a different thread
    let alt_path = path.clone();
    let evt_sender_clone = evt_sender.clone();
    std::thread::Builder::new()
        .name("dvvidget server".into())
        .spawn(move || {
            rt.block_on(async {
                if let Err(e) = server::run_server(alt_path, evt_sender.clone()).await {
                    println!("Error running the IPC server: {:?}. Dvvidget will keep running, but the cli won't work", e);
                }
                // use tokio::spawn if there are more tasks here, such as information puller
            });
        })
        .expect("Failed to start the async thread.");

    std::thread::Builder::new()
        .name("dvvidget file watcher".into())
        .spawn(|| {
            start_file_server().unwrap_or_else(|e| {
                println!("Error running the file watcher: {:?}. Dvvidget will keep running, but the file watcher won't work", e);
            });
        })
        .expect("failed to start the file server");

    let _g = handle.enter();

    start_app(backend, evt_receiver, evt_sender_clone.clone(), config);

    Ok(())
}
