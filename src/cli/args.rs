use crate::daemon::structs::{Bri, DaemonCmdType, Vol};
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
        #[clap(short, long = "path")]
        socket_path: Option<String>,
        #[clap(short, long = "config")]
        config_path: Option<String>,
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

    #[clap(about = "Configure dvoty")]
    Dvoty {
        #[clap(subcommand)]
        actions: DvotyCmd,
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

#[derive(Subcommand)]
pub enum DvotyCmd {
    #[clap(about = "Open dvoty")]
    Open,
    #[clap(about = "Close dvoty")]
    Close,
    #[clap(about = "Toggle dvoty")]
    Toggle,
}

fn daemon_args(
    config_path: Option<String>,
    socket_path: Option<String>,
    option: Option<DaemonSubCmd>,
) {
    if option.is_none() {
        if let Err(e) = crate::daemon::start_daemon(config_path, socket_path) {
            println!("Error starting the daemon: {:?}", e)
        };
    } else {
        match option.unwrap() {
            DaemonSubCmd::Start => {
                if let Err(e) = crate::daemon::start_daemon(config_path, socket_path) {
                    println!("Error starting the daemon: {:?}", e)
                };
            }
            DaemonSubCmd::Shutdown => {
                if let Err(e) = crate::cli::send_evt(DaemonCmdType::ShutDown) {
                    println!("Error sending event: {:?}", e)
                }
            }
        }
    }
}

fn volume_args(actions: VolCmd) {
    let evt = match actions {
        VolCmd::SetMute { value } => match value {
            Some(val) => DaemonCmdType::Vol(Vol::SetMute(val)),
            None => DaemonCmdType::Vol(Vol::ToggleMute),
        },
        VolCmd::GetMute => DaemonCmdType::Vol(Vol::GetMute),
        VolCmd::Get => DaemonCmdType::Vol(Vol::Get),
        VolCmd::SetRough { value } => DaemonCmdType::Vol(Vol::SetRough(value as f64)),
        VolCmd::Set { value } => DaemonCmdType::Vol(Vol::Set(value as f64)),
        VolCmd::Inc { value } => DaemonCmdType::Vol(Vol::Inc(value as f64)),
        VolCmd::Dec { value } => DaemonCmdType::Vol(Vol::Dec(value as f64)),
        VolCmd::Close => DaemonCmdType::Vol(Vol::Close),
        VolCmd::Open { time } => {
            if let Some(t) = time {
                DaemonCmdType::Vol(Vol::OpenTimed(t))
            } else {
                DaemonCmdType::Vol(Vol::Open)
            }
        }
    };
    if let Err(e) = crate::cli::send_evt(evt) {
        println!("Err Sending event: {:?}", e);
    }
}

fn bri_args(actions: BriCmd) {
    let evt = match actions {
        BriCmd::Get => DaemonCmdType::Bri(Bri::Get),
        BriCmd::SetRough { value } => DaemonCmdType::Bri(Bri::SetRough(value as f64)),
        BriCmd::Set { value } => DaemonCmdType::Bri(Bri::Set(value as f64)),
        BriCmd::Inc { value } => DaemonCmdType::Bri(Bri::Inc(value as f64)),
        BriCmd::Dec { value } => DaemonCmdType::Bri(Bri::Dec(value as f64)),
        BriCmd::Close => DaemonCmdType::Bri(Bri::Close),
        BriCmd::Open { time } => {
            if let Some(t) = time {
                DaemonCmdType::Bri(Bri::OpenTimed(t))
            } else {
                DaemonCmdType::Bri(Bri::Open)
            }
        }
    };
    if let Err(e) = crate::cli::send_evt(evt) {
        println!("Err Sending event: {:?}", e);
    }
}

fn dvoty_args(actions: DvotyCmd) {
    crate::cli::send_evt(match actions {
        DvotyCmd::Open => DaemonCmdType::Dvoty(crate::daemon::structs::Dvoty::Open),
        DvotyCmd::Close => DaemonCmdType::Dvoty(crate::daemon::structs::Dvoty::Close),
        DvotyCmd::Toggle => DaemonCmdType::Dvoty(crate::daemon::structs::Dvoty::Toggle),
    })
    .unwrap_or_else(|e| println!("Error seding event: {:?}", e));
}

pub fn handle_args(args: Args) {
    match args.commands {
        Command::Daemon {
            socket_path,
            config_path,
            option,
        } => {
            daemon_args(config_path, socket_path, option);
        }

        Command::Volume { actions } => {
            volume_args(actions);
        }

        Command::Brightness { actions } => {
            bri_args(actions);
        }

        Command::Dvoty { actions } => {
            dvoty_args(actions);
        }
    }
}
