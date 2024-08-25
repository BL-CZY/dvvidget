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
        #[clap(subcommand)]
        option: Option<DaemonCmd>,
    },

    #[clap(about = "Connect to the daemon")]
    Volume {
        #[clap(short, long)]
        value: u32,
    },
}

#[derive(Subcommand)]
pub enum DaemonCmd {
    #[clap(about = "Start the daemon")]
    Start,
    #[clap(about = "Shutdown the daemon")]
    Shutdown,
}
