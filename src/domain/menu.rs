//! Domain models describing menus attached to a hub.

use serde::{Deserialize, Serialize};

use crate::domain::types::{HubId, MenuId, MenuName, MenuUrl, TypeConstraintError};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// A navigation item available to users of a [`Hub`].
pub struct Menu {
    pub id: MenuId,
    pub name: MenuName,
    pub url: MenuUrl,
    pub hub_id: HubId,
}

impl Menu {
    /// Constructs a menu from validated domain types.
    pub fn new(id: MenuId, name: MenuName, url: MenuUrl, hub_id: HubId) -> Self {
        Self {
            id,
            name,
            url,
            hub_id,
        }
    }

    /// Validates raw values before constructing a menu.
    pub fn try_new(
        id: i32,
        name: impl Into<String>,
        url: impl Into<String>,
        hub_id: i32,
    ) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            MenuId::try_from(id)?,
            MenuName::try_from(name.into())?,
            MenuUrl::try_from(url.into())?,
            HubId::try_from(hub_id)?,
        ))
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Parameters required to create a new [`Menu`].
pub struct NewMenu {
    pub name: MenuName,
    pub url: MenuUrl,
    pub hub_id: HubId,
}

impl NewMenu {
    /// Constructs a new menu payload from validated domain types.
    pub fn new(name: MenuName, url: MenuUrl, hub_id: HubId) -> Self {
        Self { name, url, hub_id }
    }

    /// Validates raw values before constructing a new menu payload.
    pub fn try_new(
        name: impl Into<String>,
        url: impl Into<String>,
        hub_id: i32,
    ) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            MenuName::try_from(name.into())?,
            MenuUrl::try_from(url.into())?,
            HubId::try_from(hub_id)?,
        ))
    }
}
