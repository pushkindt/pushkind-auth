//! Administrative services for managing users, roles, menus, and hubs.

use crate::domain::hub::NewHub;
use crate::domain::role::Role;
use crate::domain::user::{UpdateUser, User};
use crate::repository::{
    HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserReader, UserWriter,
};
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::check_role;
use pushkind_common::services::errors::{ServiceError, ServiceResult};

/// Ensures the authenticated user has the `admin` role.
fn ensure_admin(user: &AuthenticatedUser) -> ServiceResult<()> {
    if !check_role("admin", &user.roles) {
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
        None => return Ok(()),
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
    use crate::domain::user::UpdateUser;
    use crate::repository::test::TestRepository;
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

    fn regular_user() -> AuthenticatedUser {
        AuthenticatedUser {
            sub: "2".into(),
            email: "u@e".into(),
            hub_id: 1,
            name: "User".into(),
            roles: vec!["user".into()],
            exp: 0,
        }
    }

    #[test]
    fn user_modal_data_success_and_not_found() {
        let now = TestRepository::now();
        let repo = TestRepository::with_users(vec![TestRepository::make_user(7, "u@e", 1, vec![])])
            .with_roles(vec![Role {
                id: 1,
                name: "admin".into(),
                created_at: now,
                updated_at: now,
            }]);
        let current_user = admin_user();
        let (user, roles) = user_modal_data(&current_user, 7, &repo).unwrap();
        assert!(user.is_some());
        assert_eq!(roles.len(), 1);

        let (missing, _) = user_modal_data(&current_user, 99, &repo).unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn user_modal_data_unauthorized() {
        let repo = TestRepository::new();
        let res = user_modal_data(&regular_user(), 1, &repo);
        assert!(matches!(res, Err(ServiceError::Unauthorized)));
    }

    #[test]
    fn create_role_authorization() {
        let repo = TestRepository::new();
        let new_role = NewRole { name: "new".into() };
        assert!(create_role(&admin_user(), &new_role, &repo).is_ok());
        assert!(matches!(
            create_role(&regular_user(), &new_role, &repo),
            Err(ServiceError::Unauthorized)
        ));
    }

    #[test]
    fn delete_user_by_id_paths() {
        let repo = TestRepository::with_users(vec![TestRepository::make_user(2, "u@e", 1, vec![])]);
        assert!(delete_user_by_id(&admin_user(), 2, &repo).is_ok());
        let mut admin_self = admin_user();
        admin_self.sub = "2".into();
        assert!(matches!(
            delete_user_by_id(&admin_self, 2, &repo),
            Err(ServiceError::Unauthorized)
        ));
        assert!(matches!(
            delete_user_by_id(&admin_user(), 99, &repo),
            Err(ServiceError::NotFound)
        ));
        assert!(matches!(
            delete_user_by_id(&regular_user(), 2, &repo),
            Err(ServiceError::Unauthorized)
        ));
    }

    #[test]
    fn assign_roles_and_update_user_cases() {
        let repo = TestRepository::with_users(vec![TestRepository::make_user(2, "u@e", 1, vec![])]);
        let updates = UpdateUser {
            name: "New".into(),
            password: None,
            roles: None,
        };
        assert!(assign_roles_and_update_user(&admin_user(), 2, &updates, &[1, 2], &repo).is_ok());
        assert!(assign_roles_and_update_user(&admin_user(), 99, &updates, &[], &repo).is_ok());
        assert!(matches!(
            assign_roles_and_update_user(&regular_user(), 2, &updates, &[], &repo),
            Err(ServiceError::Unauthorized)
        ));
    }

    #[test]
    fn create_and_delete_hub() {
        let now = TestRepository::now();
        let repo = TestRepository::new().with_hubs(vec![Hub {
            id: 2,
            name: "hub".into(),
            created_at: now,
            updated_at: now,
        }]);
        let new_hub = NewHub { name: "hub".into() };
        assert!(create_hub(&admin_user(), &new_hub, &repo).is_ok());
        assert!(matches!(
            create_hub(&regular_user(), &new_hub, &repo),
            Err(ServiceError::Unauthorized)
        ));
        assert!(delete_hub_by_id(&admin_user(), 2, &repo).is_ok());
        assert!(matches!(
            delete_hub_by_id(&admin_user(), 1, &repo),
            Err(ServiceError::Unauthorized)
        ));
        assert!(matches!(
            delete_hub_by_id(&admin_user(), 99, &repo),
            Err(ServiceError::NotFound)
        ));
        assert!(matches!(
            delete_hub_by_id(&regular_user(), 2, &repo),
            Err(ServiceError::Unauthorized)
        ));
    }

    #[test]
    fn delete_role_by_id_paths() {
        let now = TestRepository::now();
        let repo = TestRepository::new().with_roles(vec![Role {
            id: 2,
            name: "r".into(),
            created_at: now,
            updated_at: now,
        }]);
        assert!(delete_role_by_id(&admin_user(), 2, &repo).is_ok());
        assert!(matches!(
            delete_role_by_id(&admin_user(), 1, &repo),
            Err(ServiceError::Unauthorized)
        ));
        assert!(matches!(
            delete_role_by_id(&admin_user(), 99, &repo),
            Err(ServiceError::NotFound)
        ));
        assert!(matches!(
            delete_role_by_id(&regular_user(), 2, &repo),
            Err(ServiceError::Unauthorized)
        ));
    }

    #[test]
    fn create_and_delete_menu() {
        let repo = TestRepository::new();
        let new_menu = NewMenu {
            name: "m".into(),
            url: "/".into(),
            hub_id: 1,
        };
        assert!(create_menu(&admin_user(), &new_menu, &repo).is_ok());
        assert!(matches!(
            create_menu(&regular_user(), &new_menu, &repo),
            Err(ServiceError::Unauthorized)
        ));

        let menu = Menu {
            id: 1,
            name: "m".into(),
            url: "/".into(),
            hub_id: 1,
        };
        let repo_with_menu = TestRepository::new().with_menus(vec![menu]);
        assert!(delete_menu_by_id(&admin_user(), 1, &repo_with_menu).is_ok());
        assert!(matches!(
            delete_menu_by_id(&admin_user(), 99, &repo),
            Err(ServiceError::NotFound)
        ));
        assert!(matches!(
            delete_menu_by_id(&regular_user(), 1, &repo),
            Err(ServiceError::Unauthorized)
        ));
    }
}
