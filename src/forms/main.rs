//! Forms backing the main application views and administrative pages.
//!
//! These payloads validate profile updates, role assignments, and hub or menu
//! creation before handing data off to the service layer.
use pushkind_common::routes::empty_string_as_none;
use serde::Deserialize;
use validator::Validate;

use crate::domain::types::{
    HubId, HubName, MenuName, MenuUrl, RoleId, RoleName, UserName, UserPassword,
};
use crate::domain::{
    hub::NewHub as DomainNewHub, menu::NewMenu as DomainNewMenu, role::NewRole as DomainNewRole,
    user::UpdateUser as DomainUpdateUser,
};
use crate::forms::FormError;

#[derive(Deserialize, Validate, Clone)]
/// Form used on the profile page to update the current user.
pub struct SaveUserForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub password: Option<String>,
}

// Payload after validation and conversion to domain types.
pub struct SaveUserPayload {
    pub name: UserName,
    pub password: Option<UserPassword>,
}

#[derive(Deserialize, Validate, Clone)]
/// Full user editing form used by administrators.
pub struct UpdateUserForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub roles: Vec<i32>,
}

// Payload after validation and conversion to domain types.
pub struct UpdateUserPayload {
    pub name: UserName,
    pub password: Option<UserPassword>,
    pub roles: Option<Vec<RoleId>>,
}

#[derive(Deserialize, Validate, Clone)]
/// Request payload for creating a new role via the admin interface.
pub struct AddRoleForm {
    #[validate(length(min = 1))]
    pub name: String,
}

// Payload after validation and conversion to domain types.
pub struct AddRolePayload {
    pub name: RoleName,
}

#[derive(Deserialize, Validate, Clone)]
/// Parameters for adding a new hub.
pub struct AddHubForm {
    #[validate(length(min = 1))]
    pub name: String,
}

// Payload after validation and conversion to domain types.
pub struct AddHubPayload {
    pub name: HubName,
}

#[derive(Deserialize, Validate, Clone)]
/// Payload for adding a menu entry to a hub.
pub struct AddMenuForm {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(url)]
    pub url: String,
}

// Payload after validation and conversion to domain types.
pub struct AddMenuPayload {
    pub name: MenuName,
    pub url: MenuUrl,
}

impl TryFrom<SaveUserForm> for SaveUserPayload {
    type Error = FormError;

    fn try_from(form: SaveUserForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            name: UserName::new(form.name).map_err(|_| FormError::InvalidName)?,
            password: match form.password {
                Some(pwd) => Some(UserPassword::new(pwd).map_err(|_| FormError::InvalidPassword)?),
                None => None,
            },
        })
    }
}

impl From<SaveUserPayload> for DomainUpdateUser {
    fn from(payload: SaveUserPayload) -> Self {
        Self {
            name: payload.name,
            password: payload.password,
            roles: None,
        }
    }
}

impl TryFrom<UpdateUserForm> for UpdateUserPayload {
    type Error = FormError;

    fn try_from(form: UpdateUserForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            name: UserName::new(form.name).map_err(|_| FormError::InvalidName)?,
            password: match form.password {
                Some(pwd) => Some(UserPassword::new(pwd).map_err(|_| FormError::InvalidPassword)?),
                None => None,
            },
            roles: {
                let roles = form
                    .roles
                    .into_iter()
                    .map(|id| RoleId::new(id).map_err(|_| FormError::InvalidRoleId))
                    .collect::<Result<Vec<_>, _>>()?;
                Some(roles)
            },
        })
    }
}

impl From<UpdateUserPayload> for DomainUpdateUser {
    fn from(payload: UpdateUserPayload) -> Self {
        Self {
            name: payload.name,
            password: payload.password,
            roles: payload.roles,
        }
    }
}

impl TryFrom<AddRoleForm> for AddRolePayload {
    type Error = FormError;

    fn try_from(form: AddRoleForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            name: RoleName::new(form.name).map_err(|_| FormError::InvalidName)?,
        })
    }
}

