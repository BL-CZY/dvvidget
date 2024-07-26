pub mod info;
pub mod renderer;
pub mod server;

use crate::utils::DaemonErr;

pub async fn start_daemon(path: Option<String>) -> Result<(), DaemonErr> {
    if let Err(e) = server::start_server(path).await {
        return Err(e);
    }
    Ok(())
}
