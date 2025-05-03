use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewRole {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewUserRole {
    pub user_id: i32,
    pub role_id: i32,
}
