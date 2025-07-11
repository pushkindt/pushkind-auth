use diesel::prelude::*;

use crate::domain::menu::{Menu as DomainMenu, NewMenu as DomainNewMenu};
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
            id: menu.id,
            name: menu.name,
            url: menu.url,
            hub_id: menu.hub_id,
        }
    }
}

impl<'a> From<&DomainNewMenu<'a>> for NewMenu<'a> {
    fn from(menu: &DomainNewMenu<'a>) -> Self {
        Self {
            name: menu.name,
            url: menu.url,
            hub_id: menu.hub_id,
        }
    }
}

impl<'a> From<DomainNewMenu<'a>> for NewMenu<'a> {
    fn from(menu: DomainNewMenu<'a>) -> Self {
        Self::from(&menu)
    }
}

impl From<Menu> for DomainMenu {
    fn from(menu: Menu) -> Self {
        DomainMenu {
            id: menu.id,
            name: menu.name,
            url: menu.url,
            hub_id: menu.hub_id,
        }
    }
}
