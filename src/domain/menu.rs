//! Domain models describing menus attached to a hub.

use serde::{Deserialize, Serialize};

use crate::domain::types::{HubId, MenuId, MenuName, MenuUrl};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// A navigation item available to users of a [`Hub`].
pub struct Menu {
    pub id: MenuId,
    pub name: MenuName,
    pub url: MenuUrl,
    pub hub_id: HubId,
}

#[derive(Clone, Debug, Deserialize)]
/// Parameters required to create a new [`Menu`].
pub struct NewMenu {
    pub name: MenuName,
    pub url: MenuUrl,
    pub hub_id: HubId,
}
