use pushkind_auth::domain::role::NewRole;
use pushkind_auth::repository::RoleRepository;
use pushkind_auth::repository::role::DieselRoleRepository;

mod common;

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
