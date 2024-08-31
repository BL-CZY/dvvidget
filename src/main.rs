pub mod cli;
pub mod daemon;
pub mod utils;

use clap::Parser;
use cli::args::{self, Args, DaemonSubCmd};
use daemon::structs::DaemonCmd;

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
                    DaemonSubCmd::Start => {
                        if let Err(e) = daemon::start_daemon(path) {
                            println!("Error starting the daemon: {:?}", e)
                        };
                    }
                    DaemonSubCmd::Shutdown => {
                        if let Err(e) = cli::send_evt(DaemonCmd::ShutDown) {
                            println!("Error sending event: {:?}", e)
                        }
                    }
                }
            }
        }

        args::Command::Volume { actions } => {
            let evt = match actions {
                args::VolCmd::Get => DaemonCmd::GetVol,
                args::VolCmd::Set { value } => DaemonCmd::SetVol(value),
                args::VolCmd::Inc { value } => DaemonCmd::IncVol(value),
                args::VolCmd::Dec { value } => DaemonCmd::DecVol(value),
            };
            if let Err(e) = cli::send_evt(evt) {
                println!("Err Sending event: {:?}", e);
            }
        }
    }
}
