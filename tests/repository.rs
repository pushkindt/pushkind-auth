use pushkind_auth::domain::hub::NewHub;
use pushkind_auth::domain::menu::NewMenu;
use pushkind_auth::domain::role::NewRole;
use pushkind_auth::domain::user::NewUser;
use pushkind_auth::domain::user::UpdateUser;
use pushkind_auth::repository::DieselRepository;
use pushkind_auth::repository::UserListQuery;
use pushkind_auth::repository::{HubReader, HubWriter};
use pushkind_auth::repository::{MenuReader, MenuWriter};
use pushkind_auth::repository::{RoleReader, RoleWriter};
use pushkind_auth::repository::{UserReader, UserWriter};

mod common;

#[test]
fn test_hub_repository_crud() {
    let test_db = common::TestDb::new("test_hub_repository_crud.db");
    let repo = DieselRepository::new(test_db.pool());

    // Create
    let new_hub = NewHub {
        name: "TestHub".to_string(),
    };
    let hub = repo.create_hub(&new_hub).unwrap();
    assert_eq!(hub.name, "TestHub");

    // Get by id
    let found = repo.get_hub_by_id(hub.id).unwrap();
    assert!(found.is_some());

    // Get by name
    let found = repo.get_hub_by_name("TestHub").unwrap();
    assert!(found.is_some());

    // List
    let hubs = repo.list_hubs().unwrap();
    assert_eq!(hubs.len(), 2); // 1 from setup + 1 from test

    // create a menu for the hub and ensure it's deleted along with the hub
    let menu_repo = DieselRepository::new(test_db.pool());
    let new_menu = NewMenu {
        name: "TestMenu".to_string(),
        url: "/test".to_string(),
        hub_id: hub.id,
    };
    menu_repo.create_menu(&new_menu).unwrap();
    assert_eq!(menu_repo.list_menu(hub.id).unwrap().len(), 1);

    repo.delete_hub(hub.id).unwrap();

    // menus should be removed when hub is deleted
    assert!(menu_repo.list_menu(hub.id).unwrap().is_empty());
}

#[test]
fn test_user_repository_crud() {
    let test_db = common::TestDb::new("test_user_repository_crud.db");
    let hub_repo = DieselRepository::new(test_db.pool());

    // Create Hub
    let new_hub = NewHub {
        name: "TestHub".to_string(),
    };
    let hub = hub_repo.create_hub(&new_hub).unwrap();

    let role_repo = DieselRepository::new(test_db.pool());

    let new_role = NewRole {
        name: "TestRole".to_string(),
    };

    let role = role_repo.create_role(&new_role).unwrap();

    let user_repo = DieselRepository::new(test_db.pool());

    // Create User
    let new_user = NewUser::new(
        "test@test.test".to_string(),
        Some("TestUser".to_string()),
        hub.id,
        "test".to_string(),
    );
    let user = user_repo.create_user(&new_user).unwrap();
    assert_eq!(user.name, Some("TestUser".to_string()));
    assert_eq!(user.email, "test@test.test");
    let created_at = user.created_at;
    let original_updated_at = user.updated_at;

    let inserted = user_repo.assign_roles_to_user(user.id, &[role.id]).unwrap();
    assert!(inserted == 1);

    // Get by email
    let found = user_repo
        .get_user_by_email("test@test.test", hub.id)
        .unwrap();
    assert!(found.is_some());

    // List
    let (_total, users) = user_repo.list_users(UserListQuery::new(hub.id)).unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].roles.len(), 1);

    assert!(user_repo.verify_password("test", &user.password_hash));

    assert!(
        user_repo
            .login("test@test.test", "test", hub.id)
            .is_ok_and(|u| u.is_some())
    );

    let user_roles = user_repo.get_roles(user.id).unwrap();

    assert!(user_roles.iter().any(|r| r.name == role.name));

    let user = user_repo
        .update_user(
            user.id,
            hub.id,
            &UpdateUser {
                name: "new name".to_string(),
                password: Some("new password".to_string()),
                roles: None,
            },
        )
        .unwrap();
    assert_eq!(user.name, Some("new name".to_string()));
    assert!(user_repo.verify_password("new password", &user.password_hash));
    assert!(user.updated_at > original_updated_at);
    assert_eq!(user.created_at, created_at);

    user_repo.assign_roles_to_user(user.id, &[]).unwrap();

    let roles = user_repo.get_roles(user.id).unwrap();
    assert!(roles.is_empty());

    user_repo.delete_user(user.id).unwrap();
}

#[test]
fn test_role_repository_crud() {
    let test_db = common::TestDb::new("test_role_repository_crud.db");
    let repo = DieselRepository::new(test_db.pool());

    // Create
    let new_role = NewRole {
        name: "TestRole".to_string(),
    };
    let role = repo.create_role(&new_role).unwrap();
    assert_eq!(role.name, "TestRole");

    // Get by id
    let found = repo.get_role_by_id(role.id).unwrap();
    assert!(found.is_some());

    // Get by name
    let found = repo.get_role_by_name("TestRole").unwrap();
    assert!(found.is_some());

    // List
    let roles = repo.list_roles().unwrap();
    assert_eq!(roles.len(), 2); // admin and TestRole

    repo.delete_role(role.id).unwrap();
}

#[test]
fn test_email_lowercase_and_login_case_insensitive() {
    let test_db = common::TestDb::new("test_email_lowercase.db");
    let hub_repo = DieselRepository::new(test_db.pool());

    // Create hub
    let hub = hub_repo
        .create_hub(&NewHub {
            name: "CaseHub".to_string(),
        })
        .unwrap();

    let user_repo = DieselRepository::new(test_db.pool());

    // Register user with mixed case email
    let new_user = NewUser::new(
        "Mixed@Example.COM".to_string(),
        Some("Case".to_string()),
        hub.id,
        "pwd".to_string(),
    );
    let user = user_repo.create_user(&new_user).unwrap();
    assert_eq!(user.email, "mixed@example.com");

    // Login should be case-insensitive
    let login = user_repo
        .login("MIXED@EXAMPLE.COM", "pwd", hub.id)
        .expect("login query failed");
    assert!(login.is_some());

    // Creating another user with same email (different case) should fail
    let dup_user = NewUser::new(
        "MIXED@example.com".to_string(),
        Some("Dup".to_string()),
        hub.id,
        "pwd".to_string(),
    );
    let res = user_repo.create_user(&dup_user);
    assert!(res.is_err());
}

#[test]
fn test_assign_roles_atomic() {
    let test_db = common::TestDb::new("test_assign_roles_atomic.db");
    let hub_repo = DieselRepository::new(test_db.pool());
    let role_repo = DieselRepository::new(test_db.pool());
    let user_repo = DieselRepository::new(test_db.pool());

    let hub = hub_repo
        .create_hub(&NewHub {
            name: "AtomicHub".to_string(),
        })
        .unwrap();
    let role = role_repo
        .create_role(&NewRole {
            name: "AtomicRole".to_string(),
        })
        .unwrap();

    let user = user_repo
        .create_user(&NewUser::new(
            "atomic@example.com".to_string(),
            Some("Atomic".to_string()),
            hub.id,
            "pwd".to_string(),
        ))
        .unwrap();

    // Assign a valid role
    user_repo.assign_roles_to_user(user.id, &[role.id]).unwrap();

    // Attempt to assign a nonexistent role to trigger an error
    let res = user_repo.assign_roles_to_user(user.id, &[9999]);
    assert!(res.is_err());

    // Original role assignment should remain intact
    let roles = user_repo.get_roles(user.id).unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].id, role.id);
}
