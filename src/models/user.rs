use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::user::{NewUser as DomainNewUser, User as DomainUser};
use crate::models::hub::Hub;

#[derive(Debug, Clone, Identifiable, Associations, Queryable)]
#[diesel(belongs_to(Hub, foreign_key=hub_id))]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub hub_id: i32,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub name: Option<&'a str>,
    pub hub_id: i32,
    pub password_hash: &'a str,
}

impl From<User> for DomainUser {
    fn from(db: User) -> Self {
        Self {
            id: db.id,
            email: db.email,
            name: db.name,
            hub_id: db.hub_id,
            password_hash: db.password_hash,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl<'a> From<&'a DomainNewUser> for NewUser<'a> {
    fn from(nu: &'a DomainNewUser) -> Self {
        NewUser {
            email: &nu.email,
            name: nu.name.as_deref(),
            hub_id: nu.hub_id,
            password_hash: &nu.password_hash,
        }
    }
}
