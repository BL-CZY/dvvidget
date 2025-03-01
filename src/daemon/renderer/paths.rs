use std::sync::{Arc, Mutex};

use freedesktop_file_parser::DesktopFile;
use once_cell::sync::OnceCell;

pub static DESKTOP_FILES: OnceCell<Arc<Mutex<Vec<DesktopFile>>>> = OnceCell::new();
