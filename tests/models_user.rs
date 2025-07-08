use bcrypt::verify;
use pushkind_auth::domain::user::NewUser as DomainNewUser;
use pushkind_auth::models::user::NewUser;

#[test]
fn test_new_user_try_from() {
    let domain = DomainNewUser {
        email: "john@example.com",
        name: Some("John Doe"),
        hub_id: 5,
        password: "super_secret",
    };

    let db_user = NewUser::try_from(domain).expect("conversion failed");

    assert_eq!(db_user.email, "john@example.com");
    assert_eq!(db_user.name, Some("John Doe"));
    assert_eq!(db_user.hub_id, 5);
    assert!(verify("super_secret", &db_user.password_hash).unwrap());
}
