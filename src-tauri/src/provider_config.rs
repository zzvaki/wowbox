const DEFAULT_CURSEFORGE_API_KEY: Option<&str> = option_env!("CURSEFORGE_API_KEY");

/// Resolves a native-process environment key first, then the key embedded from
/// the project-local `.env.local` file at build time.
///
/// `CURSEFORGE_API_KEY` is intentionally read only inside the native process so
/// it is never sent to the WebView. A desktop binary cannot keep an embedded
/// key secret from a determined user, so it must be scoped and rotated by the
/// key owner.
pub fn curseforge_api_key() -> Option<String> {
    std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .and_then(|key| non_empty_key(&key))
        .or_else(|| DEFAULT_CURSEFORGE_API_KEY.and_then(non_empty_key))
}

fn non_empty_key(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
