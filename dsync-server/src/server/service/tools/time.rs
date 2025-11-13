use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> i64 {
    let now_time = SystemTime::now();
    match now_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().try_into().unwrap(),
        Err(err) => {
            log::error!("Error when trying to get current timestamp {err}");
            // I think if this happens then something is seriously wrong.
            panic!("Error when trying to get current timestamp {err}");
        }
    }
}
