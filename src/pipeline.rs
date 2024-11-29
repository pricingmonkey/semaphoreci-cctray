use reqwest::header::AUTHORIZATION;
use reqwest::{Client, StatusCode};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
}

#[derive(Deserialize)]
pub struct Pipeline {
    // DONE | RUNNING
    pub state: String,
    // PASSED | FAILED
    pub result: Option<String>,
    pub name: String,
    pub created_at: Timestamp,
    pub done_at: Timestamp,
    pub ppl_id: String,
    pub wf_id: String,
}

pub async fn get_pipeline(
    auth_token: &String,
    org: &String,
    project_id: &String,
    client: &Client,
) -> Result<Vec<Pipeline>, &'static str> {
    let url = format!(
        "https://{}.semaphoreci.com/api/v1alpha/pipelines?project_id={}",
        org, project_id
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

    result
        .json::<Vec<Pipeline>>()
        .await
        .map_err(|e| {
            println!("Error: {}", e);
            "Invalid JSON"
        })
}
