//! Diesel models and conversions for users.

use bcrypt::{DEFAULT_COST, hash};
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::types::{HubId, TypeConstraintError, UserEmail, UserId, UserName};
use crate::domain::user::{NewUser as DomainNewUser, User as DomainUser};
use crate::models::hub::Hub;

#[derive(Debug, Clone, Identifiable, Associations, Queryable, QueryableByName)]
#[diesel(belongs_to(Hub, foreign_key=hub_id))]
#[diesel(table_name = crate::schema::users)]
#[diesel(foreign_derive)]
/// Diesel model for [`crate::domain::user::User`].
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub hub_id: i32,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(QueryableByName)]
pub struct UserCount {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
/// Insertable form of [`User`].
pub struct NewUser {
    pub email: String,
    pub name: Option<String>,
    pub hub_id: i32,
    pub password_hash: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::users)]
/// Data used when updating a [`User`] record.
pub struct UpdateUser<'a> {
    pub name: &'a str,
    pub password_hash: String,
    pub updated_at: NaiveDateTime,
}

impl TryFrom<User> for DomainUser {
    type Error = TypeConstraintError;

    fn try_from(db: User) -> Result<Self, Self::Error> {
        Ok(Self {
            id: UserId::try_from(db.id)?,
            email: UserEmail::try_from(db.email)?,
            name: db.name.map(UserName::try_from).transpose()?,
            hub_id: HubId::try_from(db.hub_id)?,
            password_hash: db.password_hash,
            created_at: db.created_at,
            updated_at: db.updated_at,
            roles: vec![],
        })
    }
}

impl<'a> TryFrom<&'a DomainNewUser> for NewUser {
    type Error = bcrypt::BcryptError;

    fn try_from(nu: &'a DomainNewUser) -> Result<Self, Self::Error> {
        let password_hash = hash(nu.password.clone(), DEFAULT_COST)?;

        Ok(NewUser {
            email: nu.email.as_str().to_string(),
            name: nu.name.clone().map(UserName::into_inner),
            hub_id: nu.hub_id.get(),
            password_hash,
        })
    }
}

impl TryFrom<DomainNewUser> for NewUser {
    type Error = bcrypt::BcryptError;

    fn try_from(nu: DomainNewUser) -> Result<Self, Self::Error> {
        let password_hash = hash(nu.password, DEFAULT_COST)?;

        Ok(NewUser {
            email: nu.email.into_inner(),
            name: nu.name.map(UserName::into_inner),
            hub_id: nu.hub_id.get(),
            password_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::types::{HubId, UserEmail, UserName};
    use crate::domain::user::NewUser as DomainNewUser;
    use crate::models::user::NewUser;
    use bcrypt::verify;

    #[test]
    fn test_new_user_try_from() {
        let domain = DomainNewUser::new(
            UserEmail::new("john@example.com").unwrap(),
            Some(UserName::new("John Doe").unwrap()),
            HubId::new(5).unwrap(),
            "super_secret".to_string(),
        );

        let db_user = NewUser::try_from(domain).expect("conversion failed");

        assert_eq!(db_user.email, "john@example.com");
        assert_eq!(db_user.name.as_deref(), Some("John Doe"));
        assert_eq!(db_user.hub_id, 5);
        assert!(verify("super_secret", &db_user.password_hash).unwrap());
    }
}
