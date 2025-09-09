use crate::domain::hub::NewHub;
use crate::domain::role::Role;
use crate::domain::user::{UpdateUser, User};
use crate::repository::{
    HubWriter, MenuReader, MenuWriter, RoleReader, RoleWriter, UserReader, UserWriter,
};
use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::routes::check_role;
use pushkind_common::services::errors::{ServiceError, ServiceResult};

pub fn create_role(
    current_user: &AuthenticatedUser,
    new_role: &crate::domain::role::NewRole,
    repo: &impl RoleWriter,
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    repo.create_role(new_role)?;
    Ok(())
}

pub fn user_modal_data(
    current_user: &AuthenticatedUser,
    user_id: i32,
    repo: &(impl UserReader + RoleReader),
) -> ServiceResult<(Option<User>, Vec<Role>)> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    let user = repo
        .get_user_by_id(user_id, current_user.hub_id)?
        .map(|u| u.user);
    let roles = repo.list_roles()?;
    Ok((user, roles))
}

pub fn delete_user_by_id(
    current_user: &AuthenticatedUser,
    user_id: i32,
    repo: &(impl UserReader + UserWriter),
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }

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

pub fn assign_roles_and_update_user(
    current_user: &AuthenticatedUser,
    user_id: i32,
    updates: &UpdateUser,
    role_ids: &[i32],
    repo: &(impl UserWriter + UserReader),
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    // Validate user exists in the hub
    let user = match repo.get_user_by_id(user_id, current_user.hub_id)? {
        Some(u) => u.user,
        None => return Ok(()),
    };
    repo.assign_roles_to_user(user_id, role_ids)?;
    repo.update_user(user.id, user.hub_id, updates)?;
    Ok(())
}

pub fn create_hub(
    current_user: &AuthenticatedUser,
    new_hub: &NewHub,
    repo: &impl HubWriter,
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    repo.create_hub(new_hub)?;
    Ok(())
}

pub fn delete_role_by_id(
    current_user: &AuthenticatedUser,
    role_id: i32,
    repo: &impl RoleWriter,
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    if role_id == 1 {
        // Protect the base admin role from deletion.
        return Err(ServiceError::Unauthorized);
    }
    repo.delete_role(role_id)?;
    Ok(())
}

pub fn delete_hub_by_id(
    current_user: &AuthenticatedUser,
    hub_id: i32,
    repo: &impl HubWriter,
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    if current_user.hub_id == hub_id {
        // Prevent deleting the hub currently associated with the user.
        return Err(ServiceError::Unauthorized);
    }
    repo.delete_hub(hub_id)?;
    Ok(())
}

pub fn create_menu(
    current_user: &AuthenticatedUser,
    new_menu: &crate::domain::menu::NewMenu,
    repo: &impl MenuWriter,
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    repo.create_menu(new_menu)?;
    Ok(())
}

pub fn delete_menu_by_id(
    current_user: &AuthenticatedUser,
    menu_id: i32,
    repo: &(impl MenuReader + MenuWriter),
) -> ServiceResult<()> {
    if !check_role("admin", &current_user.roles) {
        return Err(ServiceError::Unauthorized);
    }
    let menu = match repo.get_menu_by_id(menu_id, current_user.hub_id)? {
        Some(m) => m,
        None => return Ok(()),
    };
    repo.delete_menu(menu.id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::role::Role;
    use crate::domain::user::{User, UserWithRoles};
    use crate::repository::test::TestRepository;

    #[test]
    fn test_user_modal_data() {
        use pushkind_common::domain::auth::AuthenticatedUser;
        let now = TestRepository::now();
        let user = User {
            id: 7,
            email: "u@e".into(),
            name: Some("U".into()),
            hub_id: 10,
            password_hash: "".into(),
            created_at: now,
            updated_at: now,
            roles: vec![],
        };
        let repo = TestRepository::with_users(vec![UserWithRoles {
            user,
            roles: vec![],
        }])
        .with_roles(vec![Role {
            id: 1,
            name: "admin".into(),
            created_at: now,
            updated_at: now,
        }])
        .with_menus(vec![]);
        let current_user = AuthenticatedUser {
            sub: "1".into(),
            email: "a@b".into(),
            hub_id: 10,
            name: "Admin".into(),
            roles: vec!["admin".into()],
            exp: 0,
        };
        let (user, roles) = user_modal_data(&current_user, 7, &repo).unwrap();
        assert!(user.is_some());
        assert_eq!(roles.len(), 1);
    }
}
