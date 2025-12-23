//! Administrative services for managing users, roles, menus, and hubs.

use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::ensure_role;
use pushkind_common::services::errors::{ServiceError, ServiceResult};
use std::convert::TryInto;

use crate::SERVICE_ACCESS_ROLE;
use crate::domain::types::{HubId, MenuId, RoleId, UserId};
use crate::dto::admin::UserModalData;
use crate::forms::main::{
    AddHubForm, AddHubPayload, AddMenuForm, AddMenuPayload, AddRoleForm, AddRolePayload,
    UpdateUserForm, UpdateUserPayload,
};
use crate::repository::{
    HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserReader, UserWriter,
};

/// Creates a new role when the current user is an admin.
pub fn create_role(
    form: AddRoleForm,
    current_user: &AuthenticatedUser,
    repo: &impl RoleWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    let payload: AddRolePayload = form.try_into()?;
    let new_role = payload.into();
    repo.create_role(&new_role)?;
    Ok(())
}

/// Retrieves the user and available roles for the modal editor.
pub fn user_modal_data(
    user_id: i32,
    current_user: &AuthenticatedUser,
    repo: &(impl UserReader + RoleReader),
) -> ServiceResult<UserModalData> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    let user_id = UserId::new(user_id)?;
    let hub_id = HubId::new(current_user.hub_id)?;
    let user = repo.get_user_by_id(user_id, hub_id)?.map(|u| u.user);
    let roles = repo.list_roles()?;
    Ok(UserModalData { user, roles })
}

/// Deletes a user by ID, preventing self-deletion and non-admin access.
pub fn delete_user_by_id(
    user_id: i32,
    current_user: &AuthenticatedUser,
    repo: &(impl UserReader + UserWriter),
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;

    let current_user_id: i32 = current_user
        .sub
        .parse()
        .map_err(|_| ServiceError::Internal)?;

    if user_id == current_user_id {
        return Err(ServiceError::Unauthorized);
    }

    let user_id = UserId::new(user_id)?;
    let hub_id = HubId::new(current_user.hub_id)?;
    let user = match repo.get_user_by_id(user_id, hub_id)? {
        Some(u) => u.user,
        None => return Err(ServiceError::NotFound),
    };
    repo.delete_user(user.id)?;
    Ok(())
}

/// Assigns roles and updates a user if they belong to the current hub.
pub fn assign_roles_and_update_user(
    user_id: i32,
    form: UpdateUserForm,
    current_user: &AuthenticatedUser,
    repo: &(impl UserWriter + UserReader),
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    let payload: UpdateUserPayload = form.try_into()?;

    let user_id = UserId::new(user_id)?;
    let updates = payload.into();

    // Validate user exists in the hub
    let hub_id = HubId::new(current_user.hub_id)?;
    let user = match repo.get_user_by_id(user_id, hub_id)? {
        Some(u) => u.user,
        None => return Err(ServiceError::NotFound),
    };

    repo.update_user(user.id, user.hub_id, &updates)?;
    Ok(())
}

/// Creates a new hub when invoked by an admin.
pub fn create_hub(
    form: AddHubForm,
    current_user: &AuthenticatedUser,
    repo: &impl HubWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    let payload: AddHubPayload = form.try_into()?;
    let new_hub = payload.into();
    repo.create_hub(&new_hub)?;
    Ok(())
}

/// Deletes a role by ID, protecting the base admin role.
pub fn delete_role_by_id(
    role_id: i32,
    current_user: &AuthenticatedUser,
    repo: &impl RoleWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    if role_id == 1 {
        // Protect the base admin role from deletion.
        return Err(ServiceError::Unauthorized);
    }
    let role_id = RoleId::new(role_id)?;
    repo.delete_role(role_id)?;
    Ok(())
}

/// Deletes a hub by ID, preventing removal of the current user's hub.
pub fn delete_hub_by_id(
    hub_id: i32,
    current_user: &AuthenticatedUser,
    repo: &impl HubWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    if current_user.hub_id == hub_id {
        // Prevent deleting the hub currently associated with the user.
        return Err(ServiceError::Unauthorized);
    }
    let hub_id = HubId::new(hub_id)?;
    repo.delete_hub(hub_id)?;
    Ok(())
}

/// Creates a new menu entry for the given hub.
pub fn create_menu(
    form: AddMenuForm,
    current_user: &AuthenticatedUser,
    repo: &impl MenuWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    let payload: AddMenuPayload = form.try_into()?;
    let hub_id = HubId::new(current_user.hub_id)?;
    let new_menu = payload.into_new_menu(hub_id);
    repo.create_menu(&new_menu)?;
    Ok(())
}

