use notify::Watcher;

use crate::utils::{get_paths, DaemonErr};

use super::renderer::dvoty::app_launcher;

pub fn start_file_server() -> Result<(), DaemonErr> {
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::event::Event>>();
    let mut watcher =
        notify::recommended_watcher(tx).map_err(|e| DaemonErr::FileWatchError(e.to_string()))?;

    get_paths().iter().for_each(|p| {
        let _ = watcher.watch(p, notify::RecursiveMode::Recursive);
    });

    app_launcher::process_paths();

    for res in rx {
        if let Ok(evt) = res {
            match evt.kind {
                notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                    app_launcher::process_paths();
                }

                _ => {}
            }
        }
    }

    Ok(())
}
