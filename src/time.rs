use std::time::SystemTime;

pub struct DiagonatorDate {}

pub fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Current system time is before UNIX EPOCH")
        .as_secs()
}