/// Deletes a menu by ID if it exists for the current hub.
pub fn delete_menu_by_id(
    menu_id: i32,
    current_user: &AuthenticatedUser,
    repo: &(impl MenuReader + MenuWriter),
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    let menu_id = MenuId::new(menu_id)?;
    let hub_id = HubId::new(current_user.hub_id)?;
    let menu = match repo.get_menu_by_id(menu_id, hub_id)? {
        Some(m) => m,
        None => return Err(ServiceError::NotFound),
    };
    repo.delete_menu(menu.id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hub::Hub;
    use crate::domain::menu::Menu;
    use crate::domain::role::Role;
    use crate::domain::types::{HubId, MenuId, RoleId, RoleName, UserEmail, UserId};
    use crate::domain::user::{User, UserWithRoles};
    use crate::forms::main::{AddHubForm, AddMenuForm, AddRoleForm};
    use crate::repository::mock::MockRepository;
    use chrono::Utc;
    use pushkind_common::domain::auth::AuthenticatedUser;
    use pushkind_common::services::errors::ServiceError;

    fn admin_user() -> AuthenticatedUser {
        AuthenticatedUser {
            sub: "1".into(),
            email: "a@b".into(),
            hub_id: 1,
            name: "Admin".into(),
            roles: vec!["admin".into()],
            exp: 0,
        }
    }

    fn make_user(id: i32, email: &str, hub_id: i32) -> UserWithRoles {
        let now = Utc::now().naive_utc();
        let user = User::new(
            UserId::new(id).unwrap(),
            UserEmail::new(email).unwrap(),
            Some(crate::domain::types::UserName::new("User").unwrap()),
            HubId::new(hub_id).unwrap(),
            "hash".into(),
            now,
            now,
            vec![],
        );
        UserWithRoles::new(user, vec![])
    }

    #[test]
    fn user_modal_data_success_and_not_found() {
        let mut repo = MockRepository::new();
        let user = make_user(7, "u@e", 1);
        let role = Role::new(
            RoleId::new(1).unwrap(),
            RoleName::new("admin").unwrap(),
            user.user.created_at,
            user.user.updated_at,
        );
        repo.expect_get_user_by_id()
            .times(2)
            .returning(move |id, _| {
                if id == UserId::new(7).unwrap() {
                    Ok(Some(user.clone()))
                } else {
                    Ok(None)
                }
            });
        repo.expect_list_roles()
            .returning(move || Ok(vec![role.clone()]));
        let current_user = admin_user();
        let found = user_modal_data(7, &current_user, &repo).unwrap();
        assert!(found.user.is_some());
        assert_eq!(found.roles.len(), 1);
        let missing = user_modal_data(99, &current_user, &repo).unwrap();
        assert!(missing.user.is_none());
    }

    #[test]
    fn create_role_authorization() {
        let mut repo = MockRepository::new();
        repo.expect_create_role().returning(|new_role| {
            let now = Utc::now().naive_utc();
            Ok(Role::new(
                RoleId::new(2).unwrap(),
                new_role.name.clone(),
                now,
                now,
            ))
        });
        let form = AddRoleForm { name: "new".into() };
        assert!(create_role(form, &admin_user(), &repo).is_ok());
    }

    #[test]
    fn create_role_validation_error() {
        let repo = MockRepository::new();
        let form = AddRoleForm { name: "".into() };

        let res = create_role(form, &admin_user(), &repo);

        assert!(matches!(res, Err(ServiceError::Form(_))));
    }

    #[test]
    fn create_and_delete_hub() {
        let mut repo = MockRepository::new();
        let now = Utc::now().naive_utc();
        repo.expect_create_hub()
            .returning(move |nh| Ok(Hub::new(HubId::new(2).unwrap(), nh.name.clone(), now, now)));
        repo.expect_delete_hub().returning(|_| Ok(1));
        let form = AddHubForm { name: "hub".into() };
        assert!(create_hub(form, &admin_user(), &repo).is_ok());
        assert!(delete_hub_by_id(2, &admin_user(), &repo).is_ok());
    }

    #[test]
    fn create_and_delete_menu() {
        let mut repo = MockRepository::new();
        // The service first fetches the menu by id and hub before deleting.
        repo.expect_get_menu_by_id().returning(|id, hub_id| {
            Ok(Some(Menu::new(
                MenuId::new(id.get()).unwrap(),
                crate::domain::types::MenuName::new("m").unwrap(),
                crate::domain::types::MenuUrl::new("https://app.test.me/").unwrap(),
                hub_id,
            )))
        });
        repo.expect_create_menu().returning(|nm| {
            Ok(Menu::new(
                MenuId::new(1).unwrap(),
                nm.name.clone(),
                nm.url.clone(),
                nm.hub_id,
            ))
        });
        repo.expect_delete_menu().returning(|_| Ok(1));
        let form = AddMenuForm {
            name: "m".into(),
            url: "https://app.test.me/".into(),
        };
        assert!(create_menu(form, &admin_user(), &repo).is_ok());
        assert!(delete_menu_by_id(1, &admin_user(), &repo).is_ok());
    }
}
