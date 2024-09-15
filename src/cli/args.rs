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
        option: Option<DaemonSubCmd>,
    },

    #[clap(about = "Connect to the daemon")]
    Volume {
        #[clap(subcommand)]
        actions: VolCmd,
    },
}

#[derive(Subcommand)]
pub enum DaemonSubCmd {
    #[clap(about = "Start the daemon")]
    Start,
    #[clap(about = "Shutdown the daemon")]
    Shutdown,
}

#[derive(Subcommand)]
pub enum VolCmd {
    #[clap(about = "Set the volume")]
    SetRough { value: u32 },
    #[clap(about = "Set the volume with murph effect")]
    Set { value: u32 },
    #[clap(about = "Get the current scale")]
    Get,
    #[clap(about = "Increase the volume")]
    Inc { value: u32 },
    #[clap(about = "Decrease the volume")]
    Dec { value: u32 },
    #[clap(about = "Close the scale")]
    Close,
    #[clap(
        about = "Show the scale. If there is a number given, close the scale after the given number seconds"
    )]
    Open { time: Option<f64> },
}
