mod cctray;
mod semaphoreci;

use crate::semaphoreci::Pipeline;
use actix_web::http::header::{ContentType, HeaderMap};
use actix_web::web::Path;
use actix_web::{error, route, routes, web, HttpRequest, HttpResponse, Responder};
use cctray::CCTrayProjectInfo;
use itertools::Itertools;
use serde::Deserialize;
use std::convert::Into;

#[derive(Deserialize)]
struct ProjectInfo {
    org: String,
    project: String,
}

struct AppState {
    client: reqwest::Client,
    base_url: Option<String>,
}

#[route("/", method = "GET", method = "HEAD")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[routes]
#[get("/{org}/{project}/cctray")]
#[head("/{org}/{project}/cctray")]
#[get("/cctray/{org}/{project}")]
#[head("/cctray/{org}/{project}")]
async fn cctray_project(
    req: HttpRequest,
    info: Path<ProjectInfo>,
    data: web::Data<AppState>,
) -> actix_web::Result<HttpResponse> {
    let base_url = (&data.base_url)
        .clone()
        .unwrap_or(format!("https://{}.semaphoreci.com", info.org));

    let auth_token = get_token(req.headers()).map_err(|e| error::ErrorUnauthorized(e))?;

    let projects = semaphoreci::get_projects(&base_url, &auth_token, &data.client)
        .await
        .map_err(to_actix_error)?;

    let project = projects
        .iter()
        .find(|&p| p.metadata.name == info.project || p.metadata.id == info.project)
        .ok_or_else(|| error::ErrorNotFound(format!("Project {} not found", info.project)))?;

    let pipelines =
        semaphoreci::get_pipelines(&base_url, &project.metadata.id, &auth_token, &data.client)
            .await
            .map_err(to_actix_error)?;

    let cctray_projects = to_cctray_project_info(pipelines, &info.org);

    let xml_fragment = cctray_projects
        .iter()
        .map(to_project_xml_fragment)
        .join("\n");

    Ok(HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(format!("<Projects>{}</Projects>", xml_fragment)))
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

fn to_cctray_project_info(pipelines: Vec<Pipeline>, org: &String) -> Vec<CCTrayProjectInfo> {
    let pipelines_by_name = pipelines.iter().into_group_map_by(|p| p.name.clone());

    Vec::from_iter(
        pipelines_by_name
            .iter()
            .filter(|(name, _pipelines)| **name != String::from(TEMPORARY_PIPELINE_NAME))
            .map(|(name, pipelines)| cctray::get_cctray_project_info(name, pipelines, org))
            .sorted_by_key(|i| i.last_build_time.clone())
            .rev(),
    )
}

fn to_actix_error(e: reqwest::Error) -> actix_web::Error {
    match e.status() {
        Some(reqwest::StatusCode::UNAUTHORIZED) => error::ErrorUnauthorized(e),
        Some(reqwest::StatusCode::NOT_FOUND) => error::ErrorNotFound(e),
        _ => error::ErrorBadGateway(e),
    }
}

fn get_token(headers: &HeaderMap) -> Result<String, &'static str> {
    headers
        .get("authorization")
        .ok_or("Authorization header missing")
        .and_then(|auth_header| {
            auth_header
                .to_str()
                .map_err(|_| "Authorization header is invalid")
        })
        .map(|auth_token| auth_token.replace("Bearer", "").trim().into())
}

fn to_project_xml_fragment(info: &CCTrayProjectInfo) -> String {
    format!(
        "<Project name=\"{}\" activity=\"{}\" lastBuildStatus=\"{}\" lastBuildLabel=\"{}\" lastBuildTime=\"{}\" webUrl=\"{}\"/>",
        info.name, info.activity.as_str(), info.last_build_status.as_str(), info.last_build_label, info.last_build_time, info.web_url
    )
}

pub fn configure_app(cfg: &mut web::ServiceConfig, base_url: &Option<String>) {
    let client = reqwest::Client::new();

    cfg.app_data(web::Data::new(AppState {
        client,
        base_url: base_url.clone(),
    }))
    .service(hello)
    .service(cctray_project);
}

#[cfg(test)]
mod tests {
    use crate::cctray::{Activity, BuildStatus, CCTrayProjectInfo};
    use crate::semaphoreci::Result::{FAILED, PASSED};
    use crate::semaphoreci::{Pipeline, State, Timestamp};
    use crate::to_cctray_project_info;

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
