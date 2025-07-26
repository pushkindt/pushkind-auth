use actix_web::{HttpResponse, Responder, get, web};
use log::error;
use pushkind_common::db::DbPool;
use pushkind_common::models::auth::AuthenticatedUser;
use pushkind_common::pagination::DEFAULT_ITEMS_PER_PAGE;
use serde::Deserialize;

use crate::repository::user::DieselUserRepository;
use crate::repository::{UserListQuery, UserReader};

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
                Ok(Some(found_user)) if user.hub_id == found_user.user.hub_id => {
                    HttpResponse::Ok().json(AuthenticatedUser::from(found_user.user))
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
    role: Option<String>,
    query: Option<String>,
    page: Option<usize>,
}

#[get("/v1/users")]
pub async fn api_v1_users(
    params: web::Query<ApiV1UsersQueryParams>,
    user: AuthenticatedUser,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let repo = DieselUserRepository::new(&pool);

    let mut list_query = UserListQuery::new(user.hub_id);

    if let Some(role) = &params.role {
        list_query = list_query.role(role);
    }

    if let Some(page) = params.page {
        list_query = list_query.paginate(page, DEFAULT_ITEMS_PER_PAGE);
    }

    let result = match &params.query {
        Some(query) if !query.is_empty() => {
            list_query = list_query.search(query);
            repo.search(list_query)
        }
        _ => repo.list(list_query),
    };

    match result {
        Ok((_total, users_with_roles)) => {
            let users: Vec<AuthenticatedUser> = users_with_roles
                .into_iter()
                .map(|user_with_roles| AuthenticatedUser::from(user_with_roles.user))
                .collect();

            HttpResponse::Ok().json(users)
        }
        Err(e) => {
            error!("Failed to list users: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
