
use chrono::prelude::*;

pub fn current_time() -> String {
    return format!("{} Uhr", Local::now().format("%Y-%m-%d %H:%M:%S"));
}