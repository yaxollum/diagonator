use crate::config::DiagonatorConfig;
use crate::time::unix_time;
use crate::time::DiagonatorDate;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::Mutex;
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type")]
enum Request {
    StartSession,
    EndSession,
    GetInfo,
    CompleteRequirement { id: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Response {
    Success,
    Error { msg: String },
    Info { info: CurrentInfo },
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum CurrentState {
    Active(u64),
    Locked(u64),
    Unlockable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CurrentInfo {
    // incomplete_requirements: Vec<RequirementInfo>,
    // events: Vec<EventInfo>,
    state: CurrentState,
}

impl CurrentInfo {
    fn new() -> Self {
        Self {
            state: CurrentState::Unlockable,
        }
    }
}

struct DiagonatorManager {
    current_info: CurrentInfo,
    // current_date: DiagonatorDate,
    work_period_duration: u64,
    break_duration: u64,
}

impl DiagonatorManager {
    fn start_session(&mut self, current_time: u64) -> Response {
        self.refresh(current_time);
        match self.current_info.state {
            CurrentState::Unlockable => {
                self.current_info.state =
                    CurrentState::Active(current_time + self.work_period_duration);
                self.refresh(current_time);
                Response::Success
            }
            CurrentState::Active(_) => Response::Error {
                msg: "Session is already active".to_owned(),
            },
            CurrentState::Locked(_) => Response::Error {
                msg: "Session is locked".to_owned(),
            },
        }
    }
    fn end_session(&mut self, current_time: u64) -> Response {
        self.refresh(current_time);
        match self.current_info.state {
            CurrentState::Active(_) => {
                self.current_info.state = CurrentState::Locked(current_time + self.break_duration);
                self.refresh(current_time);
                Response::Success
            }
            _ => Response::Error {
                msg: "Session is not active".to_owned(),
            },
        }
    }
    fn get_info(&mut self, current_time: u64) -> Response {
        self.refresh(current_time);
        Response::Info {
            info: self.current_info.clone(),
        }
    }
    fn complete_requirement(&mut self, requirement_id: u64) -> Response {
        Response::Success
    }
    fn refresh(&mut self, current_time: u64) {
        if let CurrentState::Active(time) = self.current_info.state {
            if current_time >= time {
                self.current_info.state = CurrentState::Locked(time + self.break_duration);
            }
        }
        if let CurrentState::Locked(time) = self.current_info.state {
            if current_time >= time {
                self.current_info.state = CurrentState::Unlockable;
            }
        }
    }
}

enum ClientHandlingError {
    SerializeError(serde_json::Error),
    SendResponseError,
    ReadRequestError,
}

impl Display for ClientHandlingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializeError(err) => write!(f, "Failed to serialize response: '{}'.", err),
            Self::ReadRequestError => write!(f, "Failed to read client request from socket."),
            Self::SendResponseError => write!(f, "Failed to send response to client"),
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
                        Request::StartSession => manager.start_session(unix_time()),
                        Request::EndSession => manager.end_session(unix_time()),
                        Request::GetInfo => manager.get_info(unix_time()),
                        Request::CompleteRequirement { id } => manager.complete_requirement(id),
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
    let manager = Arc::new(Mutex::new(DiagonatorManager {
        current_info: CurrentInfo::new(),
        work_period_duration: config.work_period_minutes * 60,
        break_duration: config.break_minutes * 60,
    }));
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
