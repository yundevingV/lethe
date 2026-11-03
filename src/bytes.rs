pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if unit == 0 {
        return format!("{bytes} B");
    }
    let text = format!("{value:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    format!("{text} {}", UNITS[unit])
}

pub fn parse_bytes(raw: &str) -> Option<u64> {
    let token = raw.split_whitespace().next().unwrap_or(raw).trim();
    let split = token.find(|c: char| !(c.is_ascii_digit() || c == '.'))?;
    if split == 0 {
        return None;
    }
    let value: f64 = token[..split].parse().ok()?;
    let scale = match token[split..].trim().to_ascii_lowercase().as_str() {
        "tb" => 1_000_000_000_000.0,
        "gb" => 1_000_000_000.0,
        "mb" => 1_000_000.0,
        "kb" => 1_000.0,
        "b" => 1.0,
        _ => return None,
    };
    Some((value * scale).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_sizes() {
        assert_eq!(parse_bytes("1.5GB"), Some(1_500_000_000));
        assert_eq!(parse_bytes("1.5GB (71%)"), Some(1_500_000_000));
        assert_eq!(parse_bytes("800MB"), Some(800_000_000));
        assert_eq!(parse_bytes("0B"), Some(0));
        assert_eq!(parse_bytes("nope"), None);
    }
}
