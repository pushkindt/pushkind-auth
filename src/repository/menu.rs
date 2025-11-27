//! Diesel-backed repository operations for menus.

use diesel::prelude::*;
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::menu::{Menu, NewMenu};
use crate::domain::types::{HubId, MenuId};
use crate::models::menu::{Menu as DbMenu, NewMenu as NewDbMenu};
use crate::repository::{DieselRepository, MenuReader, MenuRepository, MenuWriter, map_type_error};

impl MenuWriter for DieselRepository {
    fn create_menu(&self, new_menu: &NewMenu) -> RepositoryResult<Menu> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let new_db_menu = NewDbMenu::from(new_menu); // Convert to DbNewMenu
        let menu = diesel::insert_into(menu::table)
            .values(&new_db_menu)
            .get_result::<DbMenu>(&mut connection)
            .map_err(Into::into)
            .and_then(|db_menu| TryInto::try_into(db_menu).map_err(map_type_error))?; // Convert DbMenu to DomainMenu
        Ok(menu)
    }

    fn delete_menu(&self, menu_id: MenuId) -> RepositoryResult<usize> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let result = diesel::delete(menu::table.filter(menu::id.eq(menu_id.get())))
            .execute(&mut connection)?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(result)
    }
}

impl MenuReader for DieselRepository {
    fn get_menu_by_id(&self, menu_id: MenuId, hub_id: HubId) -> RepositoryResult<Option<Menu>> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let result = menu::table
            .filter(menu::id.eq(menu_id.get()))
            .filter(menu::hub_id.eq(hub_id.get()))
            .first::<DbMenu>(&mut connection)
            .optional()?;
        result
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_type_error)
    }

    fn list_menu(&self, hub_id: HubId) -> RepositoryResult<Vec<Menu>> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let results = menu::table
            .filter(menu::hub_id.eq(hub_id.get()))
            .load::<DbMenu>(&mut connection)?;

        results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_type_error)
    }
}

impl MenuRepository for DieselRepository {}
