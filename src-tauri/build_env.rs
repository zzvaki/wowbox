pub fn value(contents: &str, wanted_key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let (key, value) = line.split_once('=')?;
        if key.trim() != wanted_key {
            return None;
        }
        normalize(value)
    })
}

pub fn normalize(value: &str) -> Option<String> {
    let value = value.trim();
    let value = if value.len() >= 2 {
        let bytes = value.as_bytes();
        if matches!(
            (bytes[0], bytes[value.len() - 1]),
            (b'\'', b'\'') | (b'"', b'"')
        ) {
            &value[1..value.len() - 1]
        } else {
            value
        }
    } else {
        value
    };
    (!value.is_empty() && !value.contains(['\r', '\n'])).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{normalize, value};

    #[test]
    fn reads_only_the_exact_requested_key() {
        let contents = r#"
            # local build secrets
            VITE_CURSEFORGE_API_KEY=public-value
            export CURSEFORGE_API_KEY="native-value"
        "#;

        assert_eq!(
            value(contents, "CURSEFORGE_API_KEY").as_deref(),
            Some("native-value")
        );
    }

    #[test]
    fn rejects_empty_or_multiline_values() {
        assert_eq!(normalize("  "), None);
        assert_eq!(normalize("unsafe\nvalue"), None);
    }
}
