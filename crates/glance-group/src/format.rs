//! Human-readable formatting shared by the CLI and the GUI.

/// Format KiB as a compact decimal size ("980 MB", "1.2 GB"), matching
/// GNOME System Monitor's units.
pub fn mem(kib: u64) -> String {
    let bytes = kib as f64 * 1024.0;
    const K: f64 = 1000.0;
    if bytes < K * K {
        format!("{:.0} kB", bytes / K)
    } else if bytes < K * K * K {
        format!("{:.0} MB", bytes / (K * K))
    } else {
        format!("{:.1} GB", bytes / (K * K * K))
    }
}

/// Format CPU permille as a percentage ("4.2%", "12%").
pub fn cpu(permille: u32) -> String {
    if permille >= 100 {
        format!("{}%", (permille + 5) / 10)
    } else {
        format!("{}.{}%", permille / 10, permille % 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(mem(500), "512 kB");
        assert_eq!(mem(980 * 1000 * 1000 / 1024), "980 MB");
        assert_eq!(mem(1_200_000_000 / 1024), "1.2 GB");
        assert_eq!(cpu(42), "4.2%");
        assert_eq!(cpu(0), "0.0%");
        assert_eq!(cpu(125), "13%");
    }
}
