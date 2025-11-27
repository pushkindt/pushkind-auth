//! Authentication services for logging in users, registering new accounts, and listing hubs.

use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::domain::emailer::email::{NewEmail, NewEmailRecipient};
use pushkind_common::models::emailer::zmq::ZMQSendEmailMessage;
use pushkind_common::services::errors::{ServiceError, ServiceResult};
use pushkind_common::zmq::ZmqSender;

use crate::domain::types::{HubId, UserEmail};
use crate::dto::auth::SessionTokenDto;
use crate::forms::auth::{LoginForm, RecoverForm, RegisterForm};
use crate::repository::{HubReader, UserReader, UserWriter};
use crate::services::{map_type_error, validate_form};

/// Attempts to authenticate a user for the given hub.
///
/// On success returns an [`AuthenticatedUser`] containing the user's claims.
/// Returns [`ServiceError::Unauthorized`] when credentials are invalid.
pub fn login_user(
    email: &UserEmail,
    password: &str,
    hub_id: HubId,
    repo: &impl UserReader,
) -> ServiceResult<AuthenticatedUser> {
    let user_roles = repo
        .login(email, password, hub_id)?
        .ok_or(ServiceError::Unauthorized)?;
    Ok(AuthenticatedUser::from(user_roles))
}

/// Persists a new user using the provided repository.
///
/// Returns [`ServiceError`] if the underlying repository fails to create the user.
pub fn register_user(form: &RegisterForm, repo: &impl UserWriter) -> ServiceResult<()> {
    validate_form(form)?;
    let new_user = form.clone().into_domain().map_err(map_type_error)?;
    repo.create_user(&new_user)?;
    Ok(())
}

/// Retrieves all hubs available in the system.
pub fn list_hubs(repo: &impl HubReader) -> ServiceResult<Vec<crate::domain::hub::Hub>> {
    Ok(repo.list_hubs()?)
}

/// Encodes the provided claims into a JWT using the given secret.
pub fn issue_jwt(user: &AuthenticatedUser, secret: &str) -> ServiceResult<SessionTokenDto> {
    user.to_jwt(secret)
        .map(SessionTokenDto::from)
        .map_err(|_| ServiceError::Internal)
}

/// Verifies an incoming token and reissues a new session token
/// with the provided expiration in days.
pub fn reissue_session_from_token(
    token: &str,
    secret: &str,
    expiration_days: i64,
    repo: &impl UserReader,
) -> ServiceResult<SessionTokenDto> {
    let mut user =
        AuthenticatedUser::from_jwt(token, secret).map_err(|_| ServiceError::Unauthorized)?;
    // Ensure the user still exists and belongs to the hub before issuing a new session
    let email = UserEmail::new(&user.email).map_err(map_type_error)?;
    let hub_id = HubId::new(user.hub_id).map_err(map_type_error)?;
    match repo.get_user_by_email(&email, hub_id)? {
        Some(_) => {
            user.set_expiration(expiration_days);
            issue_jwt(&user, secret)
        }
        None => Err(ServiceError::Unauthorized),
    }
}

/// Performs login and issues a session JWT on success.
pub fn login_and_issue_token(
    form: &LoginForm,
    repo: &impl UserReader,
    secret: &str,
) -> ServiceResult<SessionTokenDto> {
    validate_form(form)?;
    let email = form.email().map_err(map_type_error)?;
    let hub_id = form.hub_id().map_err(map_type_error)?;
    let claims = login_user(&email, &form.password, hub_id, repo)?;
    issue_jwt(&claims, secret)
}

