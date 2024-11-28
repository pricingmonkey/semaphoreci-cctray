use actix_web::http::header::ContentType;
use actix_web::middleware::Logger;
use actix_web::web::Path;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::DateTime;
use itertools::Itertools;
use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
struct ProjectInfo {
    org: String,
    project: String,
}

#[derive(Deserialize)]
struct Timestamp {
    seconds: i64,
}

#[derive(Deserialize)]
struct PipelinesListEntity {
    // DONE | RUNNING
    state: String,
    // PASSED | FAILED
    result: Option<String>,
    name: String,
    created_at: Timestamp,
    done_at: Timestamp,
    ppl_id: String,
    wf_id: String,
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
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/cctray/{org}/{project}")]
async fn cctray(req: HttpRequest, info: Path<ProjectInfo>, data: web::Data<AppState>) -> impl Responder {
    let maybe_auth_header = req.headers().get("authorization");

    if None == maybe_auth_header {
        return HttpResponse::Unauthorized().body("Unauthorized!");
    }

    let url = format!(
        "https://{}.semaphoreci.com/api/v1alpha/pipelines?project_id={}",
        info.org, info.project
    );

    let result = data.client
        .get(url)
        .header(
            AUTHORIZATION,
            maybe_auth_header
                .unwrap()
                .to_str()
                .unwrap()
                .replace("Bearer", "Token"),
        )
        .send()
        .await
        .unwrap();

    if result.status() != StatusCode::OK {
        return match result.status() {
            StatusCode::NOT_FOUND => HttpResponse::NotFound().finish(),
            StatusCode::FORBIDDEN => HttpResponse::Forbidden().finish(),
            StatusCode::UNAUTHORIZED => HttpResponse::Unauthorized().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }

    let pipelines = result.json::<Vec<PipelinesListEntity>>().await.unwrap();

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

fn get_cctray_project_info(
    name: &String,
    pipelines: &Vec<&PipelinesListEntity>,
    org: &String,
) -> CCTrayProjectInfo {
    let sorted_pipelines: Vec<&&PipelinesListEntity> = pipelines
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
    let last_build_status = last_completed_pipeline.map_or_else(
        || "Unknown",
        |p| match p.result.clone().unwrap_or(String::from("")).to_uppercase().as_str() {
            "PASSED" => "Success",
            "FAILED" => "Failure",
            _ => "Unknown",
        },
    );
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));


    HttpServer::new(|| {
        let client = reqwest::Client::new();

        App::new()
            .app_data(web::Data::new(AppState { client }))
            .wrap(Logger::default())
            .service(hello)
            .service(cctray)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
