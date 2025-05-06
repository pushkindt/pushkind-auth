use crate::domain::user::User;
use crate::forms::auth::LoginForm;
use crate::repository::UserRepository;

#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
    InternalError(anyhow::Error),
}

pub fn handle_login(form: &LoginForm, repo: &mut impl UserRepository) -> Result<User, LoginError> {
    let user = repo
        .get_by_email(&form.email, form.hub_id)
        .map_err(LoginError::InternalError)?;

    let Some(user) = user else {
        return Err(LoginError::InvalidCredentials);
    };

    if !repo.verify_password(&form.password, &user.password_hash) {
        return Err(LoginError::InvalidCredentials);
    }

    Ok(user)
}
