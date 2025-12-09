//! Services powering the main application views, such as loading index data and updating users.

use pushkind_common::services::errors::ServiceResult;
use std::convert::TryInto;

use crate::domain::types::{HubId, UserEmail, UserId};
use crate::dto::main::IndexData;
use crate::forms::main::SaveUserForm;
use crate::repository::{HubReader, MenuReader, RoleReader, UserListQuery, UserReader, UserWriter};
use crate::services::validate_form;

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
    let hub_id = HubId::new(hub_id)?;
    let email = UserEmail::new(user_email)?;
    let hub = repo
        .get_hub_by_id(hub_id)?
        .ok_or(pushkind_common::services::errors::ServiceError::NotFound)?;
    let (_total, users) = repo.list_users(UserListQuery::new(hub_id))?;
    let roles = repo.list_roles()?;
    let hubs = repo.list_hubs()?;
    let menu = repo.list_menu(hub_id)?;
    let user_name = repo
        .get_user_by_email(&email, hub_id)?
        .and_then(|u| u.user.name.map(|n| n.into_inner()));
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
    form: &SaveUserForm,
    repo: &impl UserWriter,
) -> ServiceResult<()> {
    validate_form(form)?;
    let user_id = UserId::new(user_id)?;
    let hub_id = HubId::new(hub_id)?;
    let updates: crate::domain::user::UpdateUser = form.clone().try_into()?;
    repo.update_user(user_id, hub_id, &updates)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hub::Hub;
    use crate::domain::types::{HubId, HubName, UserEmail, UserId};
    use crate::domain::user::UserWithRoles;
    use crate::forms::main::SaveUserForm;
    use crate::repository::mock::MockRepository;
    use chrono::Utc;
    use pushkind_common::repository::errors::RepositoryError;

    fn sample_repo() -> (MockRepository, UserWithRoles, Hub) {
        let mut repo = MockRepository::new();
        let now = Utc::now().naive_utc();
        let hub = Hub {
            id: HubId::new(5).unwrap(),
            name: HubName::new("h").unwrap(),
            created_at: now,
            updated_at: now,
        };
        let hub_clone = hub.clone();
        let hub_clone2 = hub.clone();
        let user = crate::domain::user::User {
            id: UserId::new(9).unwrap(),
            email: UserEmail::new("a@b").unwrap(),
            name: Some(crate::domain::types::UserName::new("N").unwrap()),
            hub_id: HubId::new(5).unwrap(),
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
        let data = get_index_data(hub.id.get(), "a@b", &repo).unwrap();
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
        let form = SaveUserForm {
            name: "X".into(),
            password: None,
        };
        let res = update_current_user(uwr.user.id.get(), hub.id.get(), &form, &repo);
        assert!(res.is_ok());
    }

    #[test]
    fn test_update_current_user_failure() {
        let (mut repo, _uwr, _hub) = sample_repo();
        repo.expect_update_user()
            .returning(|_, _, _| Err(RepositoryError::NotFound));
        let form = SaveUserForm {
            name: "X".into(),
            password: None,
        };
        let res = update_current_user(1, 1, &form, &repo);
        assert!(matches!(
            res,
            Err(pushkind_common::services::errors::ServiceError::NotFound)
        ));
    }

    #[test]
    fn test_update_current_user_validation_error() {
        let repo = MockRepository::new();
        let form = SaveUserForm {
            name: "".into(),
            password: None,
        };

        let res = update_current_user(1, 1, &form, &repo);

        assert!(matches!(
            res,
            Err(pushkind_common::services::errors::ServiceError::Form(_))
        ));
    }
}
