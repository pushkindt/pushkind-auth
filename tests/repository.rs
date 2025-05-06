use pushkind_auth::domain::hub::NewHub;
use pushkind_auth::domain::role::NewRole;
use pushkind_auth::domain::user::NewUser;
use pushkind_auth::repository::HubRepository;
use pushkind_auth::repository::RoleRepository;
use pushkind_auth::repository::UserRepository;
use pushkind_auth::repository::hub::DieselHubRepository;
use pushkind_auth::repository::role::DieselRoleRepository;
use pushkind_auth::repository::user::DieselUserRepository;

mod common;

#[test]
fn test_hub_repository_crud() {
    let pool = common::setup_test_pool();
    let mut conn = pool.get().unwrap();
    let mut repo = DieselHubRepository::new(&mut conn);

    // Create
    let new_hub = NewHub {
        name: "TestHub".to_string(),
    };
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
}

#[test]
fn test_user_repository_crud() {
    let pool = common::setup_test_pool();
    let mut conn = pool.get().unwrap();

    let mut repo = DieselHubRepository::new(&mut conn);

    // Create Hub
    let new_hub = NewHub {
        name: "TestHub".to_string(),
    };
    let hub = repo.create(&new_hub).unwrap();
    assert_eq!(hub.name, new_hub.name);

    let mut repo = DieselUserRepository::new(&mut conn);

    // Create User
    let new_user = NewUser {
        name: Some("TestUser".to_string()),
        hub_id: hub.id,
        email: "test@test.test".to_string(),
        password: "test".to_string(),
    };
    let user = repo.create(&new_user).unwrap();
    assert_eq!(user.name, new_user.name);
    assert_eq!(user.email, new_user.email);

    // Get by email
    let found = repo.get_by_email(&new_user.email, hub.id).unwrap();
    assert!(found.is_some());

    // List
    let users = repo.list().unwrap();
    assert_eq!(users.len(), 1);

    assert!(repo.verify_password(&new_user.password, &user.password_hash));

    assert!(
        repo.login(&new_user.email, &new_user.password, hub.id)
            .is_ok_and(|u| u.is_some())
    );
}

#[test]
fn test_role_repository_crud() {
    let pool = common::setup_test_pool();
    let mut conn = pool.get().unwrap();
    let mut repo = DieselRoleRepository::new(&mut conn);

    // Create
    let new_role = NewRole {
        name: "TestRole".to_string(),
    };
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
}
