pub mod cli;
pub mod daemon;
pub mod utils;

use clap::Parser;
use cli::args::{self, Args};

fn main() {
    let args = Args::parse();

    match args.commands {
        args::Command::Daemon { path } => {
            if let Err(e) = daemon::start_daemon(path) {
                println!("Error starting the daemon: {}", e)
            };
        }
    }
}
