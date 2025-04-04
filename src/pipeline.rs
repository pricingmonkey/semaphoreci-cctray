use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Timestamp {
    pub seconds: i64,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub enum State {
    DONE,
    RUNNING
}

#[derive(Deserialize, Debug, Clone)]
pub enum Result {
    PASSED,
    FAILED
}

#[derive(Deserialize, Debug)]
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
) -> core::result::Result<Vec<Pipeline>, reqwest::Error> {
    let url = format!(
        "{}/api/v1alpha/pipelines?project_id={}",
        base_url, project_id
    );

    let result = client
        .get(url)
        .header(AUTHORIZATION, format!("Token {}", auth_token))
        .send()
        .await
        .unwrap()
        .error_for_status()?;

    result.json::<Vec<Pipeline>>().await
}
