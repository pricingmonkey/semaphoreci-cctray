use crate::semaphoreci;
use crate::semaphoreci::{Pipeline, State};
use chrono::DateTime;
use itertools::Itertools;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct CCTrayProjectInfo {
    pub name: String,
    pub activity: Activity,
    pub last_build_status: BuildStatus,
    pub last_build_label: String,
    pub last_build_time: String,
    pub web_url: String,
}

/*
 * The SemaphoreCI API sometime returns the pipeline name as "Pipeline". This happens when a build
 * is queued, or when a build has failed before starting, and probably in any situation where the
 * has not read the yaml file yet or has failed to read it.
 *
 * This pipelines are excluded from the cctray output as they do not provide useful information. In
 * particular:
 * - they are confusing, because they are rendered with a generic label that doesn't identify the
 *   project;
 * - in case of failed build, they remain in the feed as failed builds even when the build is fixed.
 *
 * Excluding these pipelines is not ideal. It is useful to see when a build is queued or when it
 * failed before starting. The problem is that at the moment we cannot represent these build in a
 * meaningful way, mainly because we are using the pipeline name for grouping pipelines into cctray
 * projects. Grouping by workflow id would maybe work better.
 */
const TEMPORARY_PIPELINE_NAME: &'static str = "Pipeline";

fn get_cctray_project_info(
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
        _ => Activity::Sleeping,
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

pub fn to_cctray_project_info(pipelines: Vec<Pipeline>, org: &String) -> Vec<CCTrayProjectInfo> {
    let pipelines_by_name = pipelines.iter().into_group_map_by(|p| p.name.clone());

    Vec::from_iter(
        pipelines_by_name
            .iter()
            .filter(|(name, _pipelines)| **name != String::from(TEMPORARY_PIPELINE_NAME))
            .map(|(name, pipelines)| get_cctray_project_info(name, pipelines, org))
            .sorted_by_key(|i| i.last_build_time.clone())
            .rev(),
    )
}

fn serialize_project(info: &CCTrayProjectInfo) -> String {
    format!(
        "<Project name=\"{}\" activity=\"{}\" lastBuildStatus=\"{}\" lastBuildLabel=\"{}\" lastBuildTime=\"{}\" webUrl=\"{}\"/>",
        info.name, info.activity.as_str(), info.last_build_status.as_str(), info.last_build_label, info.last_build_time, info.web_url
    )
}

pub fn serialize(cctray_projects: Vec<CCTrayProjectInfo>) -> String {
    let xml_fragment = cctray_projects
        .iter()
        .map(serialize_project)
        .join("\n");

    format!("<Projects>{}</Projects>", xml_fragment)
}

#[cfg(test)]
mod tests {
    use crate::cctray::{Activity, BuildStatus, CCTrayProjectInfo};
    use crate::semaphoreci::Result::{FAILED, PASSED};
    use crate::semaphoreci::{Pipeline, State, Timestamp};
    use crate::cctray::to_cctray_project_info;

    #[test]
    fn convert_sem_pipelines_to_cctray_projects() {
        let pipeline1 = Pipeline {
            name: String::from("foo"),
            state: State::DONE,
            result: Some(PASSED),
            ppl_id: String::from("ppl1"),
            wf_id: String::from("wf1"),
            created_at: Timestamp { seconds: 1000 },
            done_at: Timestamp { seconds: 1100 },
        };

        let sem_pipelines = vec![pipeline1];

        let org = String::from("org-name");
        let cctray_projects = to_cctray_project_info(sem_pipelines, &org);

        assert_eq!(
            cctray_projects,
            vec![CCTrayProjectInfo {
                name: String::from("foo"),
                activity: Activity::Sleeping,
                last_build_status: BuildStatus::Success,
                last_build_label: String::from("ppl1"),
                last_build_time: String::from("1970-01-01T00:18:20+00:00"),
                web_url: String::from(
                    "https://org-name.semaphoreci.com/workflows/wf1?pipeline_id=ppl1"
                ),
            }]
        );
    }

    #[test]
    fn returns_one_cctray_project_when_multiple_pipelines_have_the_same_name() {
        let sem_pipelines = vec![
            Pipeline {
                name: String::from("foo"),
                state: State::RUNNING,
                result: None,
                ppl_id: String::from("ppl3"),
                wf_id: String::from("wf2"),
                created_at: Timestamp { seconds: 3000 },
                done_at: Timestamp { seconds: 3100 },
            },
            Pipeline {
                name: String::from("foo"),
                state: State::DONE,
                result: Some(PASSED),
                ppl_id: String::from("ppl2"),
                wf_id: String::from("wf1"),
                created_at: Timestamp { seconds: 2000 },
                done_at: Timestamp { seconds: 2100 },
            },
            Pipeline {
                name: String::from("bar"),
                state: State::DONE,
                result: Some(PASSED),
                ppl_id: String::from("ppl1"),
                wf_id: String::from("wf1"),
                created_at: Timestamp { seconds: 1000 },
                done_at: Timestamp { seconds: 1100 },
            },
        ];

        let org = String::from("org-name");
        let cctray_projects = to_cctray_project_info(sem_pipelines, &org);

        assert_eq!(
            cctray_projects,
            vec![
                CCTrayProjectInfo {
                    name: String::from("foo"),
                    activity: Activity::Building,
                    last_build_status: BuildStatus::Success,
                    last_build_label: String::from("ppl2"),
                    last_build_time: String::from("1970-01-01T00:35:00+00:00"),
                    web_url: String::from(
                        "https://org-name.semaphoreci.com/workflows/wf2?pipeline_id=ppl3"
                    ),
                },
                CCTrayProjectInfo {
                    name: String::from("bar"),
                    activity: Activity::Sleeping,
                    last_build_status: BuildStatus::Success,
                    last_build_label: String::from("ppl1"),
                    last_build_time: String::from("1970-01-01T00:18:20+00:00"),
                    web_url: String::from(
                        "https://org-name.semaphoreci.com/workflows/wf1?pipeline_id=ppl1"
                    ),
                }
            ]
        );
    }

    #[test]
    fn excludes_pipelines_with_temporary_name() {
        let sem_pipelines = vec![
            Pipeline {
                name: String::from("foo"),
                state: State::DONE,
                result: Some(PASSED),
                ppl_id: String::from("ppl2"),
                wf_id: String::from("wf2"),
                created_at: Timestamp { seconds: 2000 },
                done_at: Timestamp { seconds: 2100 },
            },
            Pipeline {
                name: String::from("Pipeline"),
                state: State::DONE,
                result: Some(FAILED),
                ppl_id: String::from("ppl1"),
                wf_id: String::from("wf1"),
                created_at: Timestamp { seconds: 1000 },
                done_at: Timestamp { seconds: 1100 },
            },
        ];

        let org = String::from("org-name");
        let cctray_projects = to_cctray_project_info(sem_pipelines, &org);

        assert_eq!(
            cctray_projects,
            vec![
                CCTrayProjectInfo {
                    name: String::from("foo"),
                    activity: Activity::Sleeping,
                    last_build_status: BuildStatus::Success,
                    last_build_label: String::from("ppl2"),
                    last_build_time: String::from("1970-01-01T00:35:00+00:00"),
                    web_url: String::from(
                        "https://org-name.semaphoreci.com/workflows/wf2?pipeline_id=ppl2"
                    ),
                }
            ]
        );
    }
}
