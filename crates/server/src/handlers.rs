use crate::db;
use shared::models::{CommitRequest, CommitResponse, FetchRequest, FetchResponse};
use axum::Json;
use axum::http::{HeaderMap};
use shared::check_ownership;

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

    match check_ownership(auth_token, &request.repo_url).await {
        Ok(status) => {
            if !status {
                return Json(CommitResponse {
                    success: false,
                    error: Some("You don't have ownership of this repository".into()),
                });
            }
        }
        Err(err) => {
            return Json(CommitResponse {
                success: false,
                error: Some(err),
            });
        }
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

pub async fn fetch(
    headers: HeaderMap,
    Json(request): Json<FetchRequest>,
) -> Json<FetchResponse> {
    let auth_token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or_default();

    if auth_token.is_empty() {
        return Json(FetchResponse {
            success: false,
            env_files: None,
            error: Some("No token provided".into()),
        });
    }

    match check_ownership(auth_token, &request.repo_url).await {
        Ok(status) => {
            if !status {
                return Json(FetchResponse {
                    success: false,
                    env_files: None,
                    error: Some("You don't have ownership of this repository".into()),
                });
            }
        }
        Err(err) => {
            return Json(FetchResponse {
                success: false,
                env_files: None,
                error: Some(err),
            });
        }
    }

    match db::read(&request.repo_url, &request.commit_id) {
        Ok(env_files) => {
            Json(FetchResponse {
                success: true,
                env_files: Some(env_files),
                error: None,
            })
        }
        Err(err) => {
            Json(FetchResponse {
                success: false,
                env_files: None,
                error: Some(err),
            })
        }
    }
}
