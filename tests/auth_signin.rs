use reqwest::{StatusCode, header};

mod common;

#[ignore]
#[actix_web::test]
async fn test_signin_page_get() {
    let app = common::spawn_app().await;

    common::setup_hub_with_admin(app.db_pool());

    let client = common::build_reqwest_client();

    let response = client
        .get(format!("{}/auth/signin", app.address()))
        .send()
        .await
        .expect("Failed to request the sign-in page.");

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .expect("sign-in response should include a valid content type");
    assert!(content_type.starts_with("text/html"));
}
