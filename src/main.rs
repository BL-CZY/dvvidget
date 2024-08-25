pub mod cli;
pub mod daemon;
pub mod utils;

use clap::Parser;
use cli::args::{self, Args, DaemonCmd};
use daemon::structs::DaemonEvt;

fn main() {
    let args = Args::parse();

    match args.commands {
        args::Command::Daemon { path, option } => {
            if option.is_none() {
                if let Err(e) = daemon::start_daemon(path) {
                    println!("Error starting the daemon: {:?}", e)
                };
            } else {
                match option.unwrap() {
                    DaemonCmd::Start => {
                        if let Err(e) = daemon::start_daemon(path) {
                            println!("Error starting the daemon: {:?}", e)
                        };
                    }
                    DaemonCmd::Shutdown => {
                        if let Err(e) = cli::send_evt(DaemonEvt::ShutDown) {
                            println!("Error sending event: {:?}", e)
                        }
                    }
                }
            }
        }

        args::Command::Volume { value } => {
            if let Err(e) = cli::send_evt(DaemonEvt::AdjustVol(value)) {
                println!("Err Sending event: {:?}", e);
            }
        }
    }
}
