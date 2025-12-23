//! Services powering the main application views, such as loading index data and updating users.

use pushkind_common::services::errors::ServiceResult;
use std::convert::TryInto;

use crate::domain::types::{HubId, UserEmail, UserId};
use crate::dto::main::IndexData;
use crate::forms::main::{SaveUserForm, SaveUserPayload};
use crate::repository::{HubReader, MenuReader, RoleReader, UserListQuery, UserReader, UserWriter};
use pushkind_common::domain::auth::AuthenticatedUser;

/// Gathers all information necessary to render the main index view for a hub.
///
/// Returns an [`IndexData`] instance populated from the provided repository or
/// a [`ServiceError`] if any of the underlying lookups fail or the hub is not
/// found.
pub fn get_index_data(
    current_user: &AuthenticatedUser,
    repo: &(impl HubReader + UserReader + RoleReader + MenuReader),
) -> ServiceResult<IndexData> {
    let hub_id = HubId::new(current_user.hub_id)?;
    let email = UserEmail::new(&current_user.email)?;
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
    form: SaveUserForm,
    current_user: &AuthenticatedUser,
    repo: &impl UserWriter,
) -> ServiceResult<()> {
    let payload: SaveUserPayload = form.try_into()?;

    let user_id: i32 = current_user
        .sub
        .parse()
        .map_err(|_| pushkind_common::services::errors::ServiceError::Internal)?;
    let user_id = UserId::new(user_id)?;
    let hub_id = HubId::new(current_user.hub_id)?;
    let updates: crate::domain::user::UpdateUser = payload.into();
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
        let hub = Hub::new(HubId::new(5).unwrap(), HubName::new("h").unwrap(), now, now);
        let hub_clone = hub.clone();
        let hub_clone2 = hub.clone();
        let user = crate::domain::user::User::new(
            UserId::new(9).unwrap(),
            UserEmail::new("a@b").unwrap(),
            Some(crate::domain::types::UserName::new("N").unwrap()),
            HubId::new(5).unwrap(),
            "".into(),
            now,
            now,
            vec![],
        );
        let uwr = UserWithRoles::new(user, vec![]);
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
        let current_user = AuthenticatedUser {
            sub: "9".into(),
            email: "a@b".into(),
            hub_id: hub.id.get(),
            name: "N".into(),
            roles: vec![],
            exp: 0,
        };
        let data = get_index_data(&current_user, &repo).unwrap();
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
        let current_user = AuthenticatedUser {
            sub: uwr.user.id.get().to_string(),
            email: uwr.user.email.as_str().to_string(),
            hub_id: hub.id.get(),
            name: "N".into(),
            roles: vec![],
            exp: 0,
        };
        let res = update_current_user(form, &current_user, &repo);
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
        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "a@b".into(),
            hub_id: 1,
            name: "N".into(),
            roles: vec![],
            exp: 0,
        };
        let res = update_current_user(form, &current_user, &repo);
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
        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "a@b".into(),
            hub_id: 1,
            name: "N".into(),
            roles: vec![],
            exp: 0,
        };

        let res = update_current_user(form, &current_user, &repo);

        assert!(matches!(
            res,
            Err(pushkind_common::services::errors::ServiceError::Form(_))
        ));
    }
}
