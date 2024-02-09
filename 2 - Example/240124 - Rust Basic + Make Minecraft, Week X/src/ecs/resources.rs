use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn now() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub struct DeltaTime {
    pub prev: Duration,
    pub delta: Duration,
}

impl Default for DeltaTime {
    fn default() -> Self {
        DeltaTime {
            prev: now(),
            delta: Duration::from_secs(0),
        }
    }
}
