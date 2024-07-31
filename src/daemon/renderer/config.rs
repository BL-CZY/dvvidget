use super::{app::AppDescriptor, dock::DockDescriptor};
use toml::{self, Value};

impl AppDescriptor {
    pub fn from_config(config: &str) -> Self {
        // let table: Value = if let Err(e) = toml::from_str(config) {};

        AppDescriptor {
            dock: DockDescriptor::new(),
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

pub fn read_config(path: Option<String>) -> AppDescriptor {
    let target_path: String = if let Some(p) = path {
        p
    } else {
        default_config_path()
    };

    match std::fs::read_to_string(target_path) {
        Ok(val) => AppDescriptor::from_config(&val),
        Err(e) => {
            println!("Failed to get the cconfig: {:?}, go with default", e);
            AppDescriptor::new()
        }
    }
}
