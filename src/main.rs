mod config;

use config::load_config;

fn main() {
    match load_config() {
        Ok(config) => {
            eprintln!("Loaded config {:?}", config);
        }
        Err(err) => {
            eprintln!("Encountered error when loading config: {}", err);
            std::process::exit(1);
        }
    }
}
