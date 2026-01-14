//! Domain user model and related wrappers for roles and creation/update input.

use chrono::NaiveDateTime;
use pushkind_common::domain::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};

use crate::domain::role::Role;
use crate::domain::types::{
    HubId, RoleId, TypeConstraintError, UserEmail, UserId, UserName, UserPassword,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Representation of a user in the system.
///
/// This struct mirrors the data stored in the database but is free of any
/// persistence related logic.
pub struct User {
    pub id: UserId,
    pub email: UserEmail,
    pub name: Option<UserName>,
    pub hub_id: HubId,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub roles: Vec<RoleId>,
}

impl User {
    /// Constructs a user from validated domain types.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: UserId,
        email: UserEmail,
        name: Option<UserName>,
        hub_id: HubId,
        password_hash: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        roles: Vec<RoleId>,
    ) -> Self {
        Self {
            id,
            email,
            name,
            hub_id,
            password_hash,
            created_at,
            updated_at,
            roles,
        }
    }

    /// Validates raw values before constructing a user.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: i32,
        email: impl Into<String>,
        name: Option<String>,
        hub_id: i32,
        password_hash: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        roles: Vec<i32>,
    ) -> Result<Self, TypeConstraintError> {
        let roles = roles
            .into_iter()
            .map(RoleId::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(
            UserId::try_from(id)?,
            UserEmail::try_from(email.into())?,
            name.map(UserName::try_from).transpose()?,
            HubId::try_from(hub_id)?,
            password_hash,
            created_at,
            updated_at,
            roles,
        ))
    }
}
#[derive(Clone, Serialize)]
/// Wrapper combining a [`User`] with the fully resolved [`Role`]s attached to
/// the account.
pub struct UserWithRoles {
    pub user: User,
    pub roles: Vec<Role>,
}

impl UserWithRoles {
    /// Constructs a user-with-roles bundle, syncing role IDs on the user.
    pub fn new(mut user: User, roles: Vec<Role>) -> Self {
        user.roles = roles.iter().map(|role| role.id).collect();
        Self { user, roles }
    }

    /// Builds a user-with-roles bundle without additional validation.
    pub fn try_new(user: User, roles: Vec<Role>) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(user, roles))
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Data required to create a new user.
pub struct NewUser {
    pub email: UserEmail,
    pub name: Option<UserName>,
    pub hub_id: HubId,
    pub password: UserPassword,
}

impl NewUser {
    /// Creates a new [`NewUser`] from already validated and normalized input.
    pub fn new(
        email: UserEmail,
        name: Option<UserName>,
        hub_id: HubId,
        password: UserPassword,
    ) -> Self {
        Self {
            email,
            name,
            hub_id,
            password,
        }
    }

    /// Validates raw values before constructing a new user payload.
    pub fn try_new(
        email: impl Into<String>,
        name: Option<String>,
        hub_id: i32,
        password: impl Into<String>,
    ) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            UserEmail::try_from(email.into())?,
            name.map(UserName::try_from).transpose()?,
            HubId::try_from(hub_id)?,
            UserPassword::try_from(password.into())?,
        ))
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Optional fields that can be updated for a user.
pub struct UpdateUser {
    pub name: UserName,
    pub password: Option<UserPassword>,
    pub roles: Option<Vec<RoleId>>,
}

impl UpdateUser {
    /// Constructs an update payload from validated domain types.
    pub fn new(name: UserName, password: Option<UserPassword>, roles: Option<Vec<RoleId>>) -> Self {
        Self {
            name,
            password,
            roles,
        }
    }

    /// Validates raw values before constructing an update payload.
    pub fn try_new(
        name: impl Into<String>,
        password: Option<String>,
        roles: Option<Vec<i32>>,
    ) -> Result<Self, TypeConstraintError> {
        let roles = roles
            .map(|roles| roles.into_iter().map(RoleId::try_from).collect())
            .transpose()?;
        Ok(Self::new(
            UserName::try_from(name.into())?,
            password.map(UserPassword::try_from).transpose()?,
            roles,
        ))
    }
}

impl From<User> for AuthenticatedUser {
    fn from(user: User) -> Self {
        let mut result = Self {
            sub: user.id.to_string(),
            email: user.email.as_str().to_string(),
            hub_id: user.hub_id.get(),
            name: user.name.map(|n| n.into_inner()).unwrap_or_default(),
            roles: vec![],
            exp: 0,
        };
        result.set_expiration(7);
        result
    }
}

