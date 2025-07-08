use pushkind_auth::domain::hub::NewHub as DomainNewHub;
use pushkind_auth::domain::role::NewRole as DomainNewRole;
use pushkind_auth::domain::user::NewUser as DomainNewUser;
use pushkind_auth::domain::user::UpdateUser as DomainUpdateUser;
use pushkind_auth::forms::auth::RegisterForm;
use pushkind_auth::forms::main::{AddHubForm, AddRoleForm, SaveUserForm, UpdateUserForm};

#[test]
fn test_register_form_into_domain_new_user() {
    let form = RegisterForm {
        email: "test@example.com".to_string(),
        password: "secret".to_string(),
        hub_id: 7,
    };

    let user: DomainNewUser = (&form).into();

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.password, "secret");
    assert_eq!(user.hub_id, 7);
    assert_eq!(user.name, None);
}

#[test]
fn test_save_user_form_into_domain_update_user() {
    let form = SaveUserForm {
        name: Some("Alice".to_string()),
        password: Some("password".to_string()),
    };

    let update: DomainUpdateUser = (&form).into();

    assert_eq!(update.name, Some("Alice"));
    assert_eq!(update.password, Some("password"));
}

#[test]
fn test_add_role_form_into_domain_new_role() {
    let form = AddRoleForm {
        name: "editor".to_string(),
    };

    let role: DomainNewRole = (&form).into();

    assert_eq!(role.name, "editor");
}

#[test]
fn test_update_user_form_into_domain_update_user() {
    let form = UpdateUserForm {
        id: 1,
        name: Some("Bob".to_string()),
        password: Some("pwd".to_string()),
        roles: vec![1, 2],
    };

    let update: DomainUpdateUser = (&form).into();

    assert_eq!(update.name, Some("Bob"));
    assert_eq!(update.password, Some("pwd"));
}

#[test]
fn test_add_hub_form_into_domain_new_hub() {
    let form = AddHubForm {
        name: "My Hub".to_string(),
    };

    let hub: DomainNewHub = (&form).into();

    assert_eq!(hub.name, "My Hub");
}
