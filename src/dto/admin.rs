use crate::domain::role::Role;
use crate::domain::user::User;

/// Data required to populate the user modal, including the user (if found)
/// and the list of available roles.
#[derive(Clone, Debug, serde::Serialize)]
pub struct UserModalData {
    pub user: Option<User>,
    pub roles: Vec<Role>,
}
