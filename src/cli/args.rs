use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub commands: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[clap(about = "Start the daemon and the graphics")]
    Daemon {
        #[clap(short, long)]
        path: Option<String>,
    },

    #[clap(about = "Connect to the daemon")]
    Connect,
}
