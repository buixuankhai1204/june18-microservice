use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike};

pub fn convert_naive_datetime_to_local(
    naive_datetime: &Option<NaiveDateTime>,
) -> Option<DateTime<Local>> {
    match naive_datetime {
        Some(naive_datetime) => Some(Local.from_local_datetime(naive_datetime).unwrap()),
        None => None,
    }
}

pub fn sum_naive_datetime_with_hms(
    naive_datetime: NaiveDate,
    hour: i64,
    minute: i64,
    second: i64,
) -> NaiveDateTime {
    let time_on_broadcast_registration: NaiveDateTime = naive_datetime
        .and_time(NaiveTime::from_hms_milli_opt(00, 00, 00, 00).unwrap_or_default())
        + Duration::seconds(second)
        + Duration::minutes(minute)
        + Duration::hours(hour);

    time_on_broadcast_registration
}

pub fn add_naive_times(t1: NaiveTime, miniutes: f32) -> NaiveTime {
    let seconds2 = miniutes.fract() * 100f32;
    let new_minute = miniutes.trunc() as i64;
    let seconds1 = t1.num_seconds_from_midnight() as i64;
    let seconds2 = new_minute * 60 + seconds2 as i64;

    let total_seconds = seconds1 + seconds2;

    // Wrap around 24h (86400 seconds)
    let wrapped_seconds = total_seconds % 86400;

    NaiveTime::from_num_seconds_from_midnight_opt(wrapped_seconds as u32, 0).expect("Invalid time")
}

pub fn naivetime_to_timecode(time: NaiveTime, fps: u32) -> String {
    let hour = time.hour();
    let minute = time.minute();
    let second = time.second();
    let millis = time.nanosecond() / 1_000_000; // từ nanosec → millisec

    // Tính frame_config gần đúng từ millis
    let frame = ((millis as f64 / 1000.0) * fps as f64).round() as u32;

    format!("{:02}:{:02}:{:02}.{:02}", hour, minute, second, frame)
}

pub fn time_str_to_minutes(time_str: &str) -> Result<f64, String> {
    let parts: Vec<&str> = time_str.split('.').collect();

    if parts.len() != 2 {
        return Err("Input must be in 'MM.SS' format".to_string());
    }

    let minutes: u32 = parts[0].parse().map_err(|_| "Invalid minutes value")?;
    let seconds: u32 = parts[1].parse().map_err(|_| "Invalid seconds value")?;

    if seconds >= 60 {
        return Err("Seconds must be between 0 and 59".to_string());
    }

    let total_minutes = minutes as f64 + (seconds as f64 / 100.0);
    Ok(total_minutes)
}
