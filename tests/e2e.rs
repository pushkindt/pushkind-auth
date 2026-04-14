use reqwest::{StatusCode, header};
use serde_json::Value;

mod common;

use pushkind_auth::{
    domain::types::{HubId, MenuId, RoleId, UserEmail, UserId, UserPassword},
    repository::{DieselRepository, HubReader, MenuReader, RoleReader, UserReader},
};

fn login_form_body(email: &str, password: &str, hub_id: i32) -> String {
    format!("email={email}&password={password}&hub_id={hub_id}")
}

fn mutation_form_body(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

async fn response_json(response: reqwest::Response) -> Value {
    let body = response
        .text()
        .await
        .expect("Response body should be readable.");
    serde_json::from_str(&body).expect("Response body should be valid JSON.")
}

async fn login_as(
    client: &reqwest::Client,
    address: &str,
    email: &str,
    password: &str,
    hub_id: i32,
) {
    let login_response = client
        .post(format!("{address}/auth/login"))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(login_form_body(email, password, hub_id))
        .send()
        .await
        .expect("Failed to submit login form.");

    assert_eq!(login_response.status(), StatusCode::OK);
    let login_payload = response_json(login_response).await;
    assert_eq!(login_payload["redirect_to"], "/");
}

#[actix_web::test]
async fn test_health() {
    let app = common::spawn_app().await;

    let client = common::build_reqwest_client();

    let response = client
        .get(format!("{}/health", app.address()))
        .send()
        .await
        .expect("Failed to request the health page.");

    assert_eq!(response.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_signin_page_get_for_logged_out_user() {
    let app = common::spawn_app().await;

    common::setup_hub_with_users(app.db_pool());

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

#[actix_web::test]
async fn test_admin_user_full_management_story() {
    let app = common::spawn_app().await;
    let seeded = common::setup_hub_with_users(app.db_pool());
    let client = common::build_reqwest_client();
    let repo = DieselRepository::new(app.db_pool());

    login_as(
        &client,
        app.address(),
        common::ADMIN_EMAIL,
        common::ADMIN_PASSWORD,
        seeded.hub_id,
    )
    .await;

    let signin_response = client
        .get(format!("{}/auth/signin", app.address()))
        .send()
        .await
        .expect("Failed to request sign-in page for authenticated admin.");

    assert_eq!(signin_response.status(), StatusCode::OK);
    assert_eq!(
        signin_response.url().as_str(),
        format!("{}/", app.address())
    );
    let signin_html = signin_response
        .text()
        .await
        .expect("Redirected admin sign-in response should be readable.");
    assert!(signin_html.contains("main-admin.tsx"));

    let index_response = client
        .get(format!("{}/", app.address()))
        .send()
        .await
        .expect("Failed to request dashboard for authenticated admin.");

    assert_eq!(index_response.status(), StatusCode::OK);
    let index_html = index_response
        .text()
        .await
        .expect("Admin dashboard response should be readable.");
    assert!(index_html.contains("main-admin.tsx"));

    let admin_dashboard_response = client
        .get(format!("{}/api/v1/admin/dashboard", app.address()))
        .send()
        .await
        .expect("Failed to request admin dashboard API.");

    assert_eq!(admin_dashboard_response.status(), StatusCode::OK);

    let add_role_response = client
        .post(format!("{}/admin/role/add", app.address()))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(mutation_form_body(&[("name", "editor")]))
        .send()
        .await
        .expect("Failed to create role.");

    assert_eq!(add_role_response.status(), StatusCode::CREATED);
    let created_role = repo
        .get_role_by_name("editor")
        .expect("Role lookup should succeed.")
        .expect("Role should exist after creation.");

    let add_hub_response = client
        .post(format!("{}/admin/hub/add", app.address()))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(mutation_form_body(&[("name", "branch")]))
        .send()
        .await
        .expect("Failed to create hub.");

    assert_eq!(add_hub_response.status(), StatusCode::CREATED);
    let created_hub = repo
        .get_hub_by_name("branch")
        .expect("Hub lookup should succeed.")
        .expect("Hub should exist after creation.");

    let add_menu_response = client
        .post(format!("{}/admin/menu/add", app.address()))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(mutation_form_body(&[
            ("name", "docs"),
            ("url", "https://docs.example.com"),
        ]))
        .send()
        .await
        .expect("Failed to create menu item.");

    assert_eq!(add_menu_response.status(), StatusCode::CREATED);
    let created_menu = repo
        .list_menu(HubId::new(seeded.hub_id).unwrap())
        .expect("Menu lookup should succeed.")
        .into_iter()
        .find(|menu| menu.name.as_str() == "docs")
        .expect("Menu item should exist after creation.");

    let modal_response = client
        .post(format!(
            "{}/admin/user/modal/{}",
            app.address(),
            seeded.user_id
        ))
        .send()
        .await
        .expect("Failed to fetch admin user modal.");

    assert_eq!(modal_response.status(), StatusCode::OK);
    let modal_payload = response_json(modal_response).await;
    assert_eq!(modal_payload["user"]["id"], seeded.user_id);
    assert_eq!(modal_payload["user"]["email"], common::USER_EMAIL);
    assert!(
        modal_payload["roles"]
            .as_array()
            .expect("Roles should be present in modal response.")
            .iter()
            .any(|role| role["name"] == "editor")
    );

    let update_user_response = client
        .post(format!(
            "{}/admin/user/update/{}",
            app.address(),
            seeded.user_id
        ))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(mutation_form_body(&[
            ("name", "managed-user"),
            ("password", "new-user-password"),
            ("roles", &created_role.id.get().to_string()),
        ]))
        .send()
        .await
        .expect("Failed to update user as admin.");

    assert_eq!(update_user_response.status(), StatusCode::OK);
    let updated_user = repo
        .get_user_by_id(
            UserId::new(seeded.user_id).unwrap(),
            HubId::new(seeded.hub_id).unwrap(),
        )
        .expect("Updated user lookup should succeed.")
        .expect("Updated user should still exist.");
    assert_eq!(
        updated_user.user.name.as_ref().map(|name| name.as_str()),
        Some("managed-user")
    );
    assert_eq!(updated_user.roles.len(), 1);
    assert_eq!(updated_user.roles[0].name.as_str(), "editor");
    let relogin = repo
        .login(
            &UserEmail::new(common::USER_EMAIL).unwrap(),
            &UserPassword::new("new-user-password").unwrap(),
            HubId::new(seeded.hub_id).unwrap(),
        )
        .expect("Updated user login lookup should succeed.");
    assert!(relogin.is_some());

    let delete_menu_response = client
        .post(format!(
            "{}/admin/menu/delete/{}",
            app.address(),
            created_menu.id.get()
        ))
        .send()
        .await
        .expect("Failed to delete menu item.");

    assert_eq!(delete_menu_response.status(), StatusCode::OK);
    assert!(
        repo.get_menu_by_id(
            MenuId::new(created_menu.id.get()).unwrap(),
            HubId::new(seeded.hub_id).unwrap(),
        )
        .expect("Deleted menu lookup should succeed.")
        .is_none()
    );

    let delete_hub_response = client
        .post(format!(
            "{}/admin/hub/delete/{}",
            app.address(),
            created_hub.id.get()
        ))
        .send()
        .await
        .expect("Failed to delete hub.");

    assert_eq!(delete_hub_response.status(), StatusCode::OK);
    assert!(
        repo.get_hub_by_name("branch")
            .expect("Deleted hub lookup should succeed.")
            .is_none()
    );

    let delete_role_response = client
        .post(format!(
            "{}/admin/role/delete/{}",
            app.address(),
            created_role.id.get()
        ))
        .send()
        .await
        .expect("Failed to delete role.");

    assert_eq!(delete_role_response.status(), StatusCode::OK);
    assert!(
        repo.get_role_by_id(RoleId::new(created_role.id.get()).unwrap())
            .expect("Deleted role lookup should succeed.")
            .is_none()
    );
}

#[actix_web::test]
async fn test_non_admin_user_self_service_story() {
    let app = common::spawn_app().await;
    let seeded = common::setup_hub_with_users(app.db_pool());
    let client = common::build_reqwest_client();
    let repo = DieselRepository::new(app.db_pool());

    login_as(
        &client,
        app.address(),
        common::USER_EMAIL,
        common::USER_PASSWORD,
        seeded.hub_id,
    )
    .await;

    let signin_response = client
        .get(format!("{}/auth/signin", app.address()))
        .send()
        .await
        .expect("Failed to request sign-in page for authenticated user.");

    assert_eq!(signin_response.status(), StatusCode::OK);
    assert_eq!(
        signin_response.url().as_str(),
        format!("{}/", app.address())
    );
    let signin_html = signin_response
        .text()
        .await
        .expect("Redirected regular user sign-in response should be readable.");
    assert!(signin_html.contains("main-basic.tsx"));

    let index_response = client
        .get(format!("{}/", app.address()))
        .send()
        .await
        .expect("Failed to request dashboard for authenticated user.");

    assert_eq!(index_response.status(), StatusCode::OK);
    let index_html = index_response
        .text()
        .await
        .expect("Basic dashboard response should be readable.");
    assert!(index_html.contains("main-basic.tsx"));

    let admin_dashboard_response = client
        .get(format!("{}/api/v1/admin/dashboard", app.address()))
        .send()
        .await
        .expect("Failed to request admin dashboard API as regular user.");

    assert_eq!(admin_dashboard_response.status(), StatusCode::FORBIDDEN);

    let save_user_response = client
        .post(format!("{}/user/save", app.address()))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(mutation_form_body(&[
            ("name", "self-edited-user"),
            ("password", "updated-self-password"),
        ]))
        .send()
        .await
        .expect("Failed to update personal profile.");

    assert_eq!(save_user_response.status(), StatusCode::OK);
    let updated_user = repo
        .get_user_by_id(
            UserId::new(seeded.user_id).unwrap(),
            HubId::new(seeded.hub_id).unwrap(),
        )
        .expect("Updated self user lookup should succeed.")
        .expect("Updated self user should still exist.");
    assert_eq!(
        updated_user.user.name.as_ref().map(|name| name.as_str()),
        Some("self-edited-user")
    );
    let relogin = repo
        .login(
            &UserEmail::new(common::USER_EMAIL).unwrap(),
            &UserPassword::new("updated-self-password").unwrap(),
            HubId::new(seeded.hub_id).unwrap(),
        )
        .expect("Self-updated user login lookup should succeed.");
    assert!(relogin.is_some());

    let forbidden_role_response = client
        .post(format!("{}/admin/role/add", app.address()))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(mutation_form_body(&[("name", "forbidden-role")]))
        .send()
        .await
        .expect("Failed to exercise forbidden admin mutation.");

    assert_eq!(forbidden_role_response.status(), StatusCode::FORBIDDEN);
}
