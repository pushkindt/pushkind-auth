//! DTOs feeding the main index template.

use crate::domain::hub::Hub;
use crate::domain::menu::Menu;
use crate::domain::role::Role;
use crate::domain::user::UserWithRoles;

/// Aggregated information required to render the index page.
///
/// The struct bundles data about the current hub, available users, roles,
/// hubs and menu entries, as well as the name of the current user if
/// available.
pub struct IndexData {
    pub hub: Hub,
    pub users: Vec<UserWithRoles>,
    pub roles: Vec<Role>,
    pub hubs: Vec<Hub>,
    pub menu: Vec<Menu>,
    pub user_name: Option<String>,
}
