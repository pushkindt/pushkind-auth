//! Actix Web route handlers for versioned API endpoints.

use actix_web::{HttpResponse, Responder, get, web};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::services::errors::ServiceError;
use serde::Deserialize;

use crate::dto::api::ApiV1UsersQueryParams;
use crate::repository::DieselRepository;
use crate::services::api as api_service;

#[derive(Deserialize)]
struct ApiV1IdParams {
    id: Option<i32>,
}

/// Returns the current user or the one specified by id via `GET /v1/id`.
#[get("/v1/id")]
pub async fn api_v1_id(
    params: web::Query<ApiV1IdParams>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    match api_service::get_user_by_optional_id(params.id, current_user, repo.get_ref()) {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!("Failed to get user: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Lists hubs via `GET /v1/hubs`.
#[get("/v1/hubs")]
pub async fn api_v1_hubs(repo: web::Data<DieselRepository>) -> impl Responder {
    match api_service::list_hubs(repo.get_ref()) {
        Ok(hubs) => HttpResponse::Ok().json(hubs),
        Err(e) => {
            error!("Failed to list hubs: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Returns the authenticated user's IAM payload via `GET /v1/iam`.
#[get("/v1/iam")]
pub async fn api_v1_iam(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    match api_service::get_iam(current_user, repo.get_ref()) {
        Ok(iam) => HttpResponse::Ok().json(iam),
        Err(ServiceError::NotFound) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!("Failed to build IAM payload: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Lists menu items for the requested hub via `GET /v1/hubs/{hub_id}/menu-items`.
#[get("/v1/hubs/{hub_id}/menu-items")]
pub async fn api_v1_hub_menu_items(
    hub_id: web::Path<i32>,
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    match api_service::list_hub_menu_items(hub_id.into_inner(), &current_user, repo.get_ref()) {
        Ok(menu_items) => HttpResponse::Ok().json(menu_items),
        Err(ServiceError::Unauthorized) => HttpResponse::Forbidden().finish(),
        Err(e) => {
            error!("Failed to list hub menu items: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Returns admin-only dashboard collections via `GET /v1/admin/dashboard`.
#[get("/v1/admin/dashboard")]
pub async fn api_v1_admin_dashboard(
    current_user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    match api_service::get_admin_dashboard_data(&current_user, repo.get_ref()) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(ServiceError::Unauthorized) => HttpResponse::Forbidden().finish(),
        Err(e) => {
            error!("Failed to build admin dashboard payload: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Lists users for the current hub with optional filters via `GET /v1/users`.
#[get("/v1/users")]
pub async fn api_v1_users(
    params: web::Query<ApiV1UsersQueryParams>,
    user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let users = api_service::list_users(params.into_inner(), &user, repo.get_ref());

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            error!("Failed to list users: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
