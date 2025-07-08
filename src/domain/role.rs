use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Role assigned to a user describing a set of permissions.
pub struct Role {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize)]
/// Information required to create a new [`Role`].
pub struct NewRole<'a> {
    pub name: &'a str,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Mapping table between users and roles.
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// New entry in the user/role mapping table.
pub struct NewUserRole {
    pub user_id: i32,
    pub role_id: i32,
}
