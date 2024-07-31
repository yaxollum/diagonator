use crate::config::DiagonatorConfig;
use crate::manager::{CurrentInfo, DiagonatorManager, DiagonatorManagerConfig};
use crate::time::{Duration, HourMinute, Timestamp};
use axum::routing::post;
use axum::Json;
use serde::{Deserialize, Serialize};
use socketioxide::{extract::SocketRef, SocketIo};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Request {
    UnlockTimer,
    LockTimer,
    GetInfo,
    CompleteRequirement { id: u64 },
    AddRequirement { name: String, due: HourMinute },
    Deactivate { duration: Duration },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Response {
    Success,
    Error { msg: String },
    Info { info: CurrentInfo },
}

pub async fn launch_server(config: DiagonatorConfig) {
    eprintln!("Starting server with config {:?}", config);
    let (layer, io) = SocketIo::new_layer();

    let manager_config = DiagonatorManagerConfig {
        requirements: config.requirements.unwrap_or(Vec::new()),
        locked_time_ranges: config.locked_time_ranges.unwrap_or(Vec::new()),
        work_period_duration: Duration::from_minutes(config.work_period_minutes),
        break_duration: Duration::from_minutes(config.break_minutes),
    };
    let manager = Box::leak(Box::new(Mutex::new(DiagonatorManager::new(
        manager_config,
        Timestamp::now(),
    ))));
    eprintln!("Listening for connections.");
    io.ns("/", |s: SocketRef| {
        s.emit(
            "info_update",
            manager.lock().unwrap().get_info(Timestamp::now()),
        )
        .ok();
    });

    let app = axum::Router::new()
        .route(
            "/",
            post(|Json(request): Json<Request>| async {
                let mut manager = manager.lock().unwrap();
                let response = match request {
                    Request::UnlockTimer => manager.unlock_timer(Timestamp::now()),
                    Request::LockTimer => manager.lock_timer(Timestamp::now()),
                    Request::GetInfo => manager.get_info(Timestamp::now()),
                    Request::CompleteRequirement { id } => {
                        manager.complete_requirement(Timestamp::now(), id)
                    }
                    Request::AddRequirement { name, due } => {
                        manager.add_requirement(Timestamp::now(), name, due)
                    }
                    Request::Deactivate { duration } => {
                        manager.deactivate(Timestamp::now(), duration)
                    }
                };
                Json(response)
            }),
        )
        .layer(layer);

    let listener = tokio::net::TcpListener::bind(config.bind_on).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
