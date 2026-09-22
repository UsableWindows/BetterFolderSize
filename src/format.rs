/// Formats a raw number with spaces as thousands separators (e.g. 1234567 -> "1 234 567").
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let len = s.len();
    let mut result = String::with_capacity(len + (len / 3));
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}

/// Formats byte count into a human-readable string (B, KB, MB, GB, TB).
pub fn format_human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b < KB {
        format!("{} B", bytes)
    } else if b < MB {
        format!("{:.2} KB", b / KB)
    } else if b < GB {
        format!("{:.2} MB", b / MB)
    } else if b < TB {
        format!("{:.2} GB", b / GB)
    } else {
        format!("{:.2} TB", b / TB)
    }
}

/// Formats size with both human-readable unit and exact byte count,
/// e.g. "1.24 GB (1 331 439 812 B)" or "512 B" if under 1 KB.
pub fn format_size_full(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else {
        format!("{} ({} B)", format_human_size(bytes), format_number(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(9), "9");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1 000");
        assert_eq!(format_number(12345), "12 345");
        assert_eq!(format_number(1234567), "1 234 567");
        assert_eq!(format_number(1000000000), "1 000 000 000");
    }

    #[test]
    fn test_format_human_size() {
        assert_eq!(format_human_size(0), "0 B");
        assert_eq!(format_human_size(512), "512 B");
        assert_eq!(format_human_size(1023), "1023 B");
        assert_eq!(format_human_size(1024), "1.00 KB");
        assert_eq!(format_human_size(1536), "1.50 KB");
        assert_eq!(format_human_size(1048576), "1.00 MB");
        assert_eq!(format_human_size(1073741824), "1.00 GB");
        assert_eq!(format_human_size(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_format_size_full() {
        assert_eq!(format_size_full(0), "0 B");
        assert_eq!(format_size_full(500), "500 B");
        assert_eq!(format_size_full(1024), "1.00 KB (1 024 B)");
        assert_eq!(format_size_full(1048576), "1.00 MB (1 048 576 B)");
        assert_eq!(format_size_full(1331439812), "1.24 GB (1 331 439 812 B)");
    }
}
