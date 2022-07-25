use crate::config::DiagonatorConfig;
use std::fmt::Display;
use std::fs;
use std::io;
use std::os::unix::net::UnixListener;

pub enum ServerError {
    SocketListenError(String, io::Error),
    UnlinkSocketError(String, io::Error),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SocketListenError(path, err) => write!(
                f,
                "Received error '{:?}' when trying to listen on socket {}",
                err.kind(),
                path
            ),
            Self::UnlinkSocketError(path, err) => write!(
                f,
                "Received error '{:?}' when trying to unlink socket {}",
                err, path
            ),
        }
    }
}

pub fn launch_server(config: DiagonatorConfig) -> Result<(), ServerError> {
    eprintln!("Starting server with config {:?}", config);
    if let Err(err) = fs::remove_file(&config.socket_path) {
        if err.kind() != io::ErrorKind::NotFound {
            return Err(ServerError::UnlinkSocketError(config.socket_path, err));
        }
    }
    let listener = UnixListener::bind(&config.socket_path)
        .map_err(|err| ServerError::SocketListenError(config.socket_path.clone(), err))?;
    Ok(())
}
