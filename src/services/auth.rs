use pushkind_common::domain::auth::AuthenticatedUser;
use pushkind_common::services::errors::{ServiceError, ServiceResult};

use crate::domain::user::NewUser;
use crate::repository::{HubReader, UserReader, UserWriter};

pub fn login_user(
    email: &str,
    password: &str,
    hub_id: i32,
    repo: &impl UserReader,
) -> ServiceResult<AuthenticatedUser> {
    let user_roles = repo
        .login(email, password, hub_id)?
        .ok_or_else(|| ServiceError::Unauthorized)?;
    Ok(AuthenticatedUser::from(user_roles))
}

pub fn register_user(new_user: &NewUser, repo: &impl UserWriter) -> ServiceResult<()> {
    repo.create_user(new_user)?;
    Ok(())
}

pub fn list_hubs(repo: &impl HubReader) -> ServiceResult<Vec<crate::domain::hub::Hub>> {
    Ok(repo.list_hubs()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::test::TestRepository;

    #[test]
    fn test_login_user_success() {
        let repo = TestRepository::with_users(vec![TestRepository::make_user(9, "a@b", 5, vec![])]);
        let claims = login_user("a@b", "pass", 5, &repo).unwrap();
        assert_eq!(claims.email, "a@b");
    }

    #[test]
    fn test_register_user() {
        let repo = TestRepository::new();
        let new = NewUser::new("x@y".into(), None, 1, "p".into());
        let res = register_user(&new, &repo);
        assert!(res.is_ok());
    }
}
