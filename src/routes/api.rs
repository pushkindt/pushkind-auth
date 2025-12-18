//! Actix Web route handlers for versioned API endpoints.

use actix_web::{HttpResponse, Responder, get, web};
use log::error;
use pushkind_common::domain::auth::AuthenticatedUser;
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

/// Lists users for the current hub with optional filters via `GET /v1/users`.
#[get("/v1/users")]
pub async fn api_v1_users(
    params: web::Query<ApiV1UsersQueryParams>,
    user: AuthenticatedUser,
    repo: web::Data<DieselRepository>,
) -> impl Responder {
    let users = api_service::list_users(params.into_inner(), user.hub_id, repo.get_ref());

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            error!("Failed to list users: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
