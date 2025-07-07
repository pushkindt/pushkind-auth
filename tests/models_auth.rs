use pushkind_auth::models::auth::AuthenticatedUser;
use pushkind_auth::domain::user::User;
use pushkind_auth::domain::role::Role;
use chrono::{Utc, NaiveDateTime, Duration};
use jsonwebtoken::{DecodingKey, Validation, decode};

#[test]
fn test_from_user_sets_fields() {
    let now = Utc::now().naive_utc();
    let user = User {
        id: 1,
        email: "user@example.com".to_string(),
        name: Some("Example".to_string()),
        hub_id: 99,
        password_hash: "hash".to_string(),
        created_at: now,
        updated_at: now,
    };

    let role1 = Role {
        id: 1,
        name: "admin".to_string(),
        created_at: now,
        updated_at: now,
    };
    let role2 = Role {
        id: 2,
        name: "editor".to_string(),
        created_at: now,
        updated_at: now,
    };
    let roles = vec![role1.clone(), role2.clone()];

    let auth = AuthenticatedUser::from_user(&user, &roles);

    assert_eq!(auth.sub, user.id.to_string());
    assert_eq!(auth.email, user.email);
    assert_eq!(auth.hub_id, user.hub_id);
    assert_eq!(auth.name, user.name.unwrap());
    assert_eq!(auth.roles, vec![role1.name, role2.name]);

    let now_ts = Utc::now().timestamp() as usize;
    let diff = if auth.exp > now_ts { auth.exp - now_ts } else { now_ts - auth.exp };
    let seven_days = 7 * 24 * 60 * 60;
    assert!(diff >= seven_days - 5 && diff <= seven_days + 5);
}

#[test]
fn test_to_jwt_round_trip() {
    let now = Utc::now().naive_utc();
    let user = User {
        id: 42,
        email: "jwt@example.com".to_string(),
        name: None,
        hub_id: 7,
        password_hash: "hash".to_string(),
        created_at: now,
        updated_at: now,
    };
    let role = Role {
        id: 3,
        name: "viewer".to_string(),
        created_at: now,
        updated_at: now,
    };
    let mut auth = AuthenticatedUser::from_user(&user, &[role.clone()]);
    let secret = "mysecret";

    let token = auth.to_jwt(secret).expect("failed to encode token");

    let decoded = decode::<AuthenticatedUser>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .expect("decode failed")
    .claims;

    assert_eq!(decoded.sub, auth.sub);
    assert_eq!(decoded.email, auth.email);
    assert_eq!(decoded.hub_id, auth.hub_id);
    assert_eq!(decoded.name, auth.name);
    assert_eq!(decoded.roles, vec![role.name]);
    assert_eq!(decoded.exp, auth.exp);
}
