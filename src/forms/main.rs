use serde::Deserialize;

use crate::domain::{
    hub::NewHub as DomainNewHub, role::NewRole as DomainNewRole,
    user::UpdateUser as DomainUpdateUser,
};

#[derive(Deserialize)]
/// Form used on the profile page to update the current user.
pub struct SaveUserForm {
    pub name: Option<String>,
    pub password: Option<String>,
}

impl<'a> From<&'a SaveUserForm> for DomainUpdateUser<'a> {
    fn from(form: &'a SaveUserForm) -> Self {
        Self {
            name: form.name.as_deref(),
            password: form.password.as_deref(),
        }
    }
}

#[derive(Deserialize)]
/// Request payload for creating a new role via the admin interface.
pub struct AddRoleForm {
    pub name: String,
}

impl<'a> From<&'a AddRoleForm> for DomainNewRole<'a> {
    fn from(form: &'a AddRoleForm) -> Self {
        Self {
            name: form.name.as_str(),
        }
    }
}

#[derive(Deserialize)]
/// Full user editing form used by administrators.
pub struct UpdateUserForm {
    pub id: i32,
    pub name: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub roles: Vec<i32>,
}

impl<'a> From<&'a UpdateUserForm> for DomainUpdateUser<'a> {
    fn from(form: &'a UpdateUserForm) -> Self {
        Self {
            name: form.name.as_deref(),
            password: form.password.as_deref(),
        }
    }
}

#[derive(Deserialize)]
/// Parameters for adding a new hub.
pub struct AddHubForm {
    pub name: String,
}

impl<'a> From<&'a AddHubForm> for DomainNewHub<'a> {
    fn from(form: &'a AddHubForm) -> Self {
        Self {
            name: form.name.as_str(),
        }
    }
}

#[derive(Deserialize)]
/// Payload for adding a menu entry to a hub.
pub struct AddMenuForm {
    pub name: String,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use crate::domain::hub::NewHub as DomainNewHub;
    use crate::domain::role::NewRole as DomainNewRole;
    use crate::domain::user::UpdateUser as DomainUpdateUser;
    use crate::forms::main::{AddHubForm, AddRoleForm, SaveUserForm, UpdateUserForm};

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
}
