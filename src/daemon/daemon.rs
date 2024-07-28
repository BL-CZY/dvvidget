use super::structs::DaemonEvt;
use super::{renderer, server};
use crate::utils::DaemonErr;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub fn start_daemon(path: Option<String>) -> Result<(), DaemonErr> {
    let (evt_sender, evt_receiver): (UnboundedSender<DaemonEvt>, UnboundedReceiver<DaemonEvt>) =
        mpsc::unbounded_channel();

    if let Err(e) = renderer::init_gtk_async(evt_receiver) {
        return Err(e);
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .thread_name("dvvidget server main")
        .enable_all()
        .build()
        .unwrap();

    // run the server in a different thread
    std::thread::Builder::new()
        .name("dvvidget server".into())
        .spawn(move || {
            rt.block_on(async {
                if let Err(e) = server::run_server(path, evt_sender.clone()).await {
                    println!("Error running the server: {:?}, exiting...", e);
                    return;
                }
                // use tokio::spawn if there are more tasks here, such as information puller
            });
        })
        .expect("failed to start the async thread");
    Ok(())
}
