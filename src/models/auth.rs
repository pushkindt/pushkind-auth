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
use crate::repository::UserReader;
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

    pub fn from_user_roles(user: &User, roles: &[Role]) -> Self {
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

impl From<User> for AuthenticatedUser {
    fn from(user: User) -> Self {
        let mut result = Self {
            sub: user.id.to_string(),
            email: user.email,
            hub_id: user.hub_id,
            name: user.name.unwrap_or_default(),
            roles: vec![],
            exp: 0,
        };
        result.set_expiration(7);
        result
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

#[cfg(test)]
mod tests {
    use crate::domain::role::Role;
    use crate::domain::user::User;
    use crate::models::auth::AuthenticatedUser;
    use chrono::Utc;
    use jsonwebtoken::{DecodingKey, Validation, decode};

    #[test]
    fn test_from_user_sets_fields() {
        let now = Utc::now().naive_utc();
        let user = User {
            id: 1,
            email: "user@example.com".to_string(),
            name: Some("Example".to_string()),
            hub_id: 99,
            password_hash: "hash".to_string(),
            created_at: now,
            updated_at: now,
        };

        let role1 = Role {
            id: 1,
            name: "admin".to_string(),
            created_at: now,
            updated_at: now,
        };
        let role2 = Role {
            id: 2,
            name: "editor".to_string(),
            created_at: now,
            updated_at: now,
        };
        let roles = vec![role1.clone(), role2.clone()];

        let auth = AuthenticatedUser::from_user_roles(&user, &roles);

        assert_eq!(auth.sub, user.id.to_string());
        assert_eq!(auth.email, user.email);
        assert_eq!(auth.hub_id, user.hub_id);
        assert_eq!(auth.name, user.name.unwrap());
        assert_eq!(auth.roles, vec![role1.name, role2.name]);

        let now_ts = Utc::now().timestamp() as usize;
        let diff = if auth.exp > now_ts {
            auth.exp - now_ts
        } else {
            now_ts - auth.exp
        };
        let seven_days = 7 * 24 * 60 * 60;
        assert!(diff >= seven_days - 5 && diff <= seven_days + 5);
    }

    #[test]
    fn test_to_jwt_round_trip() {
        let now = Utc::now().naive_utc();
        let user = User {
            id: 42,
            email: "jwt@example.com".to_string(),
            name: None,
            hub_id: 7,
            password_hash: "hash".to_string(),
            created_at: now,
            updated_at: now,
        };
        let role = Role {
            id: 3,
            name: "viewer".to_string(),
            created_at: now,
            updated_at: now,
        };
        let mut auth = AuthenticatedUser::from_user_roles(&user, &[role.clone()]);
        let secret = "mysecret";

        let token = auth.to_jwt(secret).expect("failed to encode token");

        let decoded = decode::<AuthenticatedUser>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .expect("decode failed")
        .claims;

        assert_eq!(decoded.sub, auth.sub);
        assert_eq!(decoded.email, auth.email);
        assert_eq!(decoded.hub_id, auth.hub_id);
        assert_eq!(decoded.name, auth.name);
        assert_eq!(decoded.roles, vec![role.name]);
        assert_eq!(decoded.exp, auth.exp);
    }
}