impl From<AddRolePayload> for DomainNewRole {
    fn from(payload: AddRolePayload) -> Self {
        Self { name: payload.name }
    }
}

impl TryFrom<AddHubForm> for AddHubPayload {
    type Error = FormError;

    fn try_from(form: AddHubForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            name: HubName::new(form.name).map_err(|_| FormError::InvalidName)?,
        })
    }
}

impl From<AddHubPayload> for DomainNewHub {
    fn from(payload: AddHubPayload) -> Self {
        Self { name: payload.name }
    }
}

impl TryFrom<AddMenuForm> for AddMenuPayload {
    type Error = FormError;

    fn try_from(form: AddMenuForm) -> Result<Self, Self::Error> {
        form.validate().map_err(FormError::Validation)?;
        Ok(Self {
            name: MenuName::new(form.name).map_err(|_| FormError::InvalidName)?,
            url: MenuUrl::new(form.url).map_err(|_| FormError::InvalidUrl)?,
        })
    }
}

impl AddMenuPayload {
    pub fn into_new_menu(self, hub_id: HubId) -> DomainNewMenu {
        DomainNewMenu {
            name: self.name,
            url: self.url,
            hub_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::domain::hub::NewHub as DomainNewHub;
    use crate::domain::role::NewRole as DomainNewRole;
    use crate::domain::types::{
        HubId, HubName, MenuName, MenuUrl, RoleId, RoleName, UserName, UserPassword,
    };
    use crate::domain::user::UpdateUser as DomainUpdateUser;
    use crate::forms::main::{
        AddHubForm, AddHubPayload, AddMenuForm, AddMenuPayload, AddRoleForm, AddRolePayload,
        SaveUserForm, SaveUserPayload, UpdateUserForm, UpdateUserPayload,
    };

    #[test]
    fn test_save_user_form_into_domain_update_user() {
        let form = SaveUserForm {
            name: "Alice".to_string(),
            password: Some("password".to_string()),
        };

        let payload: SaveUserPayload = form.try_into().expect("conversion failed");

        let update: DomainUpdateUser = payload.into();

        assert_eq!(update.name.as_str(), "Alice");
        assert_eq!(
            update.password.as_ref().map(UserPassword::as_str),
            Some("password")
        );
    }

    #[test]
    fn test_add_role_form_into_domain_new_role() {
        let form = AddRoleForm {
            name: "editor".to_string(),
        };

        let payload: AddRolePayload = form.try_into().expect("conversion failed");

        let role: DomainNewRole = payload.into();

        assert_eq!(role.name, RoleName::new("editor").unwrap());
    }

    #[test]
    fn test_update_user_form_into_domain_update_user() {
        let form = UpdateUserForm {
            name: "Bob".to_string(),
            password: Some("pwd".to_string()),
            roles: vec![1, 2],
        };

        let payload: UpdateUserPayload = form.try_into().expect("conversion failed");

        let update: DomainUpdateUser = payload.into();

        assert_eq!(update.name, UserName::new("Bob").unwrap());
        assert_eq!(
            update
                .password
                .as_ref()
                .map(crate::domain::types::UserPassword::as_str),
            Some("pwd")
        );
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

        let payload: AddHubPayload = form.try_into().expect("conversion failed");

        let hub: DomainNewHub = payload.into();

        assert_eq!(hub.name, HubName::new("My Hub").unwrap());
    }

    #[test]
    fn test_add_menu_form_to_new_menu() {
        let form = AddMenuForm {
            name: "Menu".to_string(),
            url: "https://app.test.me/".to_string(),
        };

        let payload: AddMenuPayload = form.try_into().expect("conversion failed");

        let menu = payload.into_new_menu(HubId::new(3).unwrap());

        assert_eq!(menu.name, MenuName::new("Menu").unwrap());
        assert_eq!(menu.url, MenuUrl::new("https://app.test.me/").unwrap());
        assert_eq!(menu.hub_id, HubId::new(3).unwrap());
    }
}
