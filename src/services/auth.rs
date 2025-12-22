//! Authentication services for logging in users, registering new accounts, and listing hubs.

use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::services::errors::{ServiceError, ServiceResult};
use pushkind_common::zmq::{ZmqSender, ZmqSenderExt};
use pushkind_emailer::domain::email::{NewEmail, NewEmailRecipient};
use pushkind_emailer::domain::types::{
    EmailBody, EmailSubject, HubId as EmailHubId, RecipientEmail, RecipientName,
};
use pushkind_emailer::models::zmq::ZMQSendEmailMessage;

use crate::domain::types::{HubId, UserEmail};
use crate::dto::auth::SessionTokenDto;
use crate::forms::auth::{
    LoginForm, LoginPayload, RecoverForm, RecoverPayload, RegisterForm, RegisterPayload,
};
use crate::repository::{HubReader, UserReader, UserWriter};

/// Persists a new user using the provided repository.
///
/// Returns [`ServiceError`] if the underlying repository fails to create the user.
pub fn register_user(form: RegisterForm, repo: &impl UserWriter) -> ServiceResult<()> {
    let payload: RegisterPayload = form.try_into()?;

    let new_user = payload.into();
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
    let email = UserEmail::new(&user.email)?;
    let hub_id = HubId::new(user.hub_id)?;
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
    form: LoginForm,
    repo: &impl UserReader,
    secret: &str,
) -> ServiceResult<SessionTokenDto> {
    let payload: LoginPayload = form.try_into()?;

    let user_roles = repo
        .login(&payload.email, &payload.password, payload.hub_id)?
        .ok_or(ServiceError::Unauthorized)?;
    let claims = AuthenticatedUser::from(user_roles);
    issue_jwt(&claims, secret)
}

/// Sends a recovery email containing a login link with a short-lived token.
///
/// `base_url` should be something like `https://example.com` (scheme + host).
pub async fn send_recovery_email(
    zmq_sender: &ZmqSender,
    repo: &impl UserReader,
    secret: &str,
    form: RecoverForm,
    base_url: &str,
) -> ServiceResult<()> {
    let payload: RecoverPayload = RecoverForm::try_into(form)?;

    let mut user: AuthenticatedUser =
        match repo.get_user_by_email(&payload.email, payload.hub_id)? {
            Some(user) => user.into(),
            None => return Err(ServiceError::NotFound),
        };

    // 1-day token for recovery
    user.set_expiration(1);
    let jwt = issue_jwt(&user, secret)?;
    let recovery_url = format!("{}/auth/login?token={}", base_url, jwt.token);

    let new_email = NewEmail {
        message: EmailBody::new(
            "Для входа в систему перейдите по ссылке: {recovery_url}\nЕсли вы не запрашивали восстановление, проигнорируйте это письмо.",
        )?,
        subject: Some(EmailSubject::new("Восстановление пароля")?),
        attachment: None,
        attachment_name: None,
        attachment_mime: None,
        hub_id: EmailHubId::new(payload.hub_id.get())?,
        recipients: vec![NewEmailRecipient {
            address: RecipientEmail::new(payload.email.as_str())?,
            name: RecipientName::new(&user.name)?,
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
    use crate::domain::types::{HubId, HubName, UserEmail, UserId, UserName};
    use crate::domain::user::{User, UserWithRoles};
    use crate::forms::auth::{LoginForm, RegisterForm};
    use crate::repository::mock::MockRepository;
    use bcrypt::{DEFAULT_COST, hash};
    use chrono::Utc;
    use pushkind_common::repository::errors::RepositoryError;

    fn make_secret() -> String {
        "my_secret_key".to_string()
    }

    fn make_user(id: i32, email: &str, hub_id: i32) -> UserWithRoles {
        let now = Utc::now().naive_utc();
        let user = User::new(
            UserId::new(id).unwrap(),
            UserEmail::new(email).unwrap(),
            Some(UserName::new("User").unwrap()),
            HubId::new(hub_id).unwrap(),
            hash("pass", DEFAULT_COST).unwrap(),
            now,
            now,
            vec![],
        );
        UserWithRoles::new(user, vec![])
    }

    #[test]
    fn test_login_user_success() {
        let mut repo = MockRepository::new();
        let user = make_user(9, "a@b", 5);
        repo.expect_login()
            .returning(move |_, _, _| Ok(Some(user.clone())));

        let form = LoginForm {
            email: "a@b".into(),
            password: "pass".into(),
            hub_id: 5,
        };

        let secret = make_secret();

        let claims = login_and_issue_token(form, &repo, &secret).unwrap();
        assert!(!claims.token.is_empty());
    }

    #[test]
    fn test_login_user_invalid_password() {
        let mut repo = MockRepository::new();
        repo.expect_login().returning(|_, _, _| Ok(None));

        let form = LoginForm {
            email: "a@b".into(),
            password: "wrong".into(),
            hub_id: 5,
        };

        let secret = make_secret();

        let res = login_and_issue_token(form, &repo, &secret);
        assert!(matches!(res, Err(ServiceError::Unauthorized)));
    }

    #[test]
    fn test_login_user_unknown_user() {
        let mut repo = MockRepository::new();
        repo.expect_login().returning(|_, _, _| Ok(None));

        let form = LoginForm {
            email: "missing@ex".into(),
            password: "pass".into(),
            hub_id: 1,
        };

        let secret = make_secret();

        let res = login_and_issue_token(form, &repo, &secret);
        assert!(matches!(res, Err(ServiceError::Unauthorized)));
    }

    #[test]
    fn test_register_user_success() {
        let mut repo = MockRepository::new();
        repo.expect_create_user().returning(|new| {
            let now = Utc::now().naive_utc();
            Ok(User::new(
                UserId::new(1).unwrap(),
                new.email.clone(),
                new.name.clone(),
                new.hub_id,
                "".into(),
                now,
                now,
                vec![],
            ))
        });
        let form = RegisterForm {
            email: "x@y".into(),
            password: "p".into(),
            hub_id: 1,
        };
        let res = register_user(form, &repo);
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
        let res = register_user(form, &repo);
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

        let res = register_user(form, &repo);

        assert!(matches!(res, Err(ServiceError::Form(_))));
    }

    #[test]
    fn test_list_hubs_returns_all() {
        let mut repo = MockRepository::new();
        let now = Utc::now().naive_utc();
        let hubs = vec![
            Hub::new(
                HubId::new(1).unwrap(),
                HubName::new("h1").unwrap(),
                now,
                now,
            ),
            Hub::new(
                HubId::new(2).unwrap(),
                HubName::new("h2").unwrap(),
                now,
                now,
            ),
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

        let res = login_and_issue_token(form, &repo, "secret");

        assert!(matches!(res, Err(ServiceError::Form(_))));
    }
}
