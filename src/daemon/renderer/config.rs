use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

use super::window::WindowDescriptor;

pub const DEFAULT_CSS_PATH: &str = "/usr/share/dvvidget/style.css";
pub const DEFAULT_VOL_CMD: VolCmdProvider = VolCmdProvider::Wpctl;
pub const DEFAULT_BRI_CMD: BriCmdProvider = BriCmdProvider::Builtin;

#[derive(Clone, Deserialize, Debug)]
pub struct AppConf {
    pub general: AppConfGeneral,
    pub vol: AppConfVol,
    pub bri: AppConfBri,
    pub dvoty: AppConfDvoty,
}

#[serde_inline_default]
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AppConfGeneral {
    #[serde_inline_default("/usr/share/dvvidget/style.css".to_string())]
    pub css_path: String,
}

#[derive(Clone, Debug)]
pub enum VolCmdProvider {
    Wpctl,
    NoCmd,
}

impl<'de> Deserialize<'de> for VolCmdProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "Wpctl" | "wpctl" => DEFAULT_VOL_CMD,
            "none" | "None" => VolCmdProvider::NoCmd,
            _ => DEFAULT_VOL_CMD,
        })
    }
}

#[serde_inline_default]
#[derive(Clone, Debug, Deserialize)]
pub struct IconDescriptor {
    #[serde_inline_default(0.0f64)]
    pub lower: f64,
    #[serde_inline_default(0.0f64)]
    pub upper: f64,
    #[serde_inline_default("".into())]
    pub icon: String,
}

impl IconDescriptor {
    pub fn from_val(bottom: f64, top: f64, icon: &str) -> Self {
        IconDescriptor {
            lower: bottom,
            upper: top,
            icon: icon.to_string(),
        }
    }
}

#[serde_inline_default]
#[derive(Clone, Deserialize, Debug)]
pub struct AppConfVol {
    #[serde_inline_default(true)]
    pub enable: bool,
    #[serde_inline_default(WindowDescriptor {anchor_bottom: true, margin_bottom: 130, ..Default::default()})]
    pub window: WindowDescriptor,
    #[serde_inline_default(100f64)]
    pub max_vol: f64,
    #[serde_inline_default(DEFAULT_VOL_CMD)]
    pub run_cmd: VolCmdProvider,
    #[serde_inline_default(false)]
    pub use_svg: bool,
    #[serde_inline_default(
        vec![
            IconDescriptor::from_val(0f64, 19f64, " "),
            IconDescriptor::from_val(20f64, 59f64, " "),
            IconDescriptor::from_val(60f64, 100f64, " "),
        ]
    )]
    pub icons: Vec<IconDescriptor>,
    #[serde_inline_default(" ".into())]
    pub mute_icon: String,
}

#[derive(Clone, Debug)]
pub enum BriCmdProvider {
    Builtin,
    NoCmd,
}

impl<'de> Deserialize<'de> for BriCmdProvider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "Builtin" | "builtin" => BriCmdProvider::Builtin,
            "None" | "none" => BriCmdProvider::NoCmd,
            _ => DEFAULT_BRI_CMD,
        })
    }
}

#[serde_inline_default]
#[derive(Clone, Deserialize, Debug)]
pub struct AppConfBri {
    #[serde_inline_default(true)]
    pub enable: bool,
    #[serde_inline_default(WindowDescriptor {anchor_bottom: true, margin_bottom: 130, ..Default::default()})]
    pub window: WindowDescriptor,
    #[serde_inline_default(DEFAULT_BRI_CMD)]
    pub run_cmd: BriCmdProvider,
    #[serde_inline_default(false)]
    pub use_svg: bool,
    #[serde_inline_default(vec![
        IconDescriptor::from_val(0f64, 19f64, "0"),
        IconDescriptor::from_val(20f64, 59f64, "1"),
        IconDescriptor::from_val(60f64, 100f64, "2"),
    ])]
    pub icons: Vec<IconDescriptor>,
}

#[derive(Clone, Default, Debug)]
pub enum SearchEngine {
    #[default]
    Google,
    Duckduckgo,
    Bing,
    Wikipedia(String),
}

impl<'de> Deserialize<'de> for SearchEngine {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_string(&s))
    }
}

impl SearchEngine {
    pub fn from_string(input: &str) -> Self {
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

#[serde_inline_default]
#[derive(Clone, Deserialize, Debug)]
pub struct AppConfDvoty {
    #[serde_inline_default(true)]
    pub enable: bool,
    #[serde_inline_default(WindowDescriptor::default())]
    pub window: WindowDescriptor,
    #[serde_inline_default(300)]
    pub max_height: u32,
    #[serde_inline_default("".into())]
    pub instruction_icon: String,
    #[serde_inline_default("".into())]
    pub math_icon: String,
    #[serde_inline_default("".into())]
    pub serach_icon: String,
    #[serde_inline_default("".into())]
    pub cmd_icon: String,
    #[serde_inline_default("".into())]
    pub url_icon: String,
    #[serde_inline_default(SearchEngine::default())]
    pub search_engine: SearchEngine,
    //pub terminal_exec: String,
}

impl Default for AppConf {
    fn default() -> Self {
        AppConf {
            general: AppConfGeneral {
                css_path: DEFAULT_CSS_PATH.to_string(),
            },
            vol: AppConfVol {
                enable: true,
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
                enable: true,
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
                enable: true,
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

fn default_bri_icons() -> Vec<IconDescriptor> {
    vec![
        IconDescriptor::from_val(0f64, 19f64, "0"),
        IconDescriptor::from_val(20f64, 59f64, "1"),
        IconDescriptor::from_val(60f64, 100f64, "2"),
    ]
}

pub fn default_config_path() -> PathBuf {
    if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(&val);
        path.push("dvvidget/config.toml");
        path
    } else if let Ok(val) = std::env::var("HOME") {
        let mut path = PathBuf::from(&val);
        path.push(".config/dvvidget/config.toml");
        path
    } else {
        println!("Failed to get config directory");
        PathBuf::new()
    }
}

//fn parse_config(content: &str) -> AppConf {
//    let toml = match content.parse::<Table>() {
//        Ok(res) => res,
//        Err(e) => {
//            println!("Err trying to parse the config into toml: {}", e);
//            return AppConf::default();
//        }
//    };
//
//    AppConf::from_toml(&toml)
//}
//
pub fn read_config(target_path: &PathBuf) -> AppConf {
    match std::fs::read_to_string(target_path) {
        Ok(val) => {
            println!("there is a config");
            toml::from_str(&val).unwrap_or_else(|_| AppConf::default())
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
