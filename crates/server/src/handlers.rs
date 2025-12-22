use crate::db;
use crate::models::{CommitRequest, CommitResponse};
use axum::Json;
use axum::http::{HeaderMap};

pub async fn commit(
    headers: HeaderMap,
    Json(request): Json<CommitRequest>,
) -> Json<CommitResponse> {
    let auth_token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or_default();

    if auth_token.is_empty() {
        return Json(CommitResponse {
            success: false,
            error: Some("No token provided".into()),
        });
    }

    db::save(
        &request.repo_url,
        &request.branch,
        &request.commit_id,
        &request.env_files,
    )
    .map_or_else(
        |e| {
            Json(CommitResponse {
                success: false,
                error: e.into(),
            })
        },
        |_| {
            Json(CommitResponse {
                success: true,
                error: None,
            })
        },
    )
}
