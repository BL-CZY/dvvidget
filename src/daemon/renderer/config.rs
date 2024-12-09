use toml::{map::Map, Table, Value};

use super::window::WindowDescriptor;

pub const DEFAULT_CSS_PATH: &str = "/usr/share/dvvidget/style.css";
pub const DEFAULT_VOL_CMD: VolCmdProvider = VolCmdProvider::Wpctl;
pub const DEFAULT_BRI_CMD: BriCmdProvider = BriCmdProvider::Builtin;

#[derive(Clone)]
pub struct AppConf {
    pub general: AppConfGeneral,
    pub vol: AppConfVol,
    pub bri: AppConfBri,
    pub dvoty: AppConfDvoty,
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

#[derive(Clone, Debug)]
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
    pub enabled: bool,
    pub window: WindowDescriptor,
    pub max_vol: f64,
    pub run_cmd: VolCmdProvider,
    pub use_svg: bool,
    pub icons: Vec<IconDescriptor>,
    pub mute_icon: String,
}

#[derive(Clone)]
pub enum BriCmdProvider {
    Builtin,
    NoCmd,
}

#[derive(Clone)]
pub struct AppConfBri {
    pub enabled: bool,
    pub window: WindowDescriptor,
    pub run_cmd: BriCmdProvider,
    pub use_svg: bool,
    pub icons: Vec<IconDescriptor>,
}

#[derive(Clone, Default)]
pub enum SearchEngine {
    #[default]
    Google,
    Duckduckgo,
    Bing,
    Wikipedia(String),
}

impl SearchEngine {
    pub fn from_str(input: &str) -> Self {
        // check for Wikipedia
        let input = input.trim();
        if input.ends_with("wiki")
            || input.ends_with("WIKI")
            || input.ends_with("Wiki")
            || input.ends_with("wikipedia")
            || input.ends_with("Wikipedia")
        {
            if let Some(index) = input.find("_") {
                return SearchEngine::Wikipedia(input[0..index].to_string());
            } else {
                return SearchEngine::Wikipedia("en".to_string());
            }
        }

        match input {
            "Goog" | "google" | "Google" | "goog" => SearchEngine::Google,
            "DDG" | "ddg" | "Ddg" | "Duckduckgo" | "duckduckgo" => SearchEngine::Duckduckgo,
            "bing" | "Bing" => SearchEngine::Bing,
            _ => SearchEngine::Google,
        }
    }
}

#[derive(Clone)]
pub struct AppConfDvoty {
    pub enabled: bool,
    pub window: WindowDescriptor,
    pub max_height: u32,
    pub instruction_icon: String,
    pub math_icon: String,
    pub serach_icon: String,
    pub cmd_icon: String,
    pub url_icon: String,
    pub search_engine: SearchEngine,
}

impl Default for AppConf {
    fn default() -> Self {
        AppConf {
            general: AppConfGeneral {
                css_path: DEFAULT_CSS_PATH.to_string(),
            },
            vol: AppConfVol {
                enabled: true,
                window: WindowDescriptor {
                    anchor_bottom: true,
                    margin_bottom: 130,
                    ..Default::default()
                },
                max_vol: 100f64,
                run_cmd: DEFAULT_VOL_CMD,
                use_svg: false,
                icons: vec![
                    IconDescriptor::from_val(0f64, 19f64, " "),
                    IconDescriptor::from_val(20f64, 59f64, " "),
                    IconDescriptor::from_val(60f64, 100f64, " "),
                ],
                mute_icon: " ".to_string(),
            },
            bri: AppConfBri {
                enabled: true,
                window: WindowDescriptor {
                    anchor_bottom: true,
                    margin_bottom: 130,
                    ..Default::default()
                },
                use_svg: false,
                run_cmd: DEFAULT_BRI_CMD,
                icons: default_bri_icons(),
            },
            dvoty: AppConfDvoty {
                enabled: true,
                window: WindowDescriptor::default(),
                max_height: 300,
                instruction_icon: "".into(),
                math_icon: "".into(),
                serach_icon: "".into(),
                cmd_icon: "".into(),
                url_icon: "".into(),
                search_engine: SearchEngine::default(),
            },
        }
    }
}

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

