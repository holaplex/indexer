use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hello {
    pkg_version: String,
    git_head: Option<String>,
    git_remote: Option<String>,
    rustc_version: String,
    build_host: String,
    build_target: String,
    build_profile: String,
    build_platform: String,
    host: String,
    name: String,
}
