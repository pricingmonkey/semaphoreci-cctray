use actix_web::{App, HttpServer};
use reqwest::header::AUTHORIZATION;
use semaphoreci_cctray::configure_app;
use std::net::TcpListener;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod fixtures;

#[actix_web::test]
async fn test_cctray_feed_for_single_project() {
    let mock_upstream = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1alpha/pipelines"))
        .and(query_param("project_id", "my-project"))
        .and(header(AUTHORIZATION, "Token : my-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::pipeline_response_body()))
        .mount(&mock_upstream)
        .await;

    std::env::set_var("CI_BASE_URL", &mock_upstream.uri());

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

    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://{}/cctray/any-org/my-project", addr))
        .header(AUTHORIZATION, "Bearer: my-token") // replace with your path
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert_eq!(body, "<Projects><Project name=\"deploy\" activity=\"Sleeping\" lastBuildStatus=\"Failure\" lastBuildLabel=\"7ba0d874-33f0-4495-af7c-8cbccb7f56e5\" lastBuildTime=\"2025-03-28T16:48:30+00:00\" webUrl=\"https://any-org.semaphoreci.com/workflows/eb86a134-3081-406a-8ca1-d6e376cf9a65?pipeline_id=7ba0d874-33f0-4495-af7c-8cbccb7f56e5\"/>
<Project name=\"build\" activity=\"Building\" lastBuildStatus=\"Success\" lastBuildLabel=\"87887fa3-ced5-4b9b-aa3c-74e65003e55a\" lastBuildTime=\"2025-03-24T14:35:23+00:00\" webUrl=\"https://any-org.semaphoreci.com/workflows/94505eb4-27d2-4d5c-a616-27077ae9ac32?pipeline_id=0a3e10c1-f046-4959-ae9d-2677a997a72c\"/></Projects>");
}
