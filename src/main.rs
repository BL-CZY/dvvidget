pub mod cli;
pub mod daemon;
pub mod utils;

use clap::Parser;
use cli::args::{self, Args};
use daemon::renderer::app::start_app;

fn main() {
    let args = Args::parse();
    start_app();

    // match args.commands {
    //     args::Command::Daemon { path } => {
    //         if let Err(e) = daemon::start_daemon(path) {
    //             println!("Error starting the daemon: {:?}", e)
    //         };
    //     }

    //     args::Command::Connect => {
    //         let rt = tokio::runtime::Builder::new_current_thread()
    //             .thread_name("dvvidget client")
    //             .enable_all()
    //             .build()
    //             .unwrap();

    //         rt.block_on(async {
    //             if let Err(e) = cli::client::connect_server().await {
    //                 println!("Error running the client: {:?}", e);
    //             }
    //         })
    //     }
    // }
}
