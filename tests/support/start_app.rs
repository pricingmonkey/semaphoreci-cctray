use actix_web::{App, HttpServer};
use semaphoreci_cctray::configure_app;
use std::net::{SocketAddr, TcpListener};

pub async fn start_app(ci_base_uri: &String) -> SocketAddr {
    // Bind to a random free port
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let op = Some(ci_base_uri.clone());

    let server = HttpServer::new(move || {
        App::new().configure(|cfg| configure_app(cfg, &op))
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
