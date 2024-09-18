use toml::{map::Map, Table, Value};

use super::window::WindowDescriptor;

pub const DEFAULT_CSS_PATH: &str = "/usr/share/dvvidget/style.css";
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
pub struct IconDescriptor {
    pub range: (f64, f64),
    pub icon: String,
}

impl IconDescriptor {
    pub fn from_val(bottom: f64, top: f64, icon: &str) -> Self {
        IconDescriptor {
            range: (bottom, top),
            icon: icon.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AppConfVol {
    pub window: WindowDescriptor,
    pub max_vol: f64,
    pub run_cmd: VolCmdProvider,
    pub icons: Vec<IconDescriptor>,
    pub mute_icon: String,
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
                icons: vec![
                    IconDescriptor::from_val(0f64, 19f64, " "),
                    IconDescriptor::from_val(20f64, 59f64, " "),
                    IconDescriptor::from_val(60f64, 100f64, " "),
                ],
                mute_icon: " ".to_string(),
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

    fn default_icons() -> Vec<IconDescriptor> {
        vec![
            IconDescriptor::from_val(0f64, 19f64, " "),
            IconDescriptor::from_val(20f64, 59f64, " "),
            IconDescriptor::from_val(60f64, 100f64, " "),
        ]
    }

    fn read_icon_table(vec: &Vec<Value>) -> Vec<IconDescriptor> {
        let mut result = vec![];
        for val in vec.iter() {
            if let Value::Table(tbl) = val {
                let lower: i64 = tbl
                    .get("lower")
                    .unwrap_or(&Value::Integer(0))
                    .as_integer()
                    .unwrap_or(0i64);

                let upper: i64 = tbl
                    .get("upper")
                    .unwrap_or(&Value::Integer(0))
                    .as_integer()
                    .unwrap_or(0i64);

                let icon: String = tbl
                    .get("icon")
                    .unwrap_or(&Value::String("".into()))
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                result.push(IconDescriptor {
                    range: (lower as f64, upper as f64),
                    icon,
                });
            }
        }
        result
    }

    fn icons(toml: &Map<String, Value>) -> Vec<IconDescriptor> {
        let inner = if let Some(v) = toml.get("volume") {
            v
        } else {
            return AppConf::default_icons();
        };

        if let Some(Value::Array(vec)) = inner.get("icons") {
            AppConf::read_icon_table(vec)
        } else {
            AppConf::default_icons()
        }
    }

    fn mute_icon(toml: &Map<String, Value>) -> String {
        let inner = if let Some(v) = toml.get("volume") {
            v
        } else {
            return "".into();
        };

        inner
            .get("mute_icon")
            .unwrap_or(&Value::String("".into()))
            .as_str()
            .unwrap_or("")
            .to_string()
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
                icons: AppConf::icons(toml),
                mute_icon: AppConf::mute_icon(toml),
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
