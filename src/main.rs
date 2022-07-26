mod config;
mod server;
mod time;

use config::load_config;
use server::launch_server;

fn main() {
    match load_config() {
        Ok(config) => {
            if let Err(err) = launch_server(config) {
                eprintln!("Server failed with error: {}", err);
            }
        }
        Err(err) => {
            eprintln!("Encountered error when loading config: {}", err);
            std::process::exit(1);
        }
    }
}
