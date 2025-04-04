use actix_web::{App, HttpServer};
use semaphoreci_cctray::configure_app;
use std::env;
use actix_web::middleware::Logger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(8080);
    let bind_ip = env::var("BIND_IP")
        .ok()
        .unwrap_or(String::from("127.0.0.1"));
    let ci_base_url = env::var("CI_BASE_URL").ok();

    HttpServer::new(move || App::new().wrap(Logger::default()).configure(|cfg| configure_app(cfg, &ci_base_url)))
        .bind((bind_ip, port))?
        .run()
        .await
}
