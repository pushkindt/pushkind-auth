use pushkind_auth::domain::hub::NewHub;
use pushkind_auth::domain::role::NewRole;
use pushkind_auth::domain::role::NewUserRole;
use pushkind_auth::domain::user::NewUser;
use pushkind_auth::domain::user::UpdateUser;
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

    let mut repo = DieselRoleRepository::new(&mut conn);

    let new_role = NewRole {
        name: "TestRole".to_string(),
    };

    let role = repo.create(&new_role).unwrap();

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

    let new_user_role = NewUserRole {
        user_id: user.id,
        role_id: role.id,
    };

    let user_role = repo.assign_role(&new_user_role).unwrap();
    assert!(user_role.role_id == role.id && user_role.user_id == user.id);

    // Get by email
    let found = repo.get_by_email(&new_user.email, hub.id).unwrap();
    assert!(found.is_some());

    // List
    let users = repo.list(hub.id).unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].1.len(), 1);

    assert!(repo.verify_password(&new_user.password, &user.password_hash));

    assert!(
        repo.login(&new_user.email, &new_user.password, hub.id)
            .is_ok_and(|u| u.is_some())
    );

    let user_roles = repo.get_roles(user.id).unwrap();

    assert!(user_roles.iter().any(|r| r.name == role.name));

    let user = repo
        .update(
            user.id,
            &UpdateUser {
                name: Some("new name".to_string()),
                password: Some("new password".to_string()),
            },
        )
        .unwrap();
    assert_eq!(user.name, Some("new name".to_string()));
    assert!(repo.verify_password("new password", &user.password_hash));

    repo.assign_roles(user.id, &[]).unwrap();

    let roles = repo.get_roles(user.id).unwrap();
    assert!(roles.is_empty());

    repo.delete(user.id).unwrap();
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
