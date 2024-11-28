use actix_web::http::header::ContentType;
use actix_web::web::Path;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use chrono::DateTime;
use itertools::Itertools;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

#[derive(Deserialize)]
struct ProjectInfo {
    org: String,
    project: String,
}

#[derive(Deserialize)]
struct Timestamp {
    seconds: i64,
    nanos: u32,
}

#[derive(Deserialize)]
struct PipelinesListEntity {
    // DONE | RUNNING
    state: String,
    // PASSED | FAILED
    result: String,
    name: String,
    created_at: Timestamp,
    done_at: Timestamp,
    ppl_id: String,
    wf_id: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/cctray/{org}/{project}")]
async fn cctray(req: HttpRequest, info: Path<ProjectInfo>) -> impl Responder {
    println!("request from {}", req.peer_addr().unwrap());

    let url = format!(
        "https://{}.semaphoreci.com/api/v1alpha/pipelines?project_id={}",
        info.org, info.project
    );
    let client = reqwest::Client::new();
    let result = client
        .get(url)
        .header(
            AUTHORIZATION,

            req.headers()
                .get("authorization")
                .unwrap()
                .to_str()
                .unwrap()
                .replace("Bearer", "Token"),
        )
        .send()
        .await
        .unwrap();

    let pipelines = result.json::<Vec<PipelinesListEntity>>().await.unwrap();

    let projects = pipelines
        .iter()
        .into_group_map_by(|p| p.name.clone())
        .iter()
        .map(|(name, pipelines)| get_project_xml(name, pipelines, &info.org))
        .join("\n");

    HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(format!("<Projects>{}</Projects>", projects))
}

fn get_project_xml(name: &String, pipelines: &Vec<&PipelinesListEntity>, org: &String) -> String {
    let sorted_pipelines: Vec<&&PipelinesListEntity> = pipelines
        .iter()
        .sorted_by_key(|p| p.created_at.seconds)
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
        |p| match p.result.to_uppercase().as_str() {
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

    format!(
        "<Project name=\"{}\" activity=\"{}\" lastBuildStatus=\"{}\" lastBuildLabel=\"{}\" lastBuildTime=\"{}\" webUrl=\"{}\"/>",
        name, activity, last_build_status, last_build_label, last_build_time, web_url
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| App::new().wrap(Logger::default()).service(hello).service(cctray))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
