//! Utility functions

/// Format download count for display
///
/// Converts a raw download count into a human-readable format:
/// - Millions: "1.5M", "10.2M"
/// - Thousands: "5.3K", "999.9K"
/// - Less than 1000: "123", "999"
pub fn format_download_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}
