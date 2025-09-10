//! Services powering the main application views, such as loading index data and updating users.

use crate::domain::hub::Hub;
use crate::domain::menu::Menu;
use crate::domain::role::Role;
use crate::domain::user::{UpdateUser, UserWithRoles};
use crate::repository::{HubReader, MenuReader, RoleReader, UserListQuery, UserReader, UserWriter};
use pushkind_common::services::errors::ServiceResult;

/// Aggregated information required to render the index page.
///
/// The struct bundles data about the current hub, available users, roles,
/// hubs and menu entries, as well as the name of the current user if
/// available.
pub struct IndexData {
    pub hub: Hub,
    pub users: Vec<UserWithRoles>,
    pub roles: Vec<Role>,
    pub hubs: Vec<Hub>,
    pub menu: Vec<Menu>,
    pub user_name: Option<String>,
}

/// Gathers all information necessary to render the main index view for a hub.
///
/// Returns an [`IndexData`] instance populated from the provided repository or
/// a [`ServiceError`] if any of the underlying lookups fail or the hub is not
/// found.
pub fn get_index_data(
    hub_id: i32,
    user_email: &str,
    repo: &(impl HubReader + UserReader + RoleReader + MenuReader),
) -> ServiceResult<IndexData> {
    let hub = repo
        .get_hub_by_id(hub_id)?
        .ok_or(pushkind_common::services::errors::ServiceError::NotFound)?;
    let (_total, users) = repo.list_users(UserListQuery::new(hub_id))?;
    let roles = repo.list_roles()?;
    let hubs = repo.list_hubs()?;
    let menu = repo.list_menu(hub_id)?;
    let user_name = repo
        .get_user_by_email(user_email, hub_id)?
        .and_then(|u| u.user.name);
    Ok(IndexData {
        hub,
        users,
        roles,
        hubs,
        menu,
        user_name,
    })
}

/// Updates the currently authenticated user with the provided changes.
///
/// The update is delegated to the [`UserWriter`] implementation and any
/// repository errors are propagated to the caller.
pub fn update_current_user(
    user_id: i32,
    hub_id: i32,
    updates: &UpdateUser,
    repo: &impl UserWriter,
) -> ServiceResult<()> {
    repo.update_user(user_id, hub_id, updates)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::test::TestRepository;

    fn sample() -> TestRepository {
        let now = TestRepository::now();
        let hub = Hub {
            id: 5,
            name: "h".into(),
            created_at: now,
            updated_at: now,
        };
        let user = crate::domain::user::User {
            id: 9,
            email: "a@b".into(),
            name: Some("N".into()),
            hub_id: 5,
            password_hash: "".into(),
            created_at: now,
            updated_at: now,
            roles: vec![],
        };
        let uwr = UserWithRoles {
            user,
            roles: vec![],
        };
        TestRepository::with_users(vec![uwr])
            .with_hubs(vec![hub])
            .with_roles(vec![])
            .with_menus(vec![])
    }

    #[test]
    fn test_get_index_data() {
        let repo = sample();
        let data = get_index_data(5, "a@b", &repo).unwrap();
        assert_eq!(data.hub.id, 5);
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.user_name.as_deref(), Some("N"));
    }

    #[test]
    fn test_update_current_user_success() {
        let repo = sample();
        let updates = UpdateUser {
            name: "X".into(),
            password: None,
            roles: None,
        };
        let res = update_current_user(9, 5, &updates, &repo);
        assert!(res.is_ok());
    }

    use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

    struct MissingUserRepo;

    impl UserWriter for MissingUserRepo {
        fn create_user(
            &self,
            _new_user: &crate::domain::user::NewUser,
        ) -> RepositoryResult<crate::domain::user::User> {
            unimplemented!()
        }

        fn assign_roles_to_user(
            &self,
            _user_id: i32,
            _role_ids: &[i32],
        ) -> RepositoryResult<usize> {
            unimplemented!()
        }

        fn update_user(
            &self,
            _user_id: i32,
            _hub_id: i32,
            _updates: &UpdateUser,
        ) -> RepositoryResult<crate::domain::user::User> {
            Err(RepositoryError::NotFound)
        }

        fn delete_user(&self, _user_id: i32) -> RepositoryResult<usize> {
            unimplemented!()
        }
    }

    #[test]
    fn test_update_current_user_missing_user() {
        let repo = MissingUserRepo;
        let updates = UpdateUser {
            name: "X".into(),
            password: None,
            roles: None,
        };
        let res = update_current_user(1, 1, &updates, &repo);
        assert!(matches!(
            res,
            Err(pushkind_common::services::errors::ServiceError::NotFound)
        ));
    }

    struct FailingRepo;

    impl UserWriter for FailingRepo {
        fn create_user(
            &self,
            _new_user: &crate::domain::user::NewUser,
        ) -> RepositoryResult<crate::domain::user::User> {
            unimplemented!()
        }

        fn assign_roles_to_user(
            &self,
            _user_id: i32,
            _role_ids: &[i32],
        ) -> RepositoryResult<usize> {
            unimplemented!()
        }

        fn update_user(
            &self,
            _user_id: i32,
            _hub_id: i32,
            _updates: &UpdateUser,
        ) -> RepositoryResult<crate::domain::user::User> {
            Err(RepositoryError::ValidationError("fail".into()))
        }

        fn delete_user(&self, _user_id: i32) -> RepositoryResult<usize> {
            unimplemented!()
        }
    }

    #[test]
    fn test_update_current_user_failure() {
        let repo = FailingRepo;
        let updates = UpdateUser {
            name: "X".into(),
            password: None,
            roles: None,
        };
        let res = update_current_user(1, 1, &updates, &repo);
        assert!(matches!(
            res,
            Err(pushkind_common::services::errors::ServiceError::Repository(
                RepositoryError::ValidationError(_)
            ))
        ));
    }
}
