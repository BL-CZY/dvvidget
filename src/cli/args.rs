use crate::daemon::structs::{Bri, DaemonCmd, Vol};
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

    #[clap(about = "Configure the volume panel")]
    Volume {
        #[clap(subcommand)]
        actions: VolCmd,
    },

    #[clap(about = "Configure the brightness panel")]
    Brightness {
        #[clap(subcommand)]
        actions: BriCmd,
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
pub enum BriCmd {
    #[clap(about = "Set brightness")]
    SetRough { value: u32 },
    #[clap(about = "Set brightness with murph effect")]
    Set { value: u32 },
    #[clap(about = "Get the current scale")]
    Get,
    #[clap(about = "Increase the brightness")]
    Inc { value: u32 },
    #[clap(about = "Decrease the brightness")]
    Dec { value: u32 },
    #[clap(about = "Hide the scale")]
    Close,
    #[clap(
        about = "Show the scale. If there is a number given, close the scale after the given number seconds"
    )]
    Open { time: Option<f64> },
}

#[derive(Subcommand)]
pub enum VolCmd {
    #[clap(about = "toggle/set mute")]
    SetMute { value: Option<bool> },
    #[clap(about = "get mute status")]
    GetMute,
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
    #[clap(about = "Hide the scale")]
    Close,
    #[clap(
        about = "Show the scale. If there is a number given, close the scale after the given number seconds"
    )]
    Open { time: Option<f64> },
}

fn daemon_args(path: Option<String>, option: Option<DaemonSubCmd>) {
    if option.is_none() {
        if let Err(e) = crate::daemon::start_daemon(path) {
            println!("Error starting the daemon: {:?}", e)
        };
    } else {
        match option.unwrap() {
            DaemonSubCmd::Start => {
                if let Err(e) = crate::daemon::start_daemon(path) {
                    println!("Error starting the daemon: {:?}", e)
                };
            }
            DaemonSubCmd::Shutdown => {
                if let Err(e) = crate::cli::send_evt(DaemonCmd::ShutDown) {
                    println!("Error sending event: {:?}", e)
                }
            }
        }
    }
}

fn volume_args(actions: VolCmd) {
    let evt = match actions {
        VolCmd::SetMute { value } => match value {
            Some(val) => DaemonCmd::Vol(Vol::SetMute(val)),
            None => DaemonCmd::Vol(Vol::ToggleMute),
        },
        VolCmd::GetMute => DaemonCmd::Vol(Vol::GetMute),
        VolCmd::Get => DaemonCmd::Vol(Vol::Get),
        VolCmd::SetRough { value } => DaemonCmd::Vol(Vol::SetRough(value as f64)),
        VolCmd::Set { value } => DaemonCmd::Vol(Vol::Set(value as f64)),
        VolCmd::Inc { value } => DaemonCmd::Vol(Vol::Inc(value as f64)),
        VolCmd::Dec { value } => DaemonCmd::Vol(Vol::Dec(value as f64)),
        VolCmd::Close => DaemonCmd::Vol(Vol::Close),
        VolCmd::Open { time } => {
            if let Some(t) = time {
                DaemonCmd::Vol(Vol::OpenTimed(t))
            } else {
                DaemonCmd::Vol(Vol::Open)
            }
        }
    };
    if let Err(e) = crate::cli::send_evt(evt) {
        println!("Err Sending event: {:?}", e);
    }
}

fn bri_args(actions: BriCmd) {
    let evt = match actions {
        BriCmd::Get => DaemonCmd::Bri(Bri::Get),
        BriCmd::SetRough { value } => DaemonCmd::Bri(Bri::SetRough(value as f64)),
        BriCmd::Set { value } => DaemonCmd::Bri(Bri::Set(value as f64)),
        BriCmd::Inc { value } => DaemonCmd::Bri(Bri::Inc(value as f64)),
        BriCmd::Dec { value } => DaemonCmd::Bri(Bri::Dec(value as f64)),
        BriCmd::Close => DaemonCmd::Bri(Bri::Close),
        BriCmd::Open { time } => {
            if let Some(t) = time {
                DaemonCmd::Bri(Bri::OpenTimed(t))
            } else {
                DaemonCmd::Bri(Bri::Open)
            }
        }
    };
    if let Err(e) = crate::cli::send_evt(evt) {
        println!("Err Sending event: {:?}", e);
    }
}

pub fn handle_args(args: Args) {
    match args.commands {
        Command::Daemon { path, option } => {
            daemon_args(path, option);
        }

        Command::Volume { actions } => {
            volume_args(actions);
        }

        Command::Brightness { actions } => {
            bri_args(actions);
        }
    }
}
