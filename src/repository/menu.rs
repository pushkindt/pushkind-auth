use diesel::prelude::*;
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::menu::{Menu, NewMenu};
use crate::models::menu::{Menu as DbMenu, NewMenu as NewDbMenu};
use crate::repository::{DieselRepository, MenuReader, MenuRepository, MenuWriter};

impl MenuWriter for DieselRepository {
    fn create_menu(&self, new_menu: &NewMenu) -> RepositoryResult<Menu> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let new_db_menu = NewDbMenu::from(new_menu); // Convert to DbNewMenu
        let menu = diesel::insert_into(menu::table)
            .values(&new_db_menu)
            .get_result::<DbMenu>(&mut connection)
            .map(|db_menu| db_menu.into())?; // Convert DbMenu to DomainMenu
        Ok(menu)
    }

    fn delete_menu(&self, menu_id: i32) -> RepositoryResult<usize> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let result =
            diesel::delete(menu::table.filter(menu::id.eq(menu_id))).execute(&mut connection)?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(result)
    }
}

impl MenuReader for DieselRepository {
    fn get_menu_by_id(&self, menu_id: i32, hub_id: i32) -> RepositoryResult<Option<Menu>> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let result = menu::table
            .filter(menu::id.eq(menu_id))
            .filter(menu::hub_id.eq(hub_id))
            .first::<DbMenu>(&mut connection)
            .optional()?;
        Ok(result.map(|db_menu| db_menu.into())) // Convert DbMenu to DomainMenu
    }

    fn list_menu(&self, hub_id: i32) -> RepositoryResult<Vec<Menu>> {
        use crate::schema::menu;

        let mut connection = self.conn()?;

        let results = menu::table
            .filter(menu::hub_id.eq(hub_id))
            .load::<DbMenu>(&mut connection)?;

        Ok(results.into_iter().map(|db_menu| db_menu.into()).collect()) // Convert DbMenu to DomainMenu
    }
}

impl MenuRepository for DieselRepository {}
