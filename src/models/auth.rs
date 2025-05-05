use std::future::{Ready, ready};

use actix_identity::Identity;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web;
use actix_web::{Error, FromRequest, HttpRequest, dev::Payload};
use serde::Serialize;

use crate::db::DbPool;
use crate::domain::user::User;
use crate::repository::UserRepository;
use crate::repository::user::DieselUserRepository;

#[derive(Serialize)]
pub struct AuthenticatedUser(pub User);

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let identity = Identity::from_request(req, &mut Payload::None)
            .into_inner()
            .map(|i| i.id().ok());

        let pool = req.app_data::<web::Data<DbPool>>();

        if let (Ok(Some(uid)), Some(pool)) = (identity, pool) {
            let mut conn = match pool.get() {
                Ok(conn) => conn,
                Err(_) => {
                    return ready(Err(ErrorInternalServerError("DB connection error")));
                }
            };

            let uid: i32 = match uid.parse() {
                Ok(uid) => uid,
                Err(_) => return ready(Err(ErrorUnauthorized("Invalid user"))),
            };

            let mut repo = DieselUserRepository::new(&mut conn);

            match repo.get_by_id(uid) {
                Ok(Some(user)) => return ready(Ok(AuthenticatedUser(user))),
                _ => return ready(Err(ErrorUnauthorized("Invalid user"))),
            }
        }
        ready(Err(ErrorUnauthorized("Unauthorized")))
    }
}
