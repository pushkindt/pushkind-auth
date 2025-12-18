//! DTOs exposed by the REST API.

use pushkind_common::domain::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ApiV1UsersQueryParams {
    pub role: Option<String>,
    pub query: Option<String>,
    pub page: Option<usize>,
}

/// DTO returned by API endpoints representing a user with roles and hub context.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserDto {
    pub sub: String,
    pub email: String,
    pub hub_id: i32,
    pub name: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

impl From<AuthenticatedUser> for UserDto {
    fn from(user: AuthenticatedUser) -> Self {
        Self {
            sub: user.sub,
            email: user.email,
            hub_id: user.hub_id,
            name: user.name,
            roles: user.roles,
            exp: user.exp,
        }
    }
}
