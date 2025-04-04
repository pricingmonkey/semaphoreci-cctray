use reqwest::header::AUTHORIZATION;
use reqwest::{Client, StatusCode};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
}

#[derive(Deserialize, Eq, PartialEq)]
pub enum State {
    DONE,
    RUNNING
}

#[derive(Deserialize, Clone)]
pub enum Result {
    PASSED,
    FAILED
}

#[derive(Deserialize)]
pub struct Pipeline {
    pub state: State,
    pub result: Option<Result>,
    pub name: String,
    pub created_at: Timestamp,
    pub done_at: Timestamp,
    pub ppl_id: String,
    pub wf_id: String,
}

pub async fn get_pipeline(
    base_url: &String,
    project_id: &String,
    auth_token: &String,
    client: &Client,
) -> core::result::Result<Vec<Pipeline>, &'static str> {
    let url = format!(
        "{}/api/v1alpha/pipelines?project_id={}",
        base_url, project_id
    );

    let result = client
        .get(url)
        .header(AUTHORIZATION, format!("Token {}", auth_token))
        .send()
        .await
        .unwrap();

    if result.status() != StatusCode::OK {
        return match result.status() {
            StatusCode::NOT_FOUND => Err("Not found"),
            StatusCode::FORBIDDEN => Err("Forbidden"),
            StatusCode::UNAUTHORIZED => Err("Unauthorized"),
            _ => Err("Internal server error"),
        };
    }

    result.text().await.map_err(|e| {
        println!("Error: {}", e);
        "Invalid response body"
    }).and_then(|text| {
        return serde_json::from_str(&text).map_err(|e| {
            println!("Error: {}", e);
            "Invalid JSON"
        });
    })
}
