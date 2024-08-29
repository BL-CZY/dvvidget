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

        args::Command::Volume { actions } => {
            let evt = match actions {
                args::VolCmd::Get => DaemonEvt::GetVol,
                args::VolCmd::Set { value } => DaemonEvt::SetVol(value),
                args::VolCmd::Inc { value } => DaemonEvt::IncVol(value),
                args::VolCmd::Dec { value } => DaemonEvt::DecVol(value),
            };
            if let Err(e) = cli::send_evt(evt) {
                println!("Err Sending event: {:?}", e);
            }
        }
    }
}
