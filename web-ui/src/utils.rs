// Utility functions for the web UI

/// Format a timestamp for display
pub fn format_timestamp(timestamp: f64) -> String {
    // In a real implementation, this would format the timestamp properly
    format!("{}", timestamp)
}

/// Format a duration for display
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.2}ms", seconds * 1000.0)
    } else {
        format!("{:.2}s", seconds)
    }
}

/// Truncate a string to a maximum length
pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() > max_length {
        format!("{}...", &s[..max_length - 3])
    } else {
        s.to_string()
    }
}