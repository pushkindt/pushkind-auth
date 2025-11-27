use diesel::prelude::*;

use crate::domain::menu::{Menu as DomainMenu, NewMenu as DomainNewMenu};
use crate::domain::types::{HubId, MenuId, MenuName, MenuUrl, TypeConstraintError};
use crate::models::hub::Hub;

#[derive(Debug, Clone, Identifiable, Associations, Queryable)]
#[diesel(belongs_to(Hub, foreign_key=hub_id))]
#[diesel(table_name = crate::schema::menu)]
/// Database model for [`crate::domain::menu::Menu`].
pub struct Menu {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub hub_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::menu)]
/// Insertable variant of [`Menu`].
pub struct NewMenu<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub hub_id: i32,
}

impl From<DomainMenu> for Menu {
    fn from(menu: DomainMenu) -> Self {
        Menu {
            id: menu.id.get(),
            name: menu.name.into_inner(),
            url: menu.url.into_inner(),
            hub_id: menu.hub_id.get(),
        }
    }
}

impl<'a> From<&'a DomainNewMenu> for NewMenu<'a> {
    fn from(menu: &'a DomainNewMenu) -> Self {
        Self {
            name: menu.name.as_str(),
            url: menu.url.as_str(),
            hub_id: menu.hub_id.get(),
        }
    }
}

impl TryFrom<Menu> for DomainMenu {
    type Error = TypeConstraintError;

    fn try_from(menu: Menu) -> Result<Self, Self::Error> {
        Ok(DomainMenu {
            id: MenuId::try_from(menu.id)?,
            name: MenuName::try_from(menu.name)?,
            url: MenuUrl::try_from(menu.url)?,
            hub_id: HubId::try_from(menu.hub_id)?,
        })
    }
}