impl From<UserWithRoles> for AuthenticatedUser {
    fn from(ur: UserWithRoles) -> Self {
        let mut result = Self {
            sub: ur.user.id.to_string(),
            email: ur.user.email.as_str().to_string(),
            hub_id: ur.user.hub_id.get(),
            name: ur.user.name.map(|n| n.into_inner()).unwrap_or_default(),
            roles: ur.roles.into_iter().map(|r| r.name.into_inner()).collect(),
            exp: 0,
        };
        result.set_expiration(7);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::RoleName;
    use chrono::NaiveDateTime;

    fn sample_timestamp() -> NaiveDateTime {
        chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0)
            .unwrap()
            .naive_utc()
    }

    #[test]
    fn user_try_new_maps_roles_and_optionals() {
        let ts = sample_timestamp();
        let user = User::try_new(
            1,
            "Test@Example.com",
            Some(" Alice ".to_string()),
            2,
            "hash".to_string(),
            ts,
            ts,
            vec![1, 2],
        )
        .unwrap();
        assert_eq!(user.email.as_str(), "test@example.com");
        assert_eq!(user.name.as_ref().unwrap().as_str(), "Alice");
        assert_eq!(user.roles.len(), 2);
    }

    #[test]
    fn user_try_new_rejects_invalid_roles() {
        let ts = sample_timestamp();
        assert_eq!(
            User::try_new(
                1,
                "test@example.com",
                None,
                1,
                "hash".to_string(),
                ts,
                ts,
                vec![0],
            )
            .unwrap_err(),
            TypeConstraintError::NonPositiveId
        );
    }

    #[test]
    fn new_user_try_new_validates_inputs() {
        let new_user = NewUser::try_new("test@example.com", None, 3, "pass").unwrap();
        assert_eq!(new_user.email.as_str(), "test@example.com");
        assert_eq!(new_user.hub_id.get(), 3);
        assert_eq!(new_user.password.as_str(), "pass");
    }

    #[test]
    fn update_user_try_new_accepts_optional_fields() {
        let update =
            UpdateUser::try_new("  Name ", Some("secret".to_string()), Some(vec![1, 2])).unwrap();
        assert_eq!(update.name.as_str(), "Name");
        assert_eq!(update.password.as_ref().unwrap().as_str(), "secret");
        assert_eq!(update.roles.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn user_with_roles_updates_user_role_ids() {
        let ts = sample_timestamp();
        let user = User::new(
            UserId::new(1).unwrap(),
            UserEmail::new("test@example.com").unwrap(),
            None,
            HubId::new(2).unwrap(),
            "hash".to_string(),
            ts,
            ts,
            vec![],
        );
        let roles = vec![
            Role::new(
                RoleId::new(10).unwrap(),
                RoleName::new("admin").unwrap(),
                ts,
                ts,
            ),
            Role::new(
                RoleId::new(20).unwrap(),
                RoleName::new("viewer").unwrap(),
                ts,
                ts,
            ),
        ];
        let bundle = UserWithRoles::new(user, roles);
        assert_eq!(bundle.user.roles.len(), 2);
        assert_eq!(bundle.user.roles[0].get(), 10);
    }

    #[test]
    fn authenticated_user_from_user_sets_defaults() {
        let ts = sample_timestamp();
        let user = User::new(
            UserId::new(5).unwrap(),
            UserEmail::new("test@example.com").unwrap(),
            None,
            HubId::new(7).unwrap(),
            "hash".to_string(),
            ts,
            ts,
            vec![],
        );
        let auth: AuthenticatedUser = user.into();
        assert_eq!(auth.sub, "5");
        assert_eq!(auth.email, "test@example.com");
        assert_eq!(auth.hub_id, 7);
        assert_eq!(auth.name, "");
        assert!(auth.exp > 0);
    }

    #[test]
    fn authenticated_user_from_user_with_roles_maps_names() {
        let ts = sample_timestamp();
        let user = User::new(
            UserId::new(1).unwrap(),
            UserEmail::new("test@example.com").unwrap(),
            Some(UserName::new("Alice").unwrap()),
            HubId::new(2).unwrap(),
            "hash".to_string(),
            ts,
            ts,
            vec![],
        );
        let roles = vec![Role::new(
            RoleId::new(1).unwrap(),
            RoleName::new("admin").unwrap(),
            ts,
            ts,
        )];
        let auth: AuthenticatedUser = UserWithRoles::new(user, roles).into();
        assert_eq!(auth.roles, vec!["admin".to_string()]);
        assert_eq!(auth.name, "Alice");
        assert!(auth.exp > 0);
    }
}
