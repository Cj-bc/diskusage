use std::time::Duration;

/// How byte counts are rendered as text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SizeFormat {
    /// Exact byte count, e.g. "1536 bytes".
    Bytes,
    /// Powers of 1024, e.g. "1.50 KiB".
    Binary,
    /// Powers of 1000, e.g. "1.54 KB".
    Decimal,
    /// Binary units, but plain bytes below 1 KiB.
    #[default]
    Auto,
}

const BINARY_UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
const DECIMAL_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

pub fn format_size(size: u64) -> String {
    format_size_with_format(size, SizeFormat::Auto)
}

pub fn format_size_with_format(size: u64, format: SizeFormat) -> String {
    match format {
        SizeFormat::Bytes => format!("{} bytes", size),
        SizeFormat::Binary => format_binary_size(size),
        SizeFormat::Decimal => format_decimal_size(size),
        SizeFormat::Auto => {
            if size < 1024 {
                format!("{} B", size)
            } else {
                format_binary_size(size)
            }
        }
    }
}

fn format_binary_size(size: u64) -> String {
    if size == 0 {
        return "0 B".to_string();
    }

    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < BINARY_UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, BINARY_UNITS[unit_index])
    } else if size_f >= 100.0 {
        format!("{:.0} {}", size_f, BINARY_UNITS[unit_index])
    } else if size_f >= 10.0 {
        format!("{:.1} {}", size_f, BINARY_UNITS[unit_index])
    } else {
        format!("{:.2} {}", size_f, BINARY_UNITS[unit_index])
    }
}

fn format_decimal_size(size: u64) -> String {
    if size == 0 {
        return "0 B".to_string();
    }

    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1000.0 && unit_index < DECIMAL_UNITS.len() - 1 {
        size_f /= 1000.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, DECIMAL_UNITS[unit_index])
    } else if size_f >= 100.0 {
        format!("{:.0} {}", size_f, DECIMAL_UNITS[unit_index])
    } else if size_f >= 10.0 {
        format!("{:.1} {}", size_f, DECIMAL_UNITS[unit_index])
    } else {
        format!("{:.2} {}", size_f, DECIMAL_UNITS[unit_index])
    }
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    
    if total_seconds < 60 {
        format!("{}.{:02}s", total_seconds, duration.subsec_millis() / 10)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}m {:02}s", minutes, seconds)
    } else {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!("{}h {:02}m {:02}s", hours, minutes, seconds)
    }
}

pub fn format_percentage(value: f64) -> String {
    if value >= 99.95 {
        "100%".to_string()
    } else if value >= 10.0 {
        format!("{:.1}%", value)
    } else if value >= 1.0 {
        format!("{:.2}%", value)
    } else if value >= 0.1 {
        format!("{:.3}%", value)
    } else if value > 0.0 {
        "<0.1%".to_string()
    } else {
        "0%".to_string()
    }
}

pub fn format_number(number: usize) -> String {
    if number < 1_000 {
        number.to_string()
    } else if number < 1_000_000 {
        format!("{:.1}K", number as f64 / 1_000.0)
    } else if number < 1_000_000_000 {
        format!("{:.1}M", number as f64 / 1_000_000.0)
    } else {
        format!("{:.1}B", number as f64 / 1_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_binary_size() {
        assert_eq!(format_binary_size(0), "0 B");
        assert_eq!(format_binary_size(512), "512 B");
        assert_eq!(format_binary_size(1024), "1.00 KiB");
        assert_eq!(format_binary_size(1536), "1.50 KiB");
        assert_eq!(format_binary_size(1048576), "1.00 MiB");
        assert_eq!(format_binary_size(1073741824), "1.00 GiB");
    }

    #[test]
    fn test_format_decimal_size() {
        assert_eq!(format_decimal_size(0), "0 B");
        assert_eq!(format_decimal_size(500), "500 B");
        assert_eq!(format_decimal_size(1000), "1.00 KB");
        assert_eq!(format_decimal_size(1500), "1.50 KB");
        assert_eq!(format_decimal_size(1000000), "1.00 MB");
        assert_eq!(format_decimal_size(1000000000), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "0.50s");
        assert_eq!(format_duration(Duration::from_secs(30)), "30.00s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 01m 01s");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(0.0), "0%");
        assert_eq!(format_percentage(0.05), "<0.1%");
        assert_eq!(format_percentage(0.123), "0.123%");
        assert_eq!(format_percentage(1.234), "1.23%");
        assert_eq!(format_percentage(12.34), "12.3%");
        assert_eq!(format_percentage(99.99), "100%");
    }
}