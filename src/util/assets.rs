pub fn asset_key_to_url(prefix: &str, key: Option<String>) -> Option<String> {
    key.map(|v| format!("/{prefix}/{v}"))
}