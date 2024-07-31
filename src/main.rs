mod config;
mod manager;
mod server;
mod simulator;
mod time;

use config::load_config;
use server::launch_server;

#[tokio::main]
async fn main() {
    match load_config() {
        Ok(config) => {
            launch_server(config).await;
        }
        Err(err) => {
            eprintln!("Encountered error when loading config: {}", err);
            std::process::exit(1);
        }
    }
}
