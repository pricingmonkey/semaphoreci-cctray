mod pipeline;
mod cctray;

use actix_web::http::header::{ContentType, HeaderMap};
use actix_web::web::Path;
use actix_web::{route, web, HttpRequest, HttpResponse, Responder};
use cctray::CCTrayProjectInfo;
use itertools::Itertools;
use serde::Deserialize;
use std::convert::Into;
use std::env;

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

#[route("/cctray/{org}/{project}", method = "GET", method = "HEAD")]
async fn cctray_project(
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
                .map(|(name, pipelines)| cctray::get_cctray_project_info(name, pipelines, &info.org))
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

fn to_project_xml_fragment(info: CCTrayProjectInfo) -> String {
    format!(
        "<Project name=\"{}\" activity=\"{}\" lastBuildStatus=\"{}\" lastBuildLabel=\"{}\" lastBuildTime=\"{}\" webUrl=\"{}\"/>",
        info.name, info.activity.as_str(), info.last_build_status.as_str(), info.last_build_label, info.last_build_time, info.web_url
    )
}

pub fn configure_app(cfg: &mut web::ServiceConfig) {
    let client = reqwest::Client::new();
    let base_url = env::var("CI_BASE_URL").ok();

    cfg.app_data(web::Data::new(AppState { client, base_url }))
        .service(hello)
        .service(cctray_project);
}
