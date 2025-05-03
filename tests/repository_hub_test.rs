use pushkind_auth::domain::hub::NewHub;
use pushkind_auth::repository::HubRepository;
use pushkind_auth::repository::hub::DieselHubRepository;

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
    assert_eq!(hubs.len(), 1);
}
