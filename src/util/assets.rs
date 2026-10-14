pub enum AssetPrefix {
    Identity,
    Track,
}

impl std::string::ToString for AssetPrefix {
    fn to_string(&self) -> String {
        match self {
            AssetPrefix::Identity => "identity",
            AssetPrefix::Track => "track",
        }.to_string()
    }
}

pub fn asset_id_to_url(prefix: AssetPrefix, id: Option<String>) -> Option<String> {
    id.map(|v| format!("/{}/assets/{v}", prefix.to_string()))
}
