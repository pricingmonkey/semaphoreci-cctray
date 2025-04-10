use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Timestamp {
    pub seconds: i64,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub enum State {
    DONE,
    RUNNING,
    #[serde(untagged)]
    UNKNOWN(String)
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Result {
    PASSED,
    FAILED,
    #[serde(untagged)]
    UNKNOWN(String)
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

#[derive(Deserialize, Debug)]
pub struct ProjectMetadata {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Project {
    pub metadata: ProjectMetadata,
}

pub async fn get_projects(
    base_url: &String,
    auth_token: &String,
    client: &Client,
) -> core::result::Result<Vec<Project>, reqwest::Error> {
    let url = format!("{}/api/v1alpha/projects", base_url);

    get(client, url, auth_token).await
}

pub async fn get_pipelines(
    base_url: &String,
    project_id: &String,
    auth_token: &String,
    client: &Client,
) -> core::result::Result<Vec<Pipeline>, reqwest::Error> {
    let url = format!(
        "{}/api/v1alpha/pipelines?project_id={}",
        base_url, project_id
    );

    get(client, url, auth_token).await
}

async fn get<T: DeserializeOwned>(
    client: &Client,
    url: String,
    auth_token: &String,
) -> core::result::Result<T, reqwest::Error> {
    let result = client
        .get(url)
        .header(AUTHORIZATION, format!("Token {}", auth_token))
        .send()
        .await?
        .error_for_status()?;

    result.json::<T>().await
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use serde_json;

    #[test]
    fn test_result_known_value_deserialised_as_enum() {
        let result: Result = serde_json::from_str("\"PASSED\"").unwrap();
        assert_eq!(result, Result::PASSED);
    }

    #[test]
    fn test_result_unknown_value_deserialised_as_string() {
        let result: Result = serde_json::from_str("\"BLAH\"").unwrap();
        assert_eq!(result, Result::UNKNOWN("BLAH".to_string()));
    }

}
