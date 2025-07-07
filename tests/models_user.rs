use bcrypt::verify;
use pushkind_auth::domain::user::NewUser as DomainNewUser;
use pushkind_auth::models::user::NewUser;

#[test]
fn test_new_user_try_from() {
    let domain = DomainNewUser {
        email: "john@example.com".to_string(),
        name: Some("John Doe".to_string()),
        hub_id: 5,
        password: "super_secret".to_string(),
    };

    let db_user = NewUser::try_from(&domain).expect("conversion failed");

    assert_eq!(db_user.email, domain.email);
    assert_eq!(db_user.name, domain.name.as_deref());
    assert_eq!(db_user.hub_id, domain.hub_id);
    assert!(verify(&domain.password, &db_user.password_hash).unwrap());
}
