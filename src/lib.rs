mod cctray;
mod semaphoreci;

use actix_web::http::header::{ContentType, HeaderMap};
use actix_web::web::Path;
use actix_web::{error, route, routes, web, HttpRequest, HttpResponse, Responder};
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

    let cctray_projects = cctray::to_cctray_project_info(pipelines, &info.org);

    Ok(HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(cctray::serialize(cctray_projects)))
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

pub fn configure_app(cfg: &mut web::ServiceConfig, base_url: &Option<String>) {
    let client = reqwest::Client::new();

    cfg.app_data(web::Data::new(AppState {
        client,
        base_url: base_url.clone(),
    }))
    .service(hello)
    .service(cctray_project);
}

