use super::server;
use super::structs::{DaemonCmd, DaemonEvt};
use super::{renderer::app::start_app, structs::DaemonRes};
use crate::utils::DaemonErr;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub fn start_daemon(path: Option<String>) -> Result<(), DaemonErr> {
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
            crate::utils::shutdown();
        },
    );

    // run the server in a different thread
    let alt_path = path.clone();
    std::thread::Builder::new()
        .name("dvvidget server".into())
        .spawn(move || {
            rt.block_on(async {
                if let Err(e) = server::run_server(alt_path, evt_sender.clone()).await {
                    println!("Error running the server: {:?}, exiting...", e);
                    return;
                }
                // use tokio::spawn if there are more tasks here, such as information puller
            });
        })
        .expect("failed to start the async thread");

    let _g = handle.enter();

    start_app(evt_receiver, path);

    Ok(())
}
