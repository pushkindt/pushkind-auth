//! Administrative services for managing users, roles, menus, and hubs.

use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::check_role;
use pushkind_common::services::errors::{ServiceError, ServiceResult};

use crate::SERVICE_ACCESS_ROLE;
use crate::domain::hub::NewHub;
use crate::domain::role::Role;
use crate::domain::user::{UpdateUser, User};
use crate::repository::{
    HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserReader, UserWriter,
};

/// Ensures the authenticated user has the `admin` role.
fn ensure_admin(user: &AuthenticatedUser) -> ServiceResult<()> {
    if !check_role(SERVICE_ACCESS_ROLE, &user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    Ok(())
}

/// Creates a new role when the current user is an admin.
pub fn create_role(
    current_user: &AuthenticatedUser,
    new_role: &crate::domain::role::NewRole,
    repo: &impl RoleWriter,
) -> ServiceResult<()> {
    ensure_admin(current_user)?;
    repo.create_role(new_role)?;
    Ok(())
}

/// Retrieves the user and available roles for the modal editor.
pub fn user_modal_data(
    current_user: &AuthenticatedUser,
    user_id: i32,
    repo: &(impl UserReader + RoleReader),
) -> ServiceResult<(Option<User>, Vec<Role>)> {
    ensure_admin(current_user)?;
    let user = repo
        .get_user_by_id(user_id, current_user.hub_id)?
        .map(|u| u.user);
    let roles = repo.list_roles()?;
    Ok((user, roles))
}

/// Deletes a user by ID, preventing self-deletion and non-admin access.
pub fn delete_user_by_id(
    current_user: &AuthenticatedUser,
    user_id: i32,
    repo: &(impl UserReader + UserWriter),
) -> ServiceResult<()> {
    ensure_admin(current_user)?;

    let current_user_id: i32 = current_user
        .sub
        .parse()
        .map_err(|_| ServiceError::Internal)?;

    if user_id == current_user_id {
        return Err(ServiceError::Unauthorized);
    }

    let user = match repo.get_user_by_id(user_id, current_user.hub_id)? {
        Some(u) => u.user,
        None => return Err(ServiceError::NotFound),
    };
    repo.delete_user(user.id)?;
    Ok(())
}

/// Assigns roles and updates a user if they belong to the current hub.
pub fn assign_roles_and_update_user(
    current_user: &AuthenticatedUser,
    user_id: i32,
    updates: &UpdateUser,
    role_ids: &[i32],
    repo: &(impl UserWriter + UserReader),
) -> ServiceResult<()> {
    ensure_admin(current_user)?;
    // Validate user exists in the hub
    let user = match repo.get_user_by_id(user_id, current_user.hub_id)? {
        Some(u) => u.user,
        None => return Err(ServiceError::NotFound),
    };
    repo.assign_roles_to_user(user_id, role_ids)?;
    repo.update_user(user.id, user.hub_id, updates)?;
    Ok(())
}

/// Creates a new hub when invoked by an admin.
pub fn create_hub(
    current_user: &AuthenticatedUser,
    new_hub: &NewHub,
    repo: &impl HubWriter,
) -> ServiceResult<()> {
    ensure_admin(current_user)?;
    repo.create_hub(new_hub)?;
    Ok(())
}

/// Deletes a role by ID, protecting the base admin role.
pub fn delete_role_by_id(
    current_user: &AuthenticatedUser,
    role_id: i32,
    repo: &impl RoleWriter,
) -> ServiceResult<()> {
    ensure_admin(current_user)?;
    if role_id == 1 {
        // Protect the base admin role from deletion.
        return Err(ServiceError::Unauthorized);
    }
    repo.delete_role(role_id)?;
    Ok(())
}

/// Deletes a hub by ID, preventing removal of the current user's hub.
pub fn delete_hub_by_id(
    current_user: &AuthenticatedUser,
    hub_id: i32,
    repo: &impl HubWriter,
) -> ServiceResult<()> {
    ensure_admin(current_user)?;
    if current_user.hub_id == hub_id {
        // Prevent deleting the hub currently associated with the user.
        return Err(ServiceError::Unauthorized);
    }
    repo.delete_hub(hub_id)?;
    Ok(())
}

/// Creates a new menu entry for the given hub.
pub fn create_menu(
    current_user: &AuthenticatedUser,
    new_menu: &crate::domain::menu::NewMenu,
    repo: &impl MenuWriter,
) -> ServiceResult<()> {
    ensure_admin(current_user)?;
    repo.create_menu(new_menu)?;
    Ok(())
}

/// Deletes a menu by ID if it exists for the current hub.
pub fn delete_menu_by_id(
    current_user: &AuthenticatedUser,
    menu_id: i32,
    repo: &(impl MenuReader + MenuWriter),
) -> ServiceResult<()> {
    ensure_admin(current_user)?;
    let menu = match repo.get_menu_by_id(menu_id, current_user.hub_id)? {
        Some(m) => m,
        None => return Err(ServiceError::NotFound),
    };
    repo.delete_menu(menu.id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hub::{Hub, NewHub};
    use crate::domain::menu::{Menu, NewMenu};
    use crate::domain::role::{NewRole, Role};
    use crate::domain::user::{User, UserWithRoles};
    use crate::repository::mock::MockRepository;
    use chrono::Utc;
    use pushkind_common::domain::auth::AuthenticatedUser;

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
                id,
                email: email.into(),
                name: Some("User".into()),
                hub_id,
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
            id: 1,
            name: "admin".into(),
            created_at: user.user.created_at,
            updated_at: user.user.updated_at,
        };
        repo.expect_get_user_by_id()
            .times(2)
            .returning(move |id, _| {
                if id == 7 {
                    Ok(Some(user.clone()))
                } else {
                    Ok(None)
                }
            });
        repo.expect_list_roles()
            .returning(move || Ok(vec![role.clone()]));
        let current_user = admin_user();
        let (found, roles) = user_modal_data(&current_user, 7, &repo).unwrap();
        assert!(found.is_some());
        assert_eq!(roles.len(), 1);
        let (missing, _) = user_modal_data(&current_user, 99, &repo).unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn create_role_authorization() {
        let mut repo = MockRepository::new();
        repo.expect_create_role().returning(|new_role| {
            let now = Utc::now().naive_utc();
            Ok(Role {
                id: 2,
                name: new_role.name.clone(),
                created_at: now,
                updated_at: now,
            })
        });
        let new_role = NewRole { name: "new".into() };
        assert!(create_role(&admin_user(), &new_role, &repo).is_ok());
    }

    #[test]
    fn create_and_delete_hub() {
        let mut repo = MockRepository::new();
        let now = Utc::now().naive_utc();
        repo.expect_create_hub().returning(move |nh| {
            Ok(Hub {
                id: 2,
                name: nh.name.clone(),
                created_at: now,
                updated_at: now,
            })
        });
        repo.expect_delete_hub().returning(|_| Ok(1));
        let new_hub = NewHub { name: "hub".into() };
        assert!(create_hub(&admin_user(), &new_hub, &repo).is_ok());
        assert!(delete_hub_by_id(&admin_user(), 2, &repo).is_ok());
    }

    #[test]
    fn create_and_delete_menu() {
        let mut repo = MockRepository::new();
        // The service first fetches the menu by id and hub before deleting.
        repo.expect_get_menu_by_id().returning(|id, hub_id| {
            Ok(Some(Menu {
                id,
                name: "m".into(),
                url: "/".into(),
                hub_id,
            }))
        });
        repo.expect_create_menu().returning(|nm| {
            Ok(Menu {
                id: 1,
                name: nm.name.clone(),
                url: nm.url.clone(),
                hub_id: nm.hub_id,
            })
        });
        repo.expect_delete_menu().returning(|_| Ok(1));
        let new_menu = NewMenu {
            name: "m".into(),
            url: "/".into(),
            hub_id: 1,
        };
        assert!(create_menu(&admin_user(), &new_menu, &repo).is_ok());
        assert!(delete_menu_by_id(&admin_user(), 1, &repo).is_ok());
    }
}
