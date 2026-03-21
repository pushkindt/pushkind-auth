//! DTOs used to bootstrap React-rendered pages.

use pushkind_common::domain::auth::AuthenticatedUser;
use serde::Serialize;

use crate::domain::hub::Hub;
use crate::domain::menu::Menu;
use crate::domain::role::Role;
use crate::domain::user::{User, UserWithRoles};

/// Flash alert exposed to the React frontend.
#[derive(Clone, Debug, Serialize)]
pub struct FlashAlertDto {
    pub message: String,
    pub level: String,
}

impl FlashAlertDto {
    /// Construct a serialized flash alert.
    pub fn new(message: impl Into<String>, level: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: level.into(),
        }
    }
}

/// Minimal hub data exposed to the React frontend.
#[derive(Clone, Debug, Serialize)]
pub struct HubOptionDto {
    pub id: i32,
    pub name: String,
}

impl From<Hub> for HubOptionDto {
    fn from(hub: Hub) -> Self {
        Self {
            id: hub.id.get(),
            name: hub.name.into_inner(),
        }
    }
}

/// User data exposed to the React frontend.
#[derive(Clone, Debug, Serialize)]
pub struct CurrentUserDto {
    pub email: String,
    pub roles: Vec<String>,
}

impl From<&AuthenticatedUser> for CurrentUserDto {
    fn from(user: &AuthenticatedUser) -> Self {
        Self {
            email: user.email.clone(),
            roles: user.roles.clone(),
        }
    }
}

/// Hub data exposed to the React frontend for page shell navigation.
#[derive(Clone, Debug, Serialize)]
pub struct CurrentHubDto {
    pub name: String,
}

impl From<Hub> for CurrentHubDto {
    fn from(hub: Hub) -> Self {
        Self {
            name: hub.name.into_inner(),
        }
    }
}

/// Navigation link data exposed to the React frontend.
#[derive(Clone, Debug, Serialize)]
pub struct MenuItemDto {
    pub name: String,
    pub url: String,
}

impl From<Menu> for MenuItemDto {
    fn from(menu: Menu) -> Self {
        Self {
            name: menu.name.into_inner(),
            url: menu.url.into_inner(),
        }
    }
}

/// Shared shell bootstrap payload for React-backed pages.
#[derive(Clone, Debug, Serialize)]
pub struct SharedShellBootstrap {
    pub alerts: Vec<FlashAlertDto>,
}

/// Bootstrap payload shared by auth pages.
#[derive(Clone, Debug, Serialize)]
pub struct AuthPageBootstrap {
    pub shell: SharedShellBootstrap,
    pub next: Option<String>,
    pub hubs: Vec<HubOptionDto>,
}

/// Bootstrap payload for the React-backed sign-up page.
pub type SignupPageBootstrap = AuthPageBootstrap;

/// Bootstrap payload for the React-backed sign-in page.
pub type SigninPageBootstrap = AuthPageBootstrap;

/// Bootstrap payload for the React-backed basic dashboard.
#[derive(Clone, Debug, Serialize)]
pub struct BasicDashboardBootstrap {
    pub shell: SharedShellBootstrap,
    pub current_user: CurrentUserDto,
    pub current_hub: CurrentHubDto,
    pub current_page: String,
    pub menu: Vec<MenuItemDto>,
    pub user_name: Option<String>,
}

/// Role data exposed to the React admin dashboard.
#[derive(Clone, Debug, Serialize)]
pub struct AdminRoleDto {
    pub id: i32,
    pub name: String,
    pub can_delete: bool,
}

impl From<Role> for AdminRoleDto {
    fn from(role: Role) -> Self {
        let id = role.id.get();
        Self {
            id,
            name: role.name.into_inner(),
            can_delete: id != 1,
        }
    }
}

/// Hub data exposed to the React admin dashboard.
#[derive(Clone, Debug, Serialize)]
pub struct AdminHubDto {
    pub id: i32,
    pub name: String,
    pub can_delete: bool,
}

impl From<Hub> for AdminHubDto {
    fn from(hub: Hub) -> Self {
        let id = hub.id.get();
        Self {
            id,
            name: hub.name.into_inner(),
            can_delete: id != 1,
        }
    }
}

/// Menu item data exposed to the React admin management section.
#[derive(Clone, Debug, Serialize)]
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

/// User list row data exposed to the React admin dashboard.
#[derive(Clone, Debug, Serialize)]
pub struct AdminUserListItemDto {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl From<UserWithRoles> for AdminUserListItemDto {
    fn from(user_with_roles: UserWithRoles) -> Self {
        Self {
            id: user_with_roles.user.id.get(),
            name: user_with_roles
                .user
                .name
                .map(|name| name.into_inner())
                .unwrap_or_default(),
            email: user_with_roles.user.email.into_inner(),
            roles: user_with_roles
                .roles
                .into_iter()
                .map(|role| role.name.into_inner())
                .collect(),
        }
    }
}

/// Role option exposed in the React admin user modal.
#[derive(Clone, Debug, Serialize)]
pub struct RoleOptionDto {
    pub id: i32,
    pub name: String,
}

impl From<Role> for RoleOptionDto {
    fn from(role: Role) -> Self {
        Self {
            id: role.id.get(),
            name: role.name.into_inner(),
        }
    }
}

/// Editable user data exposed in the React admin user modal.
#[derive(Clone, Debug, Serialize)]
pub struct AdminEditableUserDto {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub roles: Vec<i32>,
}

impl From<User> for AdminEditableUserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.get(),
            email: user.email.into_inner(),
            name: user.name.map(|name| name.into_inner()).unwrap_or_default(),
            roles: user
                .roles
                .into_iter()
                .map(|role_id| role_id.get())
                .collect(),
        }
    }
}

/// Bootstrap payload for the React-backed admin dashboard.
#[derive(Clone, Debug, Serialize)]
pub struct AdminDashboardBootstrap {
    pub shell: SharedShellBootstrap,
    pub current_user: CurrentUserDto,
    pub current_hub: CurrentHubDto,
    pub current_page: String,
    pub menu: Vec<MenuItemDto>,
    pub roles: Vec<AdminRoleDto>,
    pub hubs: Vec<AdminHubDto>,
    pub admin_menu: Vec<AdminMenuItemDto>,
    pub users: Vec<AdminUserListItemDto>,
}

/// JSON payload used to populate the React admin user modal.
#[derive(Clone, Debug, Serialize)]
pub struct AdminUserModalBootstrap {
    pub user: Option<AdminEditableUserDto>,
    pub roles: Vec<RoleOptionDto>,
}