fn vol_run_cmd(toml: &Map<String, Value>) -> VolCmdProvider {
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

fn bri_run_cmd(toml: &Map<String, Value>) -> BriCmdProvider {
    let inner = if let Some(v) = toml.get("brightness") {
        v
    } else {
        return DEFAULT_BRI_CMD;
    };

    if let Some(Value::String(val)) = inner.get("run_cmd") {
        match val.as_str() {
            "Builtin" | "builtin" => BriCmdProvider::Builtin,
            "None" | "none" => BriCmdProvider::NoCmd,
            _ => DEFAULT_BRI_CMD,
        }
    } else {
        DEFAULT_BRI_CMD
    }
}

fn vol_window(toml: &Map<String, Value>) -> WindowDescriptor {
    WindowDescriptor::from_toml(
        toml,
        "volume",
        WindowDescriptor {
            anchor_bottom: true,
            margin_bottom: 130,
            namespace: "dvvidget-vol".into(),
            ..Default::default()
        },
    )
}

fn bri_window(toml: &Map<String, Value>) -> WindowDescriptor {
    WindowDescriptor::from_toml(
        toml,
        "brightness",
        WindowDescriptor {
            anchor_bottom: true,
            margin_bottom: 130,
            namespace: "dvvidget-bri".into(),
            ..Default::default()
        },
    )
}

fn default_vol_icons() -> Vec<IconDescriptor> {
    vec![
        IconDescriptor::from_val(0f64, 19f64, " "),
        IconDescriptor::from_val(20f64, 59f64, " "),
        IconDescriptor::from_val(60f64, 100f64, " "),
    ]
}

fn default_bri_icons() -> Vec<IconDescriptor> {
    vec![
        IconDescriptor::from_val(0f64, 19f64, "0"),
        IconDescriptor::from_val(20f64, 59f64, "1"),
        IconDescriptor::from_val(60f64, 100f64, "2"),
    ]
}

fn read_icon_table(vec: &[Value]) -> Vec<IconDescriptor> {
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

fn vol_icons(toml: &Map<String, Value>) -> Vec<IconDescriptor> {
    let inner = if let Some(v) = toml.get("volume") {
        v
    } else {
        return default_vol_icons();
    };

    if let Some(Value::Array(vec)) = inner.get("icons") {
        read_icon_table(vec)
    } else {
        default_vol_icons()
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

fn is_svg(toml: &Map<String, Value>, key: &str) -> bool {
    let inner = if let Some(v) = toml.get(key) {
        v
    } else {
        return false;
    };

    inner
        .get("svg")
        .unwrap_or(&Value::Boolean(false))
        .as_bool()
        .unwrap_or(false)
}

fn bri_icons(toml: &Map<String, Value>) -> Vec<IconDescriptor> {
    let inner = if let Some(v) = toml.get("brightness") {
        v
    } else {
        return default_vol_icons();
    };

    if let Some(Value::Array(vec)) = inner.get("icons") {
        read_icon_table(vec)
    } else {
        default_vol_icons()
    }
}

fn max_height(toml: &Map<String, Value>) -> u32 {
    let inner = if let Some(v) = toml.get("dvoty") {
        v
    } else {
        return 300;
    };

    if let Some(Value::Integer(val)) = inner.get("max_height") {
        *val as u32
    } else {
        300
    }
}

fn dvoty_icon(toml: &Map<String, Value>, icon: &str) -> String {
    let inner = if let Some(v) = toml.get("dvoty") {
        v
    } else {
        return "".into();
    };

    if let Some(Value::String(val)) = inner.get(icon) {
        val.into()
    } else {
        "".into()
    }
}

fn search_engine(toml: &Map<String, Value>) -> SearchEngine {
    let inner = if let Some(v) = toml.get("dvoty") {
        v
    } else {
        return SearchEngine::default();
    };

    if let Some(Value::String(val)) = inner.get("search_engine") {
        SearchEngine::from_str(val)
    } else {
        SearchEngine::default()
    }
}

fn is_enabled(toml: &Map<String, Value>, key: &str) -> bool {
    let inner = if let Some(v) = toml.get(key) {
        v
    } else {
        return true;
    };

    if let Some(Value::Boolean(val)) = inner.get("enable") {
        *val
    } else {
        true
    }
}

impl AppConf {
    pub fn from_toml(toml: &Map<String, Value>) -> Self {
        AppConf {
            general: AppConfGeneral {
                css_path: css_path(toml),
            },
            vol: AppConfVol {
                enabled: is_enabled(toml, "volume"),
                window: vol_window(toml),
                max_vol: max_vol(toml),
                run_cmd: vol_run_cmd(toml),
                use_svg: is_svg(toml, "volume"),
                icons: vol_icons(toml),
                mute_icon: mute_icon(toml),
            },
            bri: AppConfBri {
                enabled: is_enabled(toml, "brightness"),
                window: bri_window(toml),
                run_cmd: bri_run_cmd(toml),
                use_svg: is_svg(toml, "brightness"),
                icons: bri_icons(toml),
            },
            dvoty: AppConfDvoty {
                enabled: is_enabled(toml, "dvoty"),
                window: WindowDescriptor::from_toml(
                    toml,
                    "dvoty",
                    WindowDescriptor {
                        //layer: gtk4_layer_shell::Layer::Top,
                        namespace: "dvvidget-dvoty".into(),
                        keyboard_mode: gtk4_layer_shell::KeyboardMode::OnDemand,
                        ..Default::default()
                    },
                ),
                url_icon: dvoty_icon(toml, "url_icon"),
                cmd_icon: dvoty_icon(toml, "cmd_icon"),
                serach_icon: dvoty_icon(toml, "search_icon"),
                math_icon: dvoty_icon(toml, "math_icon"),
                instruction_icon: dvoty_icon(toml, "instruction_icon"),
                max_height: max_height(toml),
                search_engine: search_engine(toml),
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
