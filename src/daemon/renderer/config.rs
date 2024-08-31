use toml::{map::Map, Table, Value};

use super::window::WindowDescriptor;

pub const DEFAULT_CSS_PATH: &str = "/home/tpl/projects/dvvidget/src/daemon/renderer/style.css";

pub struct AppConf {
    general: AppConfGeneral,
    vol: AppConfVol,
}

pub struct AppConfGeneral {
    css_path: String,
}

pub struct AppConfVol {
    window: WindowDescriptor,
    max_vol: f64,
    set_vol_cmd: Vec<String>,
    get_vol_cmd: Vec<String>,
    inc_vol_cmd: Vec<String>,
    dec_vol_cmd: Vec<String>,
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

    fn max_vol() -> f64 {
        0.0
    }
    fn set_vol_cmd() -> Vec<String> {
        vec![]
    }
    fn get_vol_cmd() -> Vec<String> {
        vec![]
    }
    fn inc_vol_cmd() -> Vec<String> {
        vec![]
    }
    fn dec_vol_cmd() -> Vec<String> {
        vec![]
    }
    fn window() -> WindowDescriptor {
        WindowDescriptor::new()
    }

    pub fn from_toml(toml: &Map<String, Value>) -> Self {
        AppConf {
            general: AppConfGeneral {
                css_path: AppConf::css_path(&toml),
            },
            vol: AppConfVol {
                window: AppConf::window(),
                max_vol: AppConf::max_vol(),
                set_vol_cmd: AppConf::set_vol_cmd(),
                get_vol_cmd: AppConf::get_vol_cmd(),
                inc_vol_cmd: AppConf::inc_vol_cmd(),
                dec_vol_cmd: AppConf::dec_vol_cmd(),
            },
        }
    }
}

fn append_path(target: &str, append: &str) -> String {
    if !target.ends_with("/") {
        return target.to_owned() + "/" + append;
    } else {
        target.to_owned() + append
    }
}

fn default_config_path() -> String {
    let result = if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
        append_path(&val, "dvvidget/config.toml")
    } else {
        if let Ok(val) = std::env::var("HOME") {
            append_path(&val, ".config/dvvidget/config.toml")
        } else {
            println!("Failed to get config directory");
            "".into()
        }
    };

    result
}

fn parse_condig(content: &str) -> Result<(), ()> {
    let toml = match content.parse::<Table>() {
        Ok(res) => res,
        Err(e) => {
            println!("Err trying to parse the config into toml: {}", e);
            return Err(());
        }
    };

    AppConf::from_toml(&toml);

    Ok(())
}

pub fn read_config(path: Option<String>) {
    let target_path: String = if let Some(p) = path {
        p
    } else {
        default_config_path()
    };

    match std::fs::read_to_string(&target_path) {
        Ok(val) => {
            println!("there is a config");
            if let Err(_) = parse_condig(&val) {
                return;
            }
        }
        Err(e) => {
            println!(
                "Failed to get the config from path: {:?}\nErr: {:?}, go with default",
                target_path, e
            );
        }
    }
}
