use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CommitRequest {
    pub repo_url: String,
    pub branch: String,
    pub commit_id: String,
    pub env_files: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FetchRequest {
    pub repo_url: String,
    pub commit_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResponse {
    pub success: bool,
    pub env_files: Option<HashMap<String, String>>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckCommitRequest {
    pub repo_url: String,
    pub commit_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckCommitResponse {
    pub exists: bool,
    pub error: Option<String>,
}