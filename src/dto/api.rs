//! DTOs exposed by the REST API.

use crate::domain::hub::Hub;
use crate::domain::menu::Menu;
use crate::domain::role::Role;
use pushkind_common::domain::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ApiV1UsersQueryParams {
    pub role: Option<String>,
    pub query: Option<String>,
    pub page: Option<usize>,
}

/// Field-level validation error returned by JSON mutation endpoints.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiFieldErrorDto {
    pub field: String,
    pub message: String,
}

/// Successful JSON mutation response.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiMutationSuccessDto {
    pub message: String,
    pub redirect_to: Option<String>,
}

/// Failed JSON mutation response with optional field-level errors.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiMutationErrorDto {
    pub message: String,
    pub field_errors: Vec<ApiFieldErrorDto>,
}

/// DTO returned by API endpoints representing a user with roles and hub context.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserDto {
    pub sub: String,
    pub email: String,
    pub hub_id: i32,
    pub name: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

impl From<AuthenticatedUser> for UserDto {
    fn from(user: AuthenticatedUser) -> Self {
        Self {
            sub: user.sub,
            email: user.email,
            hub_id: user.hub_id,
            name: user.name,
            roles: user.roles,
            exp: user.exp,
        }
    }
}

/// Minimal hub representation used by resource-style API responses.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HubListItemDto {
    pub id: i32,
    pub name: String,
}

impl From<Hub> for HubListItemDto {
    fn from(hub: Hub) -> Self {
        Self {
            id: hub.id.get(),
            name: hub.name.into_inner(),
        }
    }
}

/// Hub summary embedded into IAM responses.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HubSummaryDto {
    pub id: i32,
    pub name: String,
}

impl From<Hub> for HubSummaryDto {
    fn from(hub: Hub) -> Self {
        Self {
            id: hub.id.get(),
            name: hub.name.into_inner(),
        }
    }
}

/// Editable current-user profile fields exposed via the IAM contract.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EditableProfileDto {
    pub name: String,
}

/// IAM-style response shape built on top of the existing current-user DTO.
///
/// This intentionally reuses [`UserDto`] as its identity core so the existing
/// `/api/v1/id` route can be assessed for reuse before introducing a separate
/// `/api/v1/iam` endpoint.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct IamDto {
    pub user: UserDto,
    pub current_hub: HubSummaryDto,
    pub editable_profile: EditableProfileDto,
}

/// Menu item exposed by hub-scoped menu APIs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HubMenuItemDto {
    pub name: String,
    pub url: String,
}

impl From<Menu> for HubMenuItemDto {
    fn from(menu: Menu) -> Self {
        Self {
            name: menu.name.into_inner(),
            url: menu.url.into_inner(),
        }
    }
}

/// Administrative role item exposed by the future admin dashboard API.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminRoleItemDto {
    pub id: i32,
    pub name: String,
    pub can_delete: bool,
}

impl From<Role> for AdminRoleItemDto {
    fn from(role: Role) -> Self {
        let id = role.id.get();
        Self {
            id,
            name: role.name.into_inner(),
            can_delete: id != 1,
        }
    }
}

/// Administrative hub item exposed by the future admin dashboard API.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminHubItemDto {
    pub id: i32,
    pub name: String,
    pub can_delete: bool,
}

impl From<Hub> for AdminHubItemDto {
    fn from(hub: Hub) -> Self {
        let id = hub.id.get();
        Self {
            id,
            name: hub.name.into_inner(),
            can_delete: id != 1,
        }
    }
}

/// Administrative menu item exposed by the future admin dashboard API.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminMenuItemDto {
    pub id: i32,
    pub name: String,
}

impl From<Menu> for AdminMenuItemDto {
    fn from(menu: Menu) -> Self {
        Self {
            id: menu.id.get(),
            name: menu.name.into_inner(),
        }
    }
}

/// Aggregate admin-only data not covered by the existing `/api/v1/users` list.
///
/// User list data is intentionally excluded so the future admin dashboard API
/// can reuse `/api/v1/users` instead of duplicating that functionality.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdminDashboardDto {
    pub roles: Vec<AdminRoleItemDto>,
    pub hubs: Vec<AdminHubItemDto>,
    pub admin_menu: Vec<AdminMenuItemDto>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hub::Hub;
    use crate::domain::menu::Menu;
    use crate::domain::role::Role;
    use crate::domain::types::{HubId, HubName, MenuId, MenuName, MenuUrl, RoleId, RoleName};
    use chrono::Utc;

    #[test]
    fn hub_list_item_from_hub_preserves_identity() {
        let now = Utc::now().naive_utc();
        let hub = Hub::new(
            HubId::new(7).unwrap(),
            HubName::new("Main").unwrap(),
            now,
            now,
        );

        let dto = HubListItemDto::from(hub);

        assert_eq!(dto.id, 7);
        assert_eq!(dto.name, "Main");
    }

    #[test]
    fn hub_menu_item_from_menu_preserves_name_and_url() {
        let menu = Menu::new(
            MenuId::new(3).unwrap(),
            MenuName::new("Orders").unwrap(),
            MenuUrl::new("https://example.com/orders").unwrap(),
            HubId::new(7).unwrap(),
        );

        let dto = HubMenuItemDto::from(menu);

        assert_eq!(dto.name, "Orders");
        assert_eq!(dto.url, "https://example.com/orders");
    }

    #[test]
    fn admin_dashboard_dto_excludes_users() {
        let role = Role::new(
            RoleId::new(1).unwrap(),
            RoleName::new("admin").unwrap(),
            Utc::now().naive_utc(),
            Utc::now().naive_utc(),
        );
        let hub = Hub::new(
            HubId::new(1).unwrap(),
            HubName::new("HQ").unwrap(),
            Utc::now().naive_utc(),
            Utc::now().naive_utc(),
        );
        let menu = Menu::new(
            MenuId::new(1).unwrap(),
            MenuName::new("Root").unwrap(),
            MenuUrl::new("https://example.com/").unwrap(),
            HubId::new(1).unwrap(),
        );

        let dto = AdminDashboardDto {
            roles: vec![AdminRoleItemDto::from(role)],
            hubs: vec![AdminHubItemDto::from(hub)],
            admin_menu: vec![AdminMenuItemDto::from(menu)],
        };

        assert_eq!(dto.roles.len(), 1);
        assert_eq!(dto.hubs.len(), 1);
        assert_eq!(dto.admin_menu.len(), 1);
    }
}
