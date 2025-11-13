// src/utils/time_util.rs

use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// 当前 UTC 时间戳（秒）
pub fn now_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u64
}

/// 当前 UTC 时间戳（毫秒）
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// 毫秒转秒（向下取整）
pub fn ms_to_s(ms: u64) -> u64 {
    ms / 1000
}

/// 秒转毫秒
pub fn s_to_ms(s: u64) -> u64 {
    s * 1000
}

/// 距离指定时间戳（秒）的差值（当前 - ts）
pub fn duration_since(ts_s: u64) -> u64 {
    now_s() - ts_s
}

/// 计算两个毫秒时间戳的间隔
pub fn elapsed_ms(start_ms: u64, end_ms: u64) -> u64 {
    end_ms - start_ms
}

/// Unix 秒 → SystemTime
pub fn unix_to_system(ts_s: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(ts_s as u64)
}

/// SystemTime → Unix 秒
pub fn system_to_unix(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let s = now_s();
        let ms = now_ms();
        assert!(ms > s * 1000);
        assert_eq!(ms_to_s(ms), s);
        assert_eq!(s_to_ms(s), s * 1000);
    }
}
