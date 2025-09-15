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
    use crate::repository::mock::MockRepository;
    use chrono::Utc;
    use pushkind_common::repository::errors::RepositoryError;

    fn sample_repo() -> (MockRepository, UserWithRoles, Hub) {
        let mut repo = MockRepository::new();
        let now = Utc::now().naive_utc();
        let hub = Hub {
            id: 5,
            name: "h".into(),
            created_at: now,
            updated_at: now,
        };
        let hub_clone = hub.clone();
        let hub_clone2 = hub.clone();
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
        let uwr_clone = uwr.clone();
        let uwr_clone2 = uwr.clone();

        repo.expect_get_hub_by_id()
            .returning(move |_| Ok(Some(hub_clone.clone())));
        repo.expect_list_users()
            .returning(move |_| Ok((1, vec![uwr_clone.clone()])));
        repo.expect_list_roles().returning(|| Ok(vec![]));
        repo.expect_list_hubs()
            .returning(move || Ok(vec![hub_clone2.clone()]));
        repo.expect_list_menu().returning(|_| Ok(vec![]));
        repo.expect_get_user_by_email()
            .returning(move |_, _| Ok(Some(uwr_clone2.clone())));
        (repo, uwr, hub)
    }

    #[test]
    fn test_get_index_data() {
        let (repo, _uwr, hub) = sample_repo();
        let data = get_index_data(hub.id, "a@b", &repo).unwrap();
        assert_eq!(data.hub.id, hub.id);
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.user_name.as_deref(), Some("N"));
    }

    #[test]
    fn test_update_current_user_success() {
        let (mut repo, uwr, hub) = sample_repo();
        let user_clone = uwr.user.clone();
        repo.expect_update_user()
            .returning(move |_, _, _| Ok(user_clone.clone()));
        let updates = UpdateUser {
            name: "X".into(),
            password: None,
            roles: None,
        };
        let res = update_current_user(uwr.user.id, hub.id, &updates, &repo);
        assert!(res.is_ok());
    }

    #[test]
    fn test_update_current_user_failure() {
        let (mut repo, _uwr, _hub) = sample_repo();
        repo.expect_update_user()
            .returning(|_, _, _| Err(RepositoryError::NotFound));
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
}
