use chrono::NaiveDateTime;

use pushkind_auth::domain::user::NewUser;
use pushkind_auth::domain::user::User;
use pushkind_auth::forms::auth::LoginForm;
use pushkind_auth::repository::UserRepository;
use pushkind_auth::services::auth::{LoginError, handle_login};

struct MockUserRepo {
    pub user: Option<User>,
    pub password_hash: String,
}

impl UserRepository for MockUserRepo {
    fn get_by_id(&mut self, _id: i32) -> anyhow::Result<Option<User>> {
        Ok(self.user.clone())
    }

    fn get_by_email(&mut self, _email: &str, _hub_id: i32) -> anyhow::Result<Option<User>> {
        Ok(self.user.clone())
    }

    fn create(&mut self, _new_user: &NewUser) -> anyhow::Result<User> {
        unimplemented!()
    }

    fn list(&mut self) -> anyhow::Result<Vec<User>> {
        unimplemented!()
    }

    fn verify_password(&self, password: &str, stored_hash: &str) -> bool {
        password == stored_hash
    }
}

fn valid_form() -> LoginForm {
    LoginForm {
        email: "user@test.com".into(),
        password: "password".into(),
        hub_id: 1,
    }
}

#[test]
fn test_handle_login_success() {
    let user = User {
        id: 1,
        name: Some("User".into()),
        email: "user@test.com".into(),
        hub_id: 1,
        password_hash: "password".to_string(), // assume plain password for testing
        created_at: NaiveDateTime::default(),
        updated_at: NaiveDateTime::default(),
    };

    let form = valid_form();
    let mut repo = MockUserRepo {
        user: Some(user.clone()),
        password_hash: user.password_hash.clone(),
    };

    let resp = handle_login(&form, &mut repo);

    assert!(resp.is_ok());
}

#[test]
fn test_handle_login_user_not_found() {
    let form = valid_form();
    let mut repo = MockUserRepo {
        user: None,
        password_hash: "".to_string(),
    };

    let resp = handle_login(&form, &mut repo);

    assert!(matches!(resp, Err(LoginError::InvalidCredentials)));
}

#[test]
fn test_handle_login_invalid_password() {
    let user = User {
        id: 1,
        name: Some("User".into()),
        email: "user@test.com".into(),
        hub_id: 1,
        password_hash: "wrong".to_string(), // expected hash does not match form input
        created_at: NaiveDateTime::default(),
        updated_at: NaiveDateTime::default(),
    };

    let form = valid_form();
    let mut repo = MockUserRepo {
        user: Some(user),
        password_hash: "wrong".to_string(),
    };

    let resp = handle_login(&form, &mut repo);

    assert!(matches!(resp, Err(LoginError::InvalidCredentials)));
}
