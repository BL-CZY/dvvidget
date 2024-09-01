use toml::{map::Map, Table, Value};

use super::window::WindowDescriptor;

pub const DEFAULT_CSS_PATH: &str = "/home/tpl/projects/dvvidget/src/daemon/renderer/style.css";
pub const DEFAULT_VOL_CMD: VolCmdProvider = VolCmdProvider::Wpctl;

#[derive(Clone)]
pub struct AppConf {
    pub general: AppConfGeneral,
    pub vol: AppConfVol,
}

#[derive(Clone)]
pub struct AppConfGeneral {
    pub css_path: String,
}

#[derive(Clone)]
pub enum VolCmdProvider {
    Wpctl,
    NoCmd,
}

#[derive(Clone)]
pub struct AppConfVol {
    pub window: WindowDescriptor,
    pub max_vol: f64,
    pub run_cmd: VolCmdProvider,
}

impl Default for AppConf {
    fn default() -> Self {
        AppConf {
            general: AppConfGeneral {
                css_path: DEFAULT_CSS_PATH.to_string(),
            },
            vol: AppConfVol {
                window: WindowDescriptor {
                    anchor_bottom: true,
                    margin_bottom: 130,
                    ..Default::default()
                },
                max_vol: 100f64,
                run_cmd: DEFAULT_VOL_CMD,
            },
        }
    }
}

impl AppConf {
    fn css_path(toml: &Map<String, Value>) -> String {
        let default = Value::String(DEFAULT_CSS_PATH.to_string());
        let result = toml
            .get("general")
            .unwrap_or(&default)
            .get("css_path")
            .unwrap_or(&default);

        match result {
            Value::String(val) => val.to_string(),
            _ => DEFAULT_CSS_PATH.to_string(),
        }
    }

    fn max_vol(toml: &Map<String, Value>) -> f64 {
        let inner = if let Some(v) = toml.get("volume") {
            v
        } else {
            return 100f64;
        };

        if let Some(Value::Integer(v)) = inner.get("max_vol") {
            *v as f64
        } else {
            100f64
        }
    }

    fn run_cmd(toml: &Map<String, Value>) -> VolCmdProvider {
        let inner = if let Some(v) = toml.get("volume") {
            v
        } else {
            return VolCmdProvider::Wpctl;
        };

        if let Some(Value::String(val)) = inner.get("run_cmd") {
            match val.as_str() {
                "Wpctl" | "wpctl" => DEFAULT_VOL_CMD,
                "none" | "None" => VolCmdProvider::NoCmd,
                _ => DEFAULT_VOL_CMD,
            }
        } else {
            DEFAULT_VOL_CMD
        }
    }

    fn vol_window(toml: &Map<String, Value>) -> WindowDescriptor {
        WindowDescriptor::vol_from_toml(toml)
    }

    pub fn from_toml(toml: &Map<String, Value>) -> Self {
        AppConf {
            general: AppConfGeneral {
                css_path: AppConf::css_path(toml),
            },
            vol: AppConfVol {
                window: AppConf::vol_window(toml),
                max_vol: AppConf::max_vol(toml),
                run_cmd: AppConf::run_cmd(toml),
            },
        }
    }
}

fn append_path(target: &str, append: &str) -> String {
    if !target.ends_with("/") {
        target.to_owned() + "/" + append
    } else {
        target.to_owned() + append
    }
}

fn default_config_path() -> String {
    if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
        append_path(&val, "dvvidget/config.toml")
    } else if let Ok(val) = std::env::var("HOME") {
        append_path(&val, ".config/dvvidget/config.toml")
    } else {
        println!("Failed to get config directory");
        "".into()
    }
}

fn parse_config(content: &str) -> AppConf {
    let toml = match content.parse::<Table>() {
        Ok(res) => res,
        Err(e) => {
            println!("Err trying to parse the config into toml: {}", e);
            return AppConf::default();
        }
    };

    AppConf::from_toml(&toml)
}

pub fn read_config(path: Option<String>) -> AppConf {
    let target_path: String = if let Some(p) = path {
        p
    } else {
        default_config_path()
    };

    match std::fs::read_to_string(&target_path) {
        Ok(val) => {
            println!("there is a config");
            parse_config(&val)
        }
        Err(e) => {
            println!(
                "Failed to get the config from path: {:?}\nErr: {:?}, go with default",
                target_path, e
            );

            AppConf::default()
        }
    }
}
