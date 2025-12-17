//! Administrative services for managing users, roles, menus, and hubs.

use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::ensure_role;
use pushkind_common::services::errors::{ServiceError, ServiceResult};
use std::convert::TryInto;

use crate::SERVICE_ACCESS_ROLE;
use crate::domain::types::{HubId, MenuId, RoleId, UserId};
use crate::dto::admin::UserModalData;
use crate::forms::main::{AddHubForm, AddMenuForm, AddRoleForm, UpdateUserForm};
use crate::repository::{
    HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserReader, UserWriter,
};
use crate::services::validate_form;

/// Creates a new role when the current user is an admin.
pub fn create_role(
    current_user: &AuthenticatedUser,
    form: &AddRoleForm,
    repo: &impl RoleWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    validate_form(form)?;
    let new_role: crate::domain::role::NewRole = form.clone().try_into()?;
    repo.create_role(&new_role)?;
    Ok(())
}

/// Retrieves the user and available roles for the modal editor.
pub fn user_modal_data(
    current_user: &AuthenticatedUser,
    user_id: i32,
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
    current_user: &AuthenticatedUser,
    user_id: i32,
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
    current_user: &AuthenticatedUser,
    form: &UpdateUserForm,
    repo: &(impl UserWriter + UserReader),
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    validate_form(form)?;
    let user_id = UserId::new(form.id)?;
    let updates: crate::domain::user::UpdateUser = form.clone().try_into()?;
    let role_ids = updates.roles.clone().unwrap_or_default();
    // Validate user exists in the hub
    let hub_id = HubId::new(current_user.hub_id)?;
    let user = match repo.get_user_by_id(user_id, hub_id)? {
        Some(u) => u.user,
        None => return Err(ServiceError::NotFound),
    };
    repo.assign_roles_to_user(user_id, &role_ids)?;
    repo.update_user(user.id, user.hub_id, &updates)?;
    Ok(())
}

/// Creates a new hub when invoked by an admin.
pub fn create_hub(
    current_user: &AuthenticatedUser,
    form: &AddHubForm,
    repo: &impl HubWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    validate_form(form)?;
    let new_hub: crate::domain::hub::NewHub = form.clone().try_into()?;
    repo.create_hub(&new_hub)?;
    Ok(())
}

/// Deletes a role by ID, protecting the base admin role.
pub fn delete_role_by_id(
    current_user: &AuthenticatedUser,
    role_id: i32,
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
    current_user: &AuthenticatedUser,
    hub_id: i32,
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
    current_user: &AuthenticatedUser,
    form: &AddMenuForm,
    repo: &impl MenuWriter,
) -> ServiceResult<()> {
    ensure_role(current_user, SERVICE_ACCESS_ROLE)?;
    validate_form(form)?;
    let hub_id = HubId::new(current_user.hub_id)?;
    let new_menu = form.to_new_menu(hub_id)?;
    repo.create_menu(&new_menu)?;
    Ok(())
}

/// Deletes a menu by ID if it exists for the current hub.
pub fn delete_menu_by_id(
    current_user: &AuthenticatedUser,
    menu_id: i32,
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
        UserWithRoles {
            user: User {
                id: UserId::new(id).unwrap(),
                email: UserEmail::new(email).unwrap(),
                name: Some(crate::domain::types::UserName::new("User").unwrap()),
                hub_id: HubId::new(hub_id).unwrap(),
                password_hash: "hash".into(),
                created_at: now,
                updated_at: now,
                roles: vec![],
            },
            roles: vec![],
        }
    }

    #[test]
    fn user_modal_data_success_and_not_found() {
        let mut repo = MockRepository::new();
        let user = make_user(7, "u@e", 1);
        let role = Role {
            id: RoleId::new(1).unwrap(),
            name: RoleName::new("admin").unwrap(),
            created_at: user.user.created_at,
            updated_at: user.user.updated_at,
        };
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
        let found = user_modal_data(&current_user, 7, &repo).unwrap();
        assert!(found.user.is_some());
        assert_eq!(found.roles.len(), 1);
        let missing = user_modal_data(&current_user, 99, &repo).unwrap();
        assert!(missing.user.is_none());
    }

    #[test]
    fn create_role_authorization() {
        let mut repo = MockRepository::new();
        repo.expect_create_role().returning(|new_role| {
            let now = Utc::now().naive_utc();
            Ok(Role {
                id: RoleId::new(2).unwrap(),
                name: new_role.name.clone(),
                created_at: now,
                updated_at: now,
            })
        });
        let form = AddRoleForm { name: "new".into() };
        assert!(create_role(&admin_user(), &form, &repo).is_ok());
    }

    #[test]
    fn create_role_validation_error() {
        let repo = MockRepository::new();
        let form = AddRoleForm { name: "".into() };

        let res = create_role(&admin_user(), &form, &repo);

        assert!(matches!(res, Err(ServiceError::Form(_))));
    }

    #[test]
    fn create_and_delete_hub() {
        let mut repo = MockRepository::new();
        let now = Utc::now().naive_utc();
        repo.expect_create_hub().returning(move |nh| {
            Ok(Hub {
                id: HubId::new(2).unwrap(),
                name: nh.name.clone(),
                created_at: now,
                updated_at: now,
            })
        });
        repo.expect_delete_hub().returning(|_| Ok(1));
        let form = AddHubForm { name: "hub".into() };
        assert!(create_hub(&admin_user(), &form, &repo).is_ok());
        assert!(delete_hub_by_id(&admin_user(), 2, &repo).is_ok());
    }

    #[test]
    fn create_and_delete_menu() {
        let mut repo = MockRepository::new();
        // The service first fetches the menu by id and hub before deleting.
        repo.expect_get_menu_by_id().returning(|id, hub_id| {
            Ok(Some(Menu {
                id: MenuId::new(id.get()).unwrap(),
                name: crate::domain::types::MenuName::new("m").unwrap(),
                url: crate::domain::types::MenuUrl::new("/").unwrap(),
                hub_id,
            }))
        });
        repo.expect_create_menu().returning(|nm| {
            Ok(Menu {
                id: MenuId::new(1).unwrap(),
                name: nm.name.clone(),
                url: nm.url.clone(),
                hub_id: nm.hub_id,
            })
        });
        repo.expect_delete_menu().returning(|_| Ok(1));
        let form = AddMenuForm {
            name: "m".into(),
            url: "/".into(),
        };
        assert!(create_menu(&admin_user(), &form, &repo).is_ok());
        assert!(delete_menu_by_id(&admin_user(), 1, &repo).is_ok());
    }
}
