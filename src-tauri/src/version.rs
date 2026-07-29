use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
enum Part {
    Number(u64),
    Text(String),
}

pub fn compare_versions(local: &str, remote: &str) -> Ordering {
    let local_parts = parts(local);
    let remote_parts = parts(remote);
    let max_length = local_parts.len().max(remote_parts.len());

    for index in 0..max_length {
        let comparison = match (local_parts.get(index), remote_parts.get(index)) {
            (Some(Part::Number(left)), Some(Part::Number(right))) => left.cmp(right),
            (Some(Part::Text(left)), Some(Part::Text(right))) => left.cmp(right),
            (Some(Part::Number(_)), Some(Part::Text(_))) => Ordering::Greater,
            (Some(Part::Text(_)), Some(Part::Number(_))) => Ordering::Less,
            (Some(part), None) => compare_to_missing(part),
            (None, Some(part)) => compare_to_missing(part).reverse(),
            (None, None) => Ordering::Equal,
        };
        if comparison != Ordering::Equal {
            return comparison;
        }
    }
    Ordering::Equal
}

/// Returns `true` only when a remote version is strictly newer than the local one.
///
/// Addon versions are frequently not valid semantic versions (for example,
/// `612-Retail`), so this intentionally uses WowBox's natural comparison.
pub fn is_remote_newer(local: &str, remote: &str) -> bool {
    compare_versions(local, remote) == Ordering::Less
}

fn compare_to_missing(part: &Part) -> Ordering {
    match part {
        Part::Number(0) => Ordering::Equal,
        Part::Text(text)
            if matches!(
                text.as_str(),
                "release"
                    | "stable"
                    | "final"
                    | "retail"
                    | "classic"
                    | "era"
                    | "anniversary"
                    | "ptr"
            ) =>
        {
            Ordering::Equal
        }
        Part::Text(text) if matches!(text.as_str(), "alpha" | "beta" | "rc") => Ordering::Less,
        _ => Ordering::Greater,
    }
}

fn parts(value: &str) -> Vec<Part> {
    let normalized = value.trim().trim_start_matches(['v', 'V']).to_lowercase();
    let mut parts = Vec::new();
    let mut buffer = String::new();
    let mut numeric: Option<bool> = None;

    for character in normalized.chars() {
        if !character.is_ascii_alphanumeric() {
            flush(&mut parts, &mut buffer, &mut numeric);
            continue;
        }
        let is_numeric = character.is_ascii_digit();
        if numeric.is_some_and(|current| current != is_numeric) {
            flush(&mut parts, &mut buffer, &mut numeric);
        }
        numeric = Some(is_numeric);
        buffer.push(character);
    }
    flush(&mut parts, &mut buffer, &mut numeric);
    parts
}

fn flush(parts: &mut Vec<Part>, buffer: &mut String, numeric: &mut Option<bool>) {
    if buffer.is_empty() {
        *numeric = None;
        return;
    }
    if numeric.unwrap_or(false) {
        parts.push(Part::Number(buffer.parse().unwrap_or(0)));
    } else {
        parts.push(Part::Text(std::mem::take(buffer)));
    }
    buffer.clear();
    *numeric = None;
}

#[cfg(test)]
mod tests {
    use super::is_remote_newer;

    #[test]
    fn identifies_only_strictly_newer_remote_releases() {
        assert!(is_remote_newer("5.4.3", "5.5.1"));
        assert!(is_remote_newer("1.0.0-beta", "1.0.0"));
        assert!(!is_remote_newer("1.10", "1.9"));
        assert!(!is_remote_newer("11.2.0", "v11.2.0-release"));
        assert!(!is_remote_newer("1.0.0", "1.0.0-classic"));
        assert!(!is_remote_newer("1.0.0", "1.0.0-retail"));
    }
}
