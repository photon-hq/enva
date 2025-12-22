use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CommitRequest {
    pub repo_url: String,
    pub branch: String,
    pub commit_id: String,
    pub env_files: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct CommitResponse {
    pub success: bool,
    pub error: Option<String>,
}
