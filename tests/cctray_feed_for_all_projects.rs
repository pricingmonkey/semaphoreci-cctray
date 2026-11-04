mod support;

use reqwest::header::AUTHORIZATION;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use support::fixtures;
use support::start_app::start_app;

#[actix_web::test]
async fn get_cctray_for_all_projects_in_org() {
    let mock_upstream = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1alpha/projects"))
        .and(header(AUTHORIZATION, "Token : my-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::projects()))
        .mount(&mock_upstream)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1alpha/pipelines"))
        .and(query_param("project_id", "id-prj-a"))
        .and(header(AUTHORIZATION, "Token : my-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::pipelines_project_a()))
        .mount(&mock_upstream)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1alpha/pipelines"))
        .and(query_param("project_id", "id-prj-b"))
        .and(header(AUTHORIZATION, "Token : my-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::pipelines_project_b()))
        .mount(&mock_upstream)
        .await;

    let addr = start_app(&mock_upstream.uri()).await;

    let res = reqwest::Client::new()
        .get(format!("http://{}/any-org/cctray", addr))
        .header(AUTHORIZATION, "Bearer: my-token") // replace with your path
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(res.status(), 200);
    let body = res.text().await.unwrap();
    assert_eq!(body, "<Projects><Project name=\"[project-a] deploy\" activity=\"Sleeping\" lastBuildStatus=\"Failure\" lastBuildLabel=\"7ba0d874-33f0-4495-af7c-8cbccb7f56e5\" lastBuildTime=\"2025-03-28T16:48:30+00:00\" webUrl=\"https://any-org.semaphoreci.com/workflows/eb86a134-3081-406a-8ca1-d6e376cf9a65?pipeline_id=7ba0d874-33f0-4495-af7c-8cbccb7f56e5\"/>
<Project name=\"[project-a] build\" activity=\"Building\" lastBuildStatus=\"Success\" lastBuildLabel=\"87887fa3-ced5-4b9b-aa3c-74e65003e55a\" lastBuildTime=\"2025-03-24T14:35:23+00:00\" webUrl=\"https://any-org.semaphoreci.com/workflows/94505eb4-27d2-4d5c-a616-27077ae9ac32?pipeline_id=0a3e10c1-f046-4959-ae9d-2677a997a72c\"/>
<Project name=\"[project-b] deploy\" activity=\"Sleeping\" lastBuildStatus=\"Success\" lastBuildLabel=\"56c59ffa-f88d-4946-a5d0-586c2ca8a659\" lastBuildTime=\"2025-03-24T12:10:05+00:00\" webUrl=\"https://any-org.semaphoreci.com/workflows/f74afa10-1250-4e0e-bbed-c41c3435be62?pipeline_id=56c59ffa-f88d-4946-a5d0-586c2ca8a659\"/>
<Project name=\"[project-b] build\" activity=\"Sleeping\" lastBuildStatus=\"Success\" lastBuildLabel=\"d58aa83e-6f1c-418c-9176-989ec20c2d62\" lastBuildTime=\"2025-03-24T12:07:43+00:00\" webUrl=\"https://any-org.semaphoreci.com/workflows/f74afa10-1250-4e0e-bbed-c41c3435be62?pipeline_id=d58aa83e-6f1c-418c-9176-989ec20c2d62\"/></Projects>");
}

// #[actix_web::test]
// async fn get_cctray_head() {
//     let mock_upstream = MockServer::start().await;
//
//     Mock::given(method("GET"))
//         .and(path("/api/v1alpha/projects"))
//         .and(header(AUTHORIZATION, "Token : my-token"))
//         .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::projects()))
//         .mount(&mock_upstream)
//         .await;
//
//     Mock::given(method("GET"))
//         .and(path("/api/v1alpha/pipelines"))
//         .and(query_param("project_id", "id-prj-a"))
//         .and(header(AUTHORIZATION, "Token : my-token"))
//         .respond_with(ResponseTemplate::new(200).set_body_json(fixtures::pipelines_project_a()))
//         .mount(&mock_upstream)
//         .await;
//
//     let addr = start_app(&mock_upstream.uri()).await;
//
//     let res = reqwest::Client::new()
//         .head(format!("http://{}/any-org/project-a/cctray", addr))
//         .header(AUTHORIZATION, "Bearer: my-token") // replace with your path
//         .send()
//         .await
//         .expect("failed to send request");
//
//     assert_eq!(res.status(), 200);
//     let body = res.text().await.unwrap();
//     assert_eq!(body, "");
// }
//
// #[actix_web::test]
// async fn returns_401_when_token_missing() {
//     let mock_upstream = MockServer::start().await;
//
//     let addr = start_app(&mock_upstream.uri()).await;
//
//     let res = reqwest::Client::new()
//         .head(format!("http://{}/any-org/my-project/cctray", addr))
//         .send()
//         .await
//         .expect("failed to send request");
//
//     assert_eq!(res.status(), 401);
//     let body = res.text().await.unwrap();
//     assert_eq!(body, "");
// }
//
// #[actix_web::test]
// async fn returns_401_when_upstream_api_returned_401() {
//     let mock_upstream = MockServer::start().await;
//
//     Mock::given(method("GET"))
//         .and(path("/api/v1alpha/projects"))
//         .and(header(AUTHORIZATION, "Token : my-token"))
//         .respond_with(ResponseTemplate::new(401).set_body_string("UNAUTHORIZED"))
//         .mount(&mock_upstream)
//         .await;
//
//     let addr = start_app(&mock_upstream.uri()).await;
//
//     let res = reqwest::Client::new()
//         .head(format!("http://{}/any-org/my-project/cctray", addr))
//         .header(AUTHORIZATION, "Bearer: my-token") // replace with your path
//         .send()
//         .await
//         .expect("failed to send request");
//
//     assert_eq!(res.status(), 401);
//     let body = res.text().await.unwrap();
//     assert_eq!(body, "");
// }
