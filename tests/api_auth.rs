use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{
    App, HttpMessage, HttpRequest, HttpResponse, cookie::Key, http::StatusCode, post, test, web,
};
use pushkind_auth::{
    middleware::RequireUserExists, repository::DieselRepository,
    routes::api::api_v1_admin_dashboard,
};
use pushkind_common::{domain::auth::AuthenticatedUser, models::config::CommonServerConfig};

mod common;

#[post("/test-login")]
async fn test_login(request: HttpRequest, body: web::Bytes) -> HttpResponse {
    let token = String::from_utf8(body.to_vec()).expect("test login token must be valid UTF-8");
    Identity::login(&request.extensions(), token).expect("test login should attach identity");
    HttpResponse::Ok().finish()
}

#[actix_web::test]
async fn api_scope_rejects_deleted_user_with_valid_session() {
    let test_db = common::TestDb::new();
    let repo = DieselRepository::new(test_db.pool());
    let secret = "test-secret";

    let mut user = AuthenticatedUser {
        sub: "999".to_string(),
        email: "deleted@example.com".to_string(),
        hub_id: 1,
        name: "Deleted Admin".to_string(),
        roles: vec!["admin".to_string()],
        exp: 0,
    };
    user.set_expiration(1);
    let token = user.to_jwt(secret).expect("test JWT should be created");

    let app = test::init_service(
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
                    .cookie_secure(false)
                    .build(),
            )
            .app_data(web::Data::new(CommonServerConfig {
                auth_service_url: "/auth/signin".to_string(),
                secret: secret.to_string(),
            }))
            .app_data(web::Data::new(repo))
            .service(test_login)
            .service(
                web::scope("/api")
                    .wrap(RequireUserExists)
                    .service(api_v1_admin_dashboard),
            ),
    )
    .await;

    let login_response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/test-login")
            .set_payload(token)
            .to_request(),
    )
    .await;

    assert_eq!(login_response.status(), StatusCode::OK);

    let session_cookie = login_response
        .response()
        .cookies()
        .next()
        .expect("login should set a session cookie")
        .to_owned();

    let error = test::try_call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/dashboard")
            .cookie(session_cookie)
            .to_request(),
    )
    .await
    .expect_err("deleted users should be rejected by the api middleware");

    assert_eq!(
        error.as_response_error().status_code(),
        StatusCode::UNAUTHORIZED
    );
}
