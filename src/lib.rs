mod pipeline;

use crate::pipeline::Pipeline;
use actix_web::http::header::{ContentType, HeaderMap};
use actix_web::web::Path;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use chrono::DateTime;
use itertools::Itertools;
use serde::Deserialize;
use std::convert::Into;
use std::env;

#[derive(Deserialize)]
struct ProjectInfo {
    org: String,
    project: String,
}

struct CCTrayProjectInfo {
    name: String,
    activity: String,
    last_build_status: String,
    last_build_label: String,
    last_build_time: String,
    web_url: String,
}

struct AppState {
    client: reqwest::Client,
    base_url: Option<String>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/cctray/{org}/{project}")]
async fn cctray(
    req: HttpRequest,
    info: Path<ProjectInfo>,
    data: web::Data<AppState>,
) -> impl Responder {
    let base_url = (&data.base_url)
        .clone()
        .unwrap_or(format!("https://{}.semaphoreci.com", info.org));

    let auth_token = get_token(req.headers());

    let pipelines = match auth_token {
        Ok(token) => pipeline::get_pipeline(&base_url, &info.project, &token, &data.client).await,
        Err(e) => {
            println!("{}", e);
            Err("Unauthorized")
        }
    };

    match pipelines {
        Err("Not found") => HttpResponse::NotFound().finish(),
        Err("Forbidden") => HttpResponse::Forbidden().finish(),
        Err("Unauthorized") => HttpResponse::Unauthorized().finish(),
        Ok(pipelines) => {
            let projects = pipelines
                .iter()
                .into_group_map_by(|p| p.name.clone())
                .iter()
                .map(|(name, pipelines)| get_cctray_project_info(name, pipelines, &info.org))
                .sorted_by_key(|i| i.last_build_time.clone())
                .rev()
                .map(to_project_xml_fragment)
                .join("\n");

            HttpResponse::Ok()
                .content_type(ContentType::xml())
                .body(format!("<Projects>{}</Projects>", projects))
        }
        _ => HttpResponse::InternalServerError().finish(),
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
    let previous_pipeline = sorted_pipelines.get(1);
    let last_completed_pipeline = if latest_pipeline.state.eq("DONE") {
        Some(latest_pipeline)
    } else {
        previous_pipeline
    };

    let activity = if latest_pipeline.state.eq("RUNNING") {
        "Building"
    } else {
        "Sleeping"
    };
    let last_build_status = last_completed_pipeline
        .and_then(|p| p.result.clone())
        .and_then(|result| match result.to_uppercase().as_str() {
            "PASSED" => Some("Success"),
            "FAILED" => Some("Failure"),
            _ => None,
        })
        .unwrap_or("Unknown");

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
        activity: activity.to_string(),
        last_build_status: last_build_status.to_string(),
        last_build_label: last_build_label.to_string(),
        last_build_time,
        web_url,
    }
}

fn to_project_xml_fragment(info: CCTrayProjectInfo) -> String {
    format!(
        "<Project name=\"{}\" activity=\"{}\" lastBuildStatus=\"{}\" lastBuildLabel=\"{}\" lastBuildTime=\"{}\" webUrl=\"{}\"/>",
        info.name, info.activity, info.last_build_status, info.last_build_label, info.last_build_time, info.web_url
    )
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    let client = reqwest::Client::new();
    let base_url = env::var("CI_BASE_URL").ok();

    cfg.app_data(web::Data::new(AppState { client, base_url }))
        .service(hello)
        .service(cctray);
}
