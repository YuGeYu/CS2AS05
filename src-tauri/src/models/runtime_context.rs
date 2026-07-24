use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeContext {
    pub app_name: String,
    pub app_version: String,
    pub rust_crate: String,
    pub target: String,
    pub profile: String,
}
