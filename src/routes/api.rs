use actix_web::{HttpResponse, Responder, get, web};
use log::error;
use serde::Deserialize;

use crate::db::DbPool;
use crate::models::auth::AuthenticatedUser;
use crate::repository::UserRepository;
use crate::repository::user::DieselUserRepository;

#[derive(Deserialize)]
struct ApiV1IdParams {
    id: Option<i32>,
}

#[get("/v1/id")]
pub async fn api_v1_id(
    params: web::Query<ApiV1IdParams>,
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    match params.id {
        Some(id) => {
            let repo = DieselUserRepository::new(&pool);
            match repo.get_by_id(id) {
                Ok(Some(found_user)) if user.hub_id == found_user.hub_id => {
                    HttpResponse::Ok().json(AuthenticatedUser::from(found_user))
                }
                Err(e) => {
                    error!("Failed to get user: {e}");
                    HttpResponse::InternalServerError().finish()
                }
                _ => HttpResponse::NotFound().finish(),
            }
        }
        None => HttpResponse::Ok().json(user),
    }
}

#[derive(Deserialize)]
struct ApiV1UsersQueryParams {
    role: String,
    query: String,
}

#[get("/v1/users")]
pub async fn api_v1_users(
    params: web::Query<ApiV1UsersQueryParams>,
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let repo = DieselUserRepository::new(&pool);

    match repo.search(user.hub_id, &params.role, &params.query) {
        Ok(users) => {
            let users: Vec<AuthenticatedUser> =
                users.into_iter().map(AuthenticatedUser::from).collect();

            HttpResponse::Ok().json(users)
        }
        Err(e) => {
            error!("Failed to list users: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
