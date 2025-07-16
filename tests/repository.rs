use pushkind_auth::domain::hub::NewHub;
use pushkind_auth::domain::menu::NewMenu;
use pushkind_auth::domain::role::NewRole;
use pushkind_auth::domain::user::NewUser;
use pushkind_auth::domain::user::UpdateUser;
use pushkind_auth::repository::hub::DieselHubRepository;
use pushkind_auth::repository::menu::DieselMenuRepository;
use pushkind_auth::repository::role::DieselRoleRepository;
use pushkind_auth::repository::user::DieselUserRepository;
use pushkind_auth::repository::{HubReader, HubWriter};
use pushkind_auth::repository::{MenuReader, MenuWriter};
use pushkind_auth::repository::{RoleReader, RoleWriter};
use pushkind_auth::repository::{UserReader, UserWriter};

mod common;

#[test]
fn test_hub_repository_crud() {
    let test_db = common::TestDb::new("test_hub_repository_crud.db");
    let repo = DieselHubRepository::new(test_db.pool());

    // Create
    let new_hub = NewHub { name: "TestHub" };
    let hub = repo.create(&new_hub).unwrap();
    assert_eq!(hub.name, "TestHub");

    // Get by id
    let found = repo.get_by_id(hub.id).unwrap();
    assert!(found.is_some());

    // Get by name
    let found = repo.get_by_name("TestHub").unwrap();
    assert!(found.is_some());

    // List
    let hubs = repo.list().unwrap();
    assert_eq!(hubs.len(), 2); // 1 from setup + 1 from test

    // create a menu for the hub and ensure it's deleted along with the hub
    let menu_repo = DieselMenuRepository::new(test_db.pool());
    let new_menu = NewMenu {
        name: "TestMenu",
        url: "/test",
        hub_id: hub.id,
    };
    menu_repo.create(&new_menu).unwrap();
    assert_eq!(menu_repo.list(hub.id).unwrap().len(), 1);

    repo.delete(hub.id).unwrap();

    // menus should be removed when hub is deleted
    assert!(menu_repo.list(hub.id).unwrap().is_empty());
}

#[test]
fn test_user_repository_crud() {
    let test_db = common::TestDb::new("test_user_repository_crud.db");
    let hub_repo = DieselHubRepository::new(test_db.pool());

    // Create Hub
    let new_hub = NewHub { name: "TestHub" };
    let hub = hub_repo.create(&new_hub).unwrap();

    let role_repo = DieselRoleRepository::new(test_db.pool());

    let new_role = NewRole { name: "TestRole" };

    let role = role_repo.create(&new_role).unwrap();

    let user_repo = DieselUserRepository::new(test_db.pool());

    // Create User
    let new_user = NewUser {
        name: Some("TestUser"),
        hub_id: hub.id,
        email: "test@test.test",
        password: "test",
    };
    let user = user_repo.create(&new_user).unwrap();
    assert_eq!(user.name, Some("TestUser".to_string()));
    assert_eq!(user.email, "test@test.test");
    let created_at = user.created_at;
    let original_updated_at = user.updated_at;

    let inserted = user_repo.assign_roles(user.id, &[role.id]).unwrap();
    assert!(inserted == 1);

    // Get by email
    let found = user_repo.get_by_email("test@test.test", hub.id).unwrap();
    assert!(found.is_some());

    // List
    let users = user_repo.list(hub.id).unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].1.len(), 1);

    assert!(user_repo.verify_password("test", &user.password_hash));

    assert!(
        user_repo
            .login("test@test.test", "test", hub.id)
            .is_ok_and(|u| u.is_some())
    );

    let user_roles = user_repo.get_roles(user.id).unwrap();

    assert!(user_roles.iter().any(|r| r.name == role.name));

    let user = user_repo
        .update(
            user.id,
            &UpdateUser {
                name: "new name",
                password: Some("new password"),
            },
        )
        .unwrap();
    assert_eq!(user.name, Some("new name".to_string()));
    assert!(user_repo.verify_password("new password", &user.password_hash));
    assert!(user.updated_at > original_updated_at);
    assert_eq!(user.created_at, created_at);

    user_repo.assign_roles(user.id, &[]).unwrap();

    let roles = user_repo.get_roles(user.id).unwrap();
    assert!(roles.is_empty());

    user_repo.delete(user.id).unwrap();
}

#[test]
fn test_role_repository_crud() {
    let test_db = common::TestDb::new("test_role_repository_crud.db");
    let repo = DieselRoleRepository::new(test_db.pool());

    // Create
    let new_role = NewRole { name: "TestRole" };
    let role = repo.create(&new_role).unwrap();
    assert_eq!(role.name, "TestRole");

    // Get by id
    let found = repo.get_by_id(role.id).unwrap();
    assert!(found.is_some());

    // Get by name
    let found = repo.get_by_name("TestRole").unwrap();
    assert!(found.is_some());

    // List
    let roles = repo.list().unwrap();
    assert_eq!(roles.len(), 2); // admin and TestRole

    repo.delete(role.id).unwrap();
}

#[test]
fn test_email_lowercase_and_login_case_insensitive() {
    let test_db = common::TestDb::new("test_email_lowercase.db");
    let hub_repo = DieselHubRepository::new(test_db.pool());

    // Create hub
    let hub = hub_repo.create(&NewHub { name: "CaseHub" }).unwrap();

    let user_repo = DieselUserRepository::new(test_db.pool());

    // Register user with mixed case email
    let new_user = NewUser {
        name: Some("Case"),
        hub_id: hub.id,
        email: "Mixed@Example.COM",
        password: "pwd",
    };
    let user = user_repo.create(&new_user).unwrap();
    assert_eq!(user.email, "mixed@example.com");

    // Login should be case-insensitive
    let login = user_repo
        .login("MIXED@EXAMPLE.COM", "pwd", hub.id)
        .expect("login query failed");
    assert!(login.is_some());

    // Creating another user with same email (different case) should fail
    let dup_user = NewUser {
        name: Some("Dup"),
        hub_id: hub.id,
        email: "MIXED@example.com",
        password: "pwd",
    };
    let res = user_repo.create(&dup_user);
    assert!(res.is_err());
}

#[test]
fn test_assign_roles_atomic() {
    let test_db = common::TestDb::new("test_assign_roles_atomic.db");
    let hub_repo = DieselHubRepository::new(test_db.pool());
    let role_repo = DieselRoleRepository::new(test_db.pool());
    let user_repo = DieselUserRepository::new(test_db.pool());

    let hub = hub_repo.create(&NewHub { name: "AtomicHub" }).unwrap();
    let role = role_repo.create(&NewRole { name: "AtomicRole" }).unwrap();

    let user = user_repo
        .create(&NewUser {
            name: Some("Atomic"),
            hub_id: hub.id,
            email: "atomic@example.com",
            password: "pwd",
        })
        .unwrap();

    // Assign a valid role
    user_repo.assign_roles(user.id, &[role.id]).unwrap();

    // Attempt to assign a nonexistent role to trigger an error
    let res = user_repo.assign_roles(user.id, &[9999]);
    assert!(res.is_err());

    // Original role assignment should remain intact
    let roles = user_repo.get_roles(user.id).unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].id, role.id);
}
