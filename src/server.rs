use crate::config::DiagonatorConfig;
use crate::manager::{CurrentInfo, DiagonatorManager, DiagonatorManagerConfig};
use crate::simulator::SimulatorError;
use crate::time::{Duration, Timestamp};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Request {
    UnlockTimer,
    LockTimer,
    GetInfo,
    CompleteRequirement { id: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Response {
    Success,
    Error { msg: String },
    Info { info: CurrentInfo },
}

pub enum ClientHandlingError {
    SerializeError(serde_json::Error),
    SendResponseError,
    ReadRequestError,
    SimulatorError(SimulatorError),
}

impl Display for ClientHandlingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializeError(err) => write!(f, "Failed to serialize response: '{}'.", err),
            Self::ReadRequestError => write!(f, "Failed to read client request from socket."),
            Self::SendResponseError => write!(f, "Failed to send response to client"),
            Self::SimulatorError(err) => write!(f, "Simulator failed with error: '{:?}'", err),
        }
    }
}

fn send_response(stream: &mut UnixStream, response: Response) -> Result<(), ClientHandlingError> {
    writeln!(
        stream,
        "{}",
        serde_json::to_string(&response).map_err(|err| ClientHandlingError::SerializeError(err))?
    )
    .map_err(|_| ClientHandlingError::SendResponseError)
}

fn handle_client_inner(
    mut stream: UnixStream,
    manager: Arc<Mutex<DiagonatorManager>>,
) -> Result<(), ClientHandlingError> {
    loop {
        let mut buf_stream = BufReader::new(&stream);
        let mut data = String::new();
        buf_stream
            .read_line(&mut data)
            .map_err(|_| ClientHandlingError::ReadRequestError)?;
        match serde_json::from_str::<Request>(&data) {
            Ok(request) => {
                let response = {
                    let mut manager = manager.lock().unwrap();
                    match request {
                        Request::UnlockTimer => manager.unlock_timer(Timestamp::now())?,
                        Request::LockTimer => manager.lock_timer(Timestamp::now())?,
                        Request::GetInfo => manager.get_info(Timestamp::now())?,
                        Request::CompleteRequirement { id } => {
                            manager.complete_requirement(Timestamp::now(), id)?
                        }
                    }
                };
                send_response(&mut stream, response)?;
            }
            Err(err) => {
                send_response(
                    &mut stream,
                    Response::Error {
                        msg: format!("Invalid request: {}", err),
                    },
                )?;
            }
        };
    }
}

fn handle_client(stream: UnixStream, manager: Arc<Mutex<DiagonatorManager>>) {
    eprintln!("Incoming connection received!");
    if let Err(err) = handle_client_inner(stream, manager) {
        eprintln!("{}", err);
    }
    eprintln!("Finished handling client.")
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

    let manager_config = DiagonatorManagerConfig {
        diagonator_command: (config.diagonator_path, config.diagonator_args),
        requirements: config.requirements.unwrap_or(Vec::new()),
        locked_time_ranges: config.locked_time_ranges.unwrap_or(Vec::new()),
        work_period_duration: Duration::from_minutes(config.work_period_minutes),
        break_duration: Duration::from_minutes(config.break_minutes),
    };
    let manager = Arc::new(Mutex::new(DiagonatorManager::new(manager_config)));
    eprintln!("Listening for connections.");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let manager = manager.clone();
                thread::spawn(move || handle_client(stream, manager));
            }
            Err(err) => {
                eprintln!("Incoming connection failed with error '{}'", err);
            }
        }
    }
    Ok(())
}
