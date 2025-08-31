use bcrypt::{DEFAULT_COST, hash};
use chrono::NaiveDateTime;
use diesel::prelude::*;

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
            roles: vec![],
        }
    }
}

impl<'a> TryFrom<&DomainNewUser<'a>> for NewUser {
    type Error = bcrypt::BcryptError;

    fn try_from(nu: &DomainNewUser<'a>) -> Result<Self, Self::Error> {
        let password_hash = hash(nu.password, DEFAULT_COST)?;

        Ok(NewUser {
            email: nu.email.clone(),
            name: nu.name.map(|n| n.to_string()),
            hub_id: nu.hub_id,
            password_hash,
        })
    }
}

impl<'a> TryFrom<DomainNewUser<'a>> for NewUser {
    type Error = bcrypt::BcryptError;

    fn try_from(nu: DomainNewUser<'a>) -> Result<Self, Self::Error> {
        Self::try_from(&nu)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::user::NewUser as DomainNewUser;
    use crate::models::user::NewUser;
    use bcrypt::verify;

    #[test]
    fn test_new_user_try_from() {
        let domain = DomainNewUser::new("John@Example.com", Some("John Doe"), 5, "super_secret");

        let db_user = NewUser::try_from(domain).expect("conversion failed");

        assert_eq!(db_user.email, "john@example.com");
        assert_eq!(db_user.name.as_deref(), Some("John Doe"));
        assert_eq!(db_user.hub_id, 5);
        assert!(verify("super_secret", &db_user.password_hash).unwrap());
    }
}
