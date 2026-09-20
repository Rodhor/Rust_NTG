use sqlx::types::chrono::{NaiveDateTime, Utc};

pub fn timestamp_now_naive() -> NaiveDateTime {
    Utc::now().naive_utc()
}