/// Sends a recovery email containing a login link with a short-lived token.
///
/// `base_url` should be something like `https://example.com` (scheme + host).
pub async fn send_recovery_email(
    zmq_sender: &ZmqSender,
    repo: &impl UserReader,
    secret: &str,
    form: &RecoverForm,
    base_url: &str,
) -> ServiceResult<()> {
    validate_form(form)?;
    let hub_id = form.hub_id().map_err(map_type_error)?;
    let email = form.email().map_err(map_type_error)?;
    let mut user: AuthenticatedUser = match repo.get_user_by_email(&email, hub_id)? {
        Some(user) => user.into(),
        None => return Err(ServiceError::NotFound),
    };

    // 1-day token for recovery
    user.set_expiration(1);
    let jwt = issue_jwt(&user, secret)?;
    let recovery_url = format!("{}/auth/login?token={}", base_url, jwt.token);

    let new_email = NewEmail {
        message: "Для входа в систему перейдите по ссылке: {recovery_url}\nЕсли вы не запрашивали восстановление, проигнорируйте это письмо.".to_string(),
        subject: Some("Восстановление пароля".to_string()),
        attachment: None,
        attachment_name: None,
        attachment_mime: None,
        hub_id: hub_id.get(),
        recipients: vec![NewEmailRecipient {
            address: email.as_str().to_string(),
            name: user.name.clone(),
            fields: std::iter::once(("recovery_url".to_string(), recovery_url)).collect(),
        }],
    };

    let zmq_message = ZMQSendEmailMessage::NewEmail(Box::new((user, new_email)));
    zmq_sender
        .send_json(&zmq_message)
        .await
        .map_err(|_| ServiceError::Internal)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hub::Hub;
    use crate::domain::types::{HubId, HubName, UserEmail, UserId};
    use crate::domain::user::{User, UserWithRoles};
    use crate::forms::auth::{LoginForm, RegisterForm};
    use crate::repository::mock::MockRepository;
    use chrono::Utc;
    use pushkind_common::repository::errors::RepositoryError;

    fn make_user(id: i32, email: &str, hub_id: i32) -> UserWithRoles {
        let now = Utc::now().naive_utc();
        UserWithRoles {
            user: User {
                id: UserId::new(id).unwrap(),
                email: UserEmail::new(email).unwrap(),
                name: Some(crate::domain::types::UserName::new("User").unwrap()),
                hub_id: HubId::new(hub_id).unwrap(),
                password_hash: "hash".into(),
                created_at: now,
                updated_at: now,
                roles: vec![],
            },
            roles: vec![],
        }
    }

    #[test]
    fn test_login_user_success() {
        let mut repo = MockRepository::new();
        let user = make_user(9, "a@b", 5);
        repo.expect_login()
            .returning(move |_, _, _| Ok(Some(user.clone())));
        let email = UserEmail::new("a@b").unwrap();
        let claims = login_user(&email, "pass", HubId::new(5).unwrap(), &repo).unwrap();
        assert_eq!(claims.email, "a@b");
    }

    #[test]
    fn test_login_user_invalid_password() {
        let mut repo = MockRepository::new();
        repo.expect_login().returning(|_, _, _| Ok(None));
        let email = UserEmail::new("a@b").unwrap();
        let res = login_user(&email, "wrong", HubId::new(2).unwrap(), &repo);
        assert!(matches!(res, Err(ServiceError::Unauthorized)));
    }

    #[test]
    fn test_login_user_unknown_user() {
        let mut repo = MockRepository::new();
        repo.expect_login().returning(|_, _, _| Ok(None));
        let email = UserEmail::new("missing@ex").unwrap();
        let res = login_user(&email, "pass", HubId::new(1).unwrap(), &repo);
        assert!(matches!(res, Err(ServiceError::Unauthorized)));
    }

    #[test]
    fn test_register_user_success() {
        let mut repo = MockRepository::new();
        repo.expect_create_user().returning(|new| {
            let now = Utc::now().naive_utc();
            Ok(User {
                id: UserId::new(1).unwrap(),
                email: new.email.clone(),
                name: new.name.clone(),
                hub_id: new.hub_id,
                password_hash: "".into(),
                created_at: now,
                updated_at: now,
                roles: vec![],
            })
        });
        let form = RegisterForm {
            email: "x@y".into(),
            password: "p".into(),
            hub_id: 1,
        };
        let res = register_user(&form, &repo);
        assert!(res.is_ok());
    }

    #[test]
    fn test_register_user_error() {
        let mut repo = MockRepository::new();
        repo.expect_create_user()
            .returning(|_| Err(RepositoryError::ValidationError("fail".into())));
        let form = RegisterForm {
            email: "x@y".into(),
            password: "p".into(),
            hub_id: 1,
        };
        let res = register_user(&form, &repo);
        assert!(res.is_err());
    }

    #[test]
    fn test_register_user_validation_error() {
        let repo = MockRepository::new();
        let form = RegisterForm {
            email: "invalid-email".into(),
            password: "p".into(),
            hub_id: 0,
        };

        let res = register_user(&form, &repo);

        assert!(matches!(res, Err(ServiceError::Form(_))));
    }

    #[test]
    fn test_list_hubs_returns_all() {
        let mut repo = MockRepository::new();
        let now = Utc::now().naive_utc();
        let hubs = vec![
            Hub {
                id: HubId::new(1).unwrap(),
                name: HubName::new("h1").unwrap(),
                created_at: now,
                updated_at: now,
            },
            Hub {
                id: HubId::new(2).unwrap(),
                name: HubName::new("h2").unwrap(),
                created_at: now,
                updated_at: now,
            },
        ];
        let hubs_clone = hubs.clone();
        repo.expect_list_hubs()
            .returning(move || Ok(hubs_clone.clone()));
        let res = list_hubs(&repo).unwrap();
        assert_eq!(res, hubs);
    }

    #[test]
    fn test_list_hubs_empty() {
        let mut repo = MockRepository::new();
        repo.expect_list_hubs().returning(|| Ok(vec![]));
        let res = list_hubs(&repo).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_reissue_session_from_token_requires_existing_user() {
        let mut repo = MockRepository::new();
        repo.expect_get_user_by_email().returning(|_, _| Ok(None));
        let mut user: AuthenticatedUser = make_user(1, "a@b", 2).into();
        user.set_expiration(1);
        let token = issue_jwt(&user, "secret").unwrap();
        let res = reissue_session_from_token(&token.token, "secret", 7, &repo);
        assert!(matches!(res, Err(ServiceError::Unauthorized)));
    }

    #[test]
    fn test_reissue_session_from_token_success() {
        let mut repo = MockRepository::new();
        let uwr = make_user(1, "a@b", 2);
        let uwr_clone = uwr.clone();
        repo.expect_get_user_by_email()
            .returning(move |_, _| Ok(Some(uwr_clone.clone())));
        let mut user: AuthenticatedUser = uwr.into();
        user.set_expiration(1);
        let token = issue_jwt(&user, "secret").unwrap();
        let res = reissue_session_from_token(&token.token, "secret", 7, &repo);
        assert!(res.is_ok());
    }

    #[test]
    fn test_login_and_issue_token_validation_error() {
        let repo = MockRepository::new();
        let form = LoginForm {
            email: "not-an-email".into(),
            password: "".into(),
            hub_id: 0,
        };

        let res = login_and_issue_token(&form, &repo, "secret");

        assert!(matches!(res, Err(ServiceError::Form(_))));
    }
}
