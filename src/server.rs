use crate::config::DiagonatorConfig;
use crate::time::DiagonatorDate;
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
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

enum Command {
    StartSession,
    EndSession,
    GetInfo,
    CompleteRequirement { id: u64 },
}

struct SimpleResponse {
    error_msg: String,
}

struct EventInfo {
    id: u64,
    name: String,
    time: u64,
}

struct RequirementInfo {
    id: u64,
    name: String,
    event_id: u64,
}

enum CurrentState {
    Active,
    Locked,
    Unlockable,
}

struct CurrentInfo {
    incomplete_requirements: Vec<RequirementInfo>,
    events: Vec<EventInfo>,
    state: CurrentState,
    state_end_time: u64,
}

struct DiagonatorManager {
    current_info: CurrentInfo,
    current_date: DiagonatorDate,
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
    eprintln!("Listening for connections.");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                eprintln!("Incoming connection received!");
                let mut stream = BufReader::new(stream);
                let mut data = String::new();
                stream
                    .read_line(&mut data)
                    .expect("Failed to read client request from socket");
                eprintln!("Read data: {}", data);
            }
            Err(err) => {
                eprintln!("Incoming connection failed with error '{}'", err);
            }
        }
    }
    Ok(())
}
