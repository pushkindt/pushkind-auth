use pushkind_auth::domain::hub::NewHub;
use pushkind_auth::domain::user::NewUser;
use pushkind_auth::repository::HubRepository;
use pushkind_auth::repository::UserRepository;
use pushkind_auth::repository::hub::DieselHubRepository;
use pushkind_auth::repository::user::DieselUserRepository;

mod common;

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
    assert_eq!(hub.name, "TestHub");

    let mut repo = DieselUserRepository::new(&mut conn);

    // Create User
    let new_user = NewUser {
        name: Some("TestUser".to_string()),
        hub_id: hub.id,
        email: "test@test.test".to_string(),
        password_hash: "test".to_string(),
    };
    let user = repo.create(&new_user).unwrap();
    assert_eq!(user.name, Some("TestUser".to_string()));
    assert_eq!(user.email, "test@test.test");

    // Get by email
    let found = repo.get_by_email("test@test.test", 1).unwrap();
    assert!(found.is_some());

    // List
    let users = repo.list().unwrap();
    assert_eq!(users.len(), 1);
}
