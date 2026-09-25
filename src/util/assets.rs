pub fn asset_key_to_url(key: Option<String>) -> Option<String> {
    key.map(|v| format!("/assets/{v}"))
}