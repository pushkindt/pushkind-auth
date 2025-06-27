use std::future::{Ready, ready};

use actix_identity::Identity;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web;
use actix_web::{Error, FromRequest, HttpRequest, dev::Payload};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::db::DbPool;
use crate::domain::role::Role;
use crate::domain::user::User;
use crate::models::config::ServerConfig;
use crate::repository::UserRepository;
use crate::repository::user::DieselUserRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub sub: String, // subject (user ID or UUID)
    pub email: String,
    pub hub_id: i32,
    pub name: String,
    pub roles: Vec<String>,
    pub exp: usize, // expiration as timestamp
}

impl AuthenticatedUser {
    pub fn set_expiration(&mut self, days: i64) {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(days))
            .expect("valid timestamp")
            .timestamp() as usize;
        self.exp = expiration;
    }

    pub fn from_user(user: &User, roles: &[Role]) -> Self {
        let mut result = Self {
            sub: user.id.to_string(),
            email: user.email.clone(),
            hub_id: user.hub_id,
            name: user.name.clone().unwrap_or_default(),
            roles: roles.iter().map(|r| r.name.clone()).collect(),
            exp: 0,
        };
        result.set_expiration(7);
        result
    }

    pub fn to_jwt(&mut self, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
        self.set_expiration(7);
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_ref()),
        )
    }
    fn from_jwt(token: &str, secret: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let validation = jsonwebtoken::Validation::default();
        let token_data = jsonwebtoken::decode::<Self>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        )?;
        Ok(token_data.claims)
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let identity = Identity::from_request(req, &mut Payload::None)
            .into_inner()
            .map(|i| i.id().ok());

        let pool = req.app_data::<web::Data<DbPool>>();

        let server_config = req.app_data::<web::Data<ServerConfig>>();

        let server_config = match server_config {
            Some(config) => config,
            None => return ready(Err(ErrorInternalServerError("Server config not found"))),
        };

        if let (Ok(Some(uid)), Some(pool)) = (identity, pool) {
            let claims = AuthenticatedUser::from_jwt(&uid, &server_config.secret);

            let claims = match claims {
                Ok(claims) => claims,
                Err(_) => return ready(Err(ErrorUnauthorized("Invalid user"))),
            };

            let uid: i32 = match claims.sub.parse() {
                Ok(uid) => uid,
                Err(_) => return ready(Err(ErrorUnauthorized("Invalid user"))),
            };

            let repo = DieselUserRepository::new(pool);

            match repo.get_by_id(uid) {
                Ok(Some(_)) => return ready(Ok(claims)),
                _ => return ready(Err(ErrorUnauthorized("Invalid user"))),
            }
        }
        ready(Err(ErrorUnauthorized("Unauthorized")))
    }
}
