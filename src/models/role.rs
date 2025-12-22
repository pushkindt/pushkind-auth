//! Diesel models and conversions for roles and user-role mappings.

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::role::{NewUserRole as DomainNewUserRole, UserRole as DomainUserRole};
use crate::domain::types::TypeConstraintError;
use crate::domain::{role::NewRole as DomainNewRole, role::Role as DomainRole};
use crate::models::user::User;

#[derive(Debug, Clone, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::roles)]
/// Diesel model for [`crate::domain::role::Role`].
pub struct Role {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::roles)]
/// Insertable form of [`Role`].
pub struct NewRole<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone, Queryable, Associations, Identifiable)]
#[diesel(primary_key(user_id, role_id))]
#[diesel(belongs_to(User, foreign_key=user_id))]
#[diesel(belongs_to(Role, foreign_key=role_id))]
#[diesel(table_name = crate::schema::user_roles)]
/// Association table linking users to roles.
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_roles)]
/// Insertable variant of [`UserRole`].
pub struct NewUserRole {
    pub user_id: i32,
    pub role_id: i32,
}

impl TryFrom<Role> for DomainRole {
    type Error = TypeConstraintError;

    fn try_from(db: Role) -> Result<Self, Self::Error> {
        DomainRole::try_new(db.id, db.name, db.created_at, db.updated_at)
    }
}

impl<'a> From<&'a DomainNewRole> for NewRole<'a> {
    fn from(domain: &'a DomainNewRole) -> Self {
        Self {
            name: domain.name.as_str(),
        }
    }
}

impl TryFrom<UserRole> for DomainUserRole {
    type Error = TypeConstraintError;

    fn try_from(db: UserRole) -> Result<Self, Self::Error> {
        DomainUserRole::try_new(db.user_id, db.role_id)
    }
}

impl From<&DomainNewUserRole> for NewUserRole {
    fn from(domain: &DomainNewUserRole) -> Self {
        Self {
            user_id: domain.user_id.get(),
            role_id: domain.role_id.get(),
        }
    }
}
