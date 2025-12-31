use crate::utils::get_token;
use log::{error};
use reqwest::{Error, Response};
use enva_shared::models::{CommitRequest, CommitResponse, FetchRequest, FetchResponse};
use serde::de::DeserializeOwned;
use std::env;

async fn parse_response<T: DeserializeOwned>(res: Result<Response, Error>) -> Option<T> {
    match res {
        Ok(response) if response.status().is_success() => {
            match response.json::<T>().await {
                Ok(data) => {
                    Some(data)
                }
                Err(e) => {
                    error!("Failed to parse response JSON: {}", e);
                    None
                }
            }
        }
        Ok(response) => {
            error!("Server returned error status: {}", response.status());
            None
        }
        Err(e) => {
            error!("Network error: {}", e);
            None
        }
    }
}

pub async fn call_commit(req: CommitRequest) -> Option<CommitResponse> {
    let client = reqwest::Client::new();
    let base_url = env::var("BASE_URL").expect("BASE_URL environment variable must be set");

    let res = client
        .post(format!("{}/commit", base_url))
        .bearer_auth(get_token().expect("Failed to get token"))
        .json(&req)
        .send()
        .await;

    parse_response::<CommitResponse>(res).await
}

pub async fn call_fetch(req: FetchRequest) -> Option<FetchResponse> {
    let client = reqwest::Client::new();
    let base_url = env::var("BASE_URL").expect("BASE_URL environment variable must be set");

    let res = client
        .post(format!("{}/fetch", base_url))
        .bearer_auth(get_token().expect("Failed to get token"))
        .json(&req)
        .send()
        .await;

    parse_response::<FetchResponse>(res).await
}