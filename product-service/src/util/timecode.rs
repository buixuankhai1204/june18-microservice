use chrono::{Duration, NaiveTime};

/// Helper function to parse timecode with frame support
/// Converts SMPTE timecode (H:MM:SS:FF or HH:MM:SS:FF) to NaiveTime
/// Also supports standard formats: HH:MM:SS.mmm and HH:MM:SS
///
/// # Arguments
/// * `timecode` - Timecode string in one of the supported formats
///
/// # Returns
/// * `Option<NaiveTime>` - Parsed time or None if parsing fails
///
/// # Example
/// ```
/// let time = parse_timecode("0:00:30:15"); // SMPTE with frames
/// let time = parse_timecode("00:00:30.500"); // Milliseconds
/// let time = parse_timecode("00:00:30"); // Standard
/// ```
pub fn parse_timecode(timecode: &str) -> Option<NaiveTime> {
    // Split by colon to check if it's frame-based timecode
    let parts: Vec<&str> = timecode.split(':').collect();

    if parts.len() == 4 {
        // Frame-based timecode: H:MM:SS:FF or HH:MM:SS:FF
        let hours: u32 = parts[0].parse().ok()?;
        let minutes: u32 = parts[1].parse().ok()?;
        let seconds: u32 = parts[2].parse().ok()?;
        let frames: u32 = parts[3].parse().ok()?;

        // Convert frames to milliseconds (assuming 25 FPS PAL standard)
        // 1 frame = 1000ms / 25 = 40ms
        const MS_PER_FRAME: u32 = 40;
        let total_ms = frames * MS_PER_FRAME;

        // Create NaiveTime with milliseconds
        NaiveTime::from_hms_milli_opt(hours, minutes, seconds, total_ms)
    } else {
        // Try parsing standard formats: HH:MM:SS.mmm or HH:MM:SS
        NaiveTime::parse_from_str(timecode, "%H:%M:%S%.3f")
            .or_else(|_| NaiveTime::parse_from_str(timecode, "%H:%M:%S"))
            .ok()
    }
}

/// Calculate duration from two timecodes
/// Supports all formats: SMPTE (H:MM:SS:FF), milliseconds (HH:MM:SS.mmm), and standard (HH:MM:SS)
///
/// # Arguments
/// * `timecode_start` - Start timecode string
/// * `timecode_end` - End timecode string
///
/// # Returns
/// * `Option<Duration>` - Duration between the two timecodes
///
/// # Example
/// ```
/// let duration = calculate_duration("0:00:00:00", "0:00:30:00");
/// ```
pub fn calculate_duration(timecode_start: &str, timecode_end: &str) -> Option<Duration> {
    let start = parse_timecode(timecode_start)?;
    let end = parse_timecode(timecode_end)?;
    Some(end.signed_duration_since(start))
}

/// Calculate duration in minutes (as f32) from two timecodes
/// This is a convenience function for APIs that expect duration in minutes
///
/// # Arguments
/// * `timecode_start` - Start timecode string
/// * `timecode_end` - End timecode string
///
/// # Returns
/// * `Option<f32>` - Duration in minutes
pub fn calculate_duration_minutes(timecode_start: &str, timecode_end: &str) -> Option<f32> {
    let duration = calculate_duration(timecode_start, timecode_end)?;
    Some(duration.num_milliseconds() as f32 / 60_000.0)
}

#[cfg(test)]
mod tests {
    use chrono::Timelike;

    use super::*;

    #[test]
    fn test_parse_smpte_timecode() {
        // Test SMPTE format with frames
        let time = parse_timecode("0:00:30:15").unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.second(), 30);
        // 15 frames * 40ms = 600ms
        assert_eq!(time.nanosecond(), 600_000_000);
    }

    #[test]
    fn test_parse_millisecond_timecode() {
        // Test HH:MM:SS.mmm format
        let time = parse_timecode("00:00:30.500").unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.second(), 30);
        assert_eq!(time.nanosecond(), 500_000_000);
    }

    #[test]
    fn test_parse_standard_timecode() {
        // Test HH:MM:SS format
        let time = parse_timecode("00:00:30").unwrap();
        assert_eq!(time.hour(), 0);
        assert_eq!(time.minute(), 0);
        assert_eq!(time.second(), 30);
    }

    #[test]
    fn test_calculate_duration_smpte() {
        // 30 seconds in SMPTE format
        let duration = calculate_duration("0:00:00:00", "0:00:30:00").unwrap();
        assert_eq!(duration.num_seconds(), 30);
    }

    #[test]
    fn test_calculate_duration_minutes() {
        // 1 minute in standard format
        let minutes = calculate_duration_minutes("00:00:00", "00:01:00").unwrap();
        assert_eq!(minutes, 1.0);
    }

    #[test]
    fn test_mixed_formats() {
        // Start with SMPTE, end with standard - should work
        let duration = calculate_duration("0:00:00:00", "00:00:30").unwrap();
        assert_eq!(duration.num_seconds(), 30);
    }
}
