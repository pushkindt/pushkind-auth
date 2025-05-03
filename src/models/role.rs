use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::role::{NewUserRole as DomainNewUserRole, UserRole as DomainUserRole};
use crate::domain::{role::NewRole as DomainNewRole, role::Role as DomainRole};
use crate::models::user::User;

#[derive(Debug, Clone, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::roles)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::roles)]
pub struct NewRole<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone, Queryable, Associations, Identifiable)]
#[diesel(primary_key(user_id, role_id))]
#[diesel(belongs_to(User, foreign_key=user_id))]
#[diesel(belongs_to(Role, foreign_key=role_id))]
#[diesel(table_name = crate::schema::user_roles)]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_roles)]
pub struct NewUserRole {
    pub user_id: i32,
    pub role_id: i32,
}

impl From<Role> for DomainRole {
    fn from(db: Role) -> Self {
        Self {
            id: db.id,
            name: db.name,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl<'a> From<&'a DomainNewRole> for NewRole<'a> {
    fn from(domain: &'a DomainNewRole) -> Self {
        Self { name: &domain.name }
    }
}

impl From<UserRole> for DomainUserRole {
    fn from(db: UserRole) -> Self {
        Self {
            user_id: db.user_id,
            role_id: db.role_id,
        }
    }
}

impl From<&DomainNewUserRole> for NewUserRole {
    fn from(domain: &DomainNewUserRole) -> Self {
        Self {
            user_id: domain.user_id,
            role_id: domain.role_id,
        }
    }
}
