use std::net::{SocketAddr, TcpListener};
use actix_web::{App, HttpServer};
use semaphoreci_cctray::configure_app;

pub async fn start_app(ci_base_uri: &String) -> SocketAddr {
    std::env::set_var("CI_BASE_URL", ci_base_uri);

    // Bind to a random free port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let server = HttpServer::new(|| {
        App::new().configure(configure_app) // your app factory
    })
        .listen(listener)
        .unwrap()
        .run();

    // Start the server in the background
    actix_web::rt::spawn(server);

    // Let the server warm up a bit
    actix_web::rt::time::sleep(std::time::Duration::from_millis(100)).await;
    addr
}