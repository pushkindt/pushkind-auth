//! Forms backing the main application views and administrative pages.
//!
//! These payloads validate profile updates, role assignments, and hub or menu
//! creation before handing data off to the service layer.
use pushkind_common::routes::empty_string_as_none;
use serde::Deserialize;
use validator::Validate;

use crate::domain::types::{HubId, RoleId, TypeConstraintError};
use crate::domain::{
    hub::NewHub as DomainNewHub, menu::NewMenu as DomainNewMenu, role::NewRole as DomainNewRole,
    user::UpdateUser as DomainUpdateUser,
};

#[derive(Deserialize, Validate, Clone)]
/// Form used on the profile page to update the current user.
pub struct SaveUserForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub password: Option<String>,
}

impl TryFrom<SaveUserForm> for DomainUpdateUser {
    type Error = TypeConstraintError;

    fn try_from(form: SaveUserForm) -> Result<Self, Self::Error> {
        (&form).try_into()
    }
}

impl TryFrom<&SaveUserForm> for DomainUpdateUser {
    type Error = TypeConstraintError;

    fn try_from(form: &SaveUserForm) -> Result<Self, Self::Error> {
        Ok(Self {
            name: crate::domain::types::UserName::new(form.name.clone())?,
            password: form.password.clone(),
            roles: None,
        })
    }
}

#[derive(Deserialize, Validate, Clone)]
/// Request payload for creating a new role via the admin interface.
pub struct AddRoleForm {
    #[validate(length(min = 1))]
    pub name: String,
}

impl TryFrom<AddRoleForm> for DomainNewRole {
    type Error = TypeConstraintError;

    fn try_from(form: AddRoleForm) -> Result<Self, Self::Error> {
        Ok(Self {
            name: crate::domain::types::RoleName::new(form.name)?,
        })
    }
}

#[derive(Deserialize, Validate, Clone)]
/// Full user editing form used by administrators.
pub struct UpdateUserForm {
    #[validate(range(min = 1))]
    pub id: i32,
    #[validate(length(min = 1))]
    pub name: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub roles: Vec<i32>,
}

impl TryFrom<UpdateUserForm> for DomainUpdateUser {
    type Error = TypeConstraintError;

    fn try_from(form: UpdateUserForm) -> Result<Self, Self::Error> {
        let roles = form
            .roles
            .into_iter()
            .map(RoleId::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            name: crate::domain::types::UserName::new(form.name)?,
            password: form.password,
            roles: Some(roles),
        })
    }
}

#[derive(Deserialize, Validate, Clone)]
/// Parameters for adding a new hub.
pub struct AddHubForm {
    #[validate(length(min = 1))]
    pub name: String,
}

impl TryFrom<AddHubForm> for DomainNewHub {
    type Error = TypeConstraintError;

    fn try_from(form: AddHubForm) -> Result<Self, Self::Error> {
        Ok(Self {
            name: crate::domain::types::HubName::new(form.name)?,
        })
    }
}

#[derive(Deserialize, Validate, Clone)]
/// Payload for adding a menu entry to a hub.
pub struct AddMenuForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub url: String,
}

impl AddMenuForm {
    pub fn to_new_menu(&self, hub_id: HubId) -> Result<DomainNewMenu, TypeConstraintError> {
        Ok(DomainNewMenu {
            name: crate::domain::types::MenuName::new(self.name.clone())?,
            url: crate::domain::types::MenuUrl::new(self.url.clone())?,
            hub_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::domain::hub::NewHub as DomainNewHub;
    use crate::domain::role::NewRole as DomainNewRole;
    use crate::domain::types::{HubId, HubName, MenuName, MenuUrl, RoleId, RoleName, UserName};
    use crate::domain::user::UpdateUser as DomainUpdateUser;
    use crate::forms::main::{AddHubForm, AddMenuForm, AddRoleForm, SaveUserForm, UpdateUserForm};

    #[test]
    fn test_save_user_form_into_domain_update_user() {
        let form = SaveUserForm {
            name: "Alice".to_string(),
            password: Some("password".to_string()),
        };

        let update: DomainUpdateUser = form.try_into().expect("conversion failed");

        assert_eq!(update.name, UserName::new("Alice").unwrap());
        assert_eq!(update.password.as_deref(), Some("password"));
    }

    #[test]
    fn test_add_role_form_into_domain_new_role() {
        let form = AddRoleForm {
            name: "editor".to_string(),
        };

        let role: DomainNewRole = form.try_into().expect("conversion failed");

        assert_eq!(role.name, RoleName::new("editor").unwrap());
    }

    #[test]
    fn test_update_user_form_into_domain_update_user() {
        let form = UpdateUserForm {
            id: 1,
            name: "Bob".to_string(),
            password: Some("pwd".to_string()),
            roles: vec![1, 2],
        };

        let update: DomainUpdateUser = form.try_into().expect("conversion failed");

        assert_eq!(update.name, UserName::new("Bob").unwrap());
        assert_eq!(update.password.as_deref(), Some("pwd"));
        assert_eq!(
            update.roles.unwrap(),
            vec![RoleId::new(1).unwrap(), RoleId::new(2).unwrap()]
        );
    }

    #[test]
    fn test_add_hub_form_into_domain_new_hub() {
        let form = AddHubForm {
            name: "My Hub".to_string(),
        };

        let hub: DomainNewHub = form.try_into().expect("conversion failed");

        assert_eq!(hub.name, HubName::new("My Hub").unwrap());
    }

    #[test]
    fn test_add_menu_form_to_new_menu() {
        let form = AddMenuForm {
            name: "Menu".to_string(),
            url: "/".to_string(),
        };

        let menu = form
            .to_new_menu(HubId::new(3).unwrap())
            .expect("conversion failed");

        assert_eq!(menu.name, MenuName::new("Menu").unwrap());
        assert_eq!(menu.url, MenuUrl::new("/").unwrap());
        assert_eq!(menu.hub_id, HubId::new(3).unwrap());
    }
}
