use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::header::ContentType;
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    org: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/cctray/{org}")]
async fn cctray(info: web::Path<Info>) -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::xml()).body(
        "<Projects>
            <Project
                name=\"SvnTest\"
                activity=\"Sleeping\"
                lastBuildStatus=\"Exception\"
                lastBuildLabel=\"8\"
                lastBuildTime=\"2005-09-28T10:30:34.6362160+01:00\"
                nextBuildTime=\"2005-10-04T14:31:52.4509248+01:00\"
                webUrl=\"http://mrtickle/ccnet/\"/>
            <Project
                name=\"HelloWorld\"
                activity=\"Sleeping\"
                lastBuildStatus=\"Success\"
                lastBuildLabel=\"13\"
                lastBuildTime=\"2005-09-15T17:33:07.6447696+01:00\"
                nextBuildTime=\"2005-10-04T14:31:51.7799600+01:00\"
                webUrl=\"http://mrtickle/ccnet/\"/>
        </Projects>",
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(cctray))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
