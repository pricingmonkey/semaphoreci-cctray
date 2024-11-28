use actix_web::http::header::ContentType;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
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
    seconds: i32,
    nanos: i32
}

#[derive(Deserialize)]
struct PipelinesListEntity {
    state: String,
    result: String,
    name: String,
    done_at: Timestamp,
    ppl_id: String
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/cctray/{org}/{project}")]
async fn cctray(req: HttpRequest, info: web::Path<ProjectInfo>) -> impl Responder {
    let url = format!(
        "https://{}.semaphoreci.com/api/v1alpha/pipelines?project_id={}",
        info.org, info.project
    );
    let client = reqwest::Client::new();
    let result = client
        .get(url)
        .header(AUTHORIZATION, req.headers().get("authorization").unwrap().to_str().unwrap())
        .send()
        .await
        .unwrap();

    println!("Status: {}", result.status());

    let pipelines = result.json::<Vec<PipelinesListEntity>>().await.unwrap();

    let projects = pipelines.iter()
        .into_group_map_by(|p| p.name.clone())
        .iter()
        .map(|(name, _)| format!("<Project
                name=\"{}\"
                activity=\"Sleeping\"
                lastBuildStatus=\"Exception\"
                lastBuildLabel=\"8\"
                lastBuildTime=\"2005-09-28T10:30:34.6362160+01:00\"
                webUrl=\"http://mrtickle/ccnet/\"/>", name))
        .join("\n");


    HttpResponse::Ok().content_type(ContentType::xml()).body(
        format!("<Projects>{}</Projects>", projects),
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(cctray))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
