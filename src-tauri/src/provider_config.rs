const DEFAULT_CURSEFORGE_API_KEY: Option<&str> = option_env!("CURSEFORGE_API_KEY");

/// Resolves the user override first, then the key embedded at build time.
///
/// `CURSEFORGE_API_KEY` is intentionally read only inside the native process so
/// it is never sent to the WebView. A desktop binary cannot keep an embedded
/// key secret from a determined user, so it must be scoped and rotated by the
/// key owner.
pub fn curseforge_api_key(user_key: Option<&str>) -> Option<String> {
    user_key
        .filter(|key| !key.trim().is_empty())
        .or_else(|| DEFAULT_CURSEFORGE_API_KEY.filter(|key| !key.trim().is_empty()))
        .map(ToString::to_string)
}
