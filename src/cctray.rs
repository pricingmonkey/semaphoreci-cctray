use chrono::DateTime;
use itertools::Itertools;
use crate::semaphoreci;
use crate::semaphoreci::{Pipeline, State};

pub enum Activity {
    Sleeping,
    Building,
    #[allow(dead_code)]
    CheckingModifications,
}

impl Activity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Activity::Sleeping => "Sleeping",
            Activity::Building => "Building",
            Activity::CheckingModifications => "CheckingModifications"
        }
    }
}

pub enum BuildStatus {
    Success,
    Failure,
    #[allow(dead_code)]
    Exception,
    Unknown,
}

impl BuildStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BuildStatus::Success => "Success",
            BuildStatus::Failure => "Failure",
            BuildStatus::Exception => "Exception",
            BuildStatus::Unknown => "Unknown",
        }
    }
}

pub struct CCTrayProjectInfo {
    pub name: String,
    pub activity: Activity,
    pub last_build_status: BuildStatus,
    pub last_build_label: String,
    pub last_build_time: String,
    pub web_url: String,
}

pub fn get_cctray_project_info(
    name: &String,
    pipelines: &Vec<&Pipeline>,
    org: &String,
) -> CCTrayProjectInfo {
    let sorted_pipelines: Vec<&&Pipeline> = pipelines
        .iter()
        .sorted_by_key(|p| p.created_at.seconds)
        .rev()
        .collect();

    let latest_pipeline = sorted_pipelines.get(0).unwrap();
    let last_completed_pipeline = sorted_pipelines.iter().find(|p| p.state == State::DONE);

    let activity = match latest_pipeline.state {
        State::RUNNING => Activity::Building,
        State::DONE => Activity::Sleeping,
    };

    let last_pipeline_result = last_completed_pipeline
        .and_then(|p| p.result.clone());

    let last_build_status = match last_pipeline_result{
        Some(semaphoreci::Result::PASSED) => BuildStatus::Success,
        Some(semaphoreci::Result::FAILED) => BuildStatus::Failure,
        _ => BuildStatus::Unknown
    };

    let last_build_label = last_completed_pipeline.map_or_else(|| "", |p| &p.ppl_id);
    let last_build_time = last_completed_pipeline
        .and_then(|p| DateTime::from_timestamp(p.done_at.seconds, 0))
        .map_or_else(|| String::from(""), |dt| dt.to_rfc3339());
    let web_url = format!(
        "https://{}.semaphoreci.com/workflows/{}?pipeline_id={}",
        org, latest_pipeline.wf_id, latest_pipeline.ppl_id
    );

    CCTrayProjectInfo {
        name: name.clone(),
        activity,
        last_build_status,
        last_build_label: last_build_label.to_string(),
        last_build_time,
        web_url,
    }
}