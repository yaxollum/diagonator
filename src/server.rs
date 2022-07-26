use crate::config::DiagonatorConfig;
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
    ListenForInfoChange,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Response {
    Success,
    Error(String),
    Info(CurrentInfo),
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

#[derive(Serialize, Deserialize, Debug)]
enum CurrentState {
    Active(u64),
    Locked(u64),
    Unlockable,
}

#[derive(Serialize, Deserialize, Debug)]
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
    fn start_session(&mut self) -> Response {
        Response::Success
    }
    fn end_session(&mut self) -> Response {
        Response::Success
    }
    fn get_info(&mut self) -> Response {
        Response::Success
    }
    fn complete_requirement(&mut self, requirement_id: u64) -> Response {
        Response::Success
    }
}

fn send_response(stream: &mut UnixStream, response: Response) {
    writeln!(
        stream,
        "{}",
        serde_json::to_string(&response).expect("Failed to serialize response")
    )
    .expect("Failed to send response to client");
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
    let mut manager = DiagonatorManager {
        current_info: CurrentInfo::new(),
        work_period_duration: config.work_period_minutes * 60,
        break_duration: config.break_minutes * 60,
    };
    eprintln!("Listening for connections.");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                eprintln!("Incoming connection received!");
                let mut buf_stream = BufReader::new(&stream);
                let mut data = String::new();
                buf_stream
                    .read_line(&mut data)
                    .expect("Failed to read client request from socket");
                match serde_json::from_str::<Request>(&data) {
                    Ok(request) => {
                        let response = match request {
                            Request::StartSession => manager.start_session(),
                            Request::EndSession => manager.end_session(),
                            Request::GetInfo => manager.get_info(),
                            Request::CompleteRequirement { id } => manager.complete_requirement(id),
                            Request::ListenForInfoChange => Response::Success,
                        };
                        send_response(&mut stream, response);
                        if request == Request::ListenForInfoChange {
                            // TODO: wait to be notified here
                        }
                    }
                    Err(err) => {
                        send_response(&mut stream, Response::Error("Invalid request".to_owned()));
                    }
                };
                eprintln!("DONE")
            }
            Err(err) => {
                eprintln!("Incoming connection failed with error '{}'", err);
            }
        }
    }
    Ok(())
}
