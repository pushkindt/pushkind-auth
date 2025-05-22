use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub hub_id: i32,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub name: Option<String>,
    pub hub_id: i32,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub password: Option<String>,
}
