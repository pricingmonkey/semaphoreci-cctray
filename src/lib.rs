mod pipeline;
mod cctray;

use actix_web::http::header::{ContentType, HeaderMap};
use actix_web::web::Path;
use actix_web::{error, route, web, HttpRequest, HttpResponse, Responder};
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

#[route("/cctray/{org}/{project}", method = "GET", method = "HEAD")]
async fn cctray_project(
    req: HttpRequest,
    info: Path<ProjectInfo>,
    data: web::Data<AppState>,
) -> actix_web::Result<HttpResponse> {
    let base_url = (&data.base_url)
        .clone()
        .unwrap_or(format!("https://{}.semaphoreci.com", info.org));

    let auth_token = get_token(req.headers()).map_err(|e| error::ErrorUnauthorized(e))?;

    let pipelines = pipeline::get_pipeline(&base_url, &info.project, &auth_token, &data.client).await.map_err(|e| {
        match e.status() {
            Some(reqwest::StatusCode::UNAUTHORIZED) => error::ErrorUnauthorized(e),
            Some(reqwest::StatusCode::NOT_FOUND) => error::ErrorNotFound(e),
            _ => error::ErrorInternalServerError(e)
        }
    })?;

    let projects = pipelines
        .iter()
        .into_group_map_by(|p| p.name.clone())
        .iter()
        .map(|(name, pipelines)| cctray::get_cctray_project_info(name, pipelines, &info.org))
        .sorted_by_key(|i| i.last_build_time.clone())
        .rev()
        .map(to_project_xml_fragment)
        .join("\n");

    Ok(HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(format!("<Projects>{}</Projects>", projects))
    )
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

pub fn configure_app(cfg: &mut web::ServiceConfig, base_url: &Option<String>) {
    let client = reqwest::Client::new();

    cfg.app_data(web::Data::new(AppState { client, base_url: base_url.clone() }))
        .service(hello)
        .service(cctray_project);
}
