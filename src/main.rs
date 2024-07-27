pub mod cli;
pub mod daemon;
pub mod utils;

use clap::Parser;
use cli::args::{self, Args};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.commands {
        args::Command::Daemon { path } => {
            if let Err(e) = daemon::start_daemon(path).await {
                println!("Error starting the daemon: {:?}", e)
            };
        }

        args::Command::Connect => {
            if let Err(e) = cli::client::connect_server().await {
                println!("Error running the client: {:?}", e);
            }
        }
    }
}
