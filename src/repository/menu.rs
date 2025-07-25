use diesel::prelude::*;
use pushkind_common::db::DbPool;

use crate::domain::menu::{Menu, NewMenu};
use crate::models::menu::{Menu as DbMenu, NewMenu as NewDbMenu};
use crate::repository::errors::{RepositoryError, RepositoryResult};
use crate::repository::{MenuReader, MenuRepository, MenuWriter};

/// Diesel implementation of [`MenuReader`] and [`MenuWriter`].
pub struct DieselMenuRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> DieselMenuRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

impl MenuWriter for DieselMenuRepository<'_> {
    fn create(&self, new_menu: &NewMenu) -> RepositoryResult<Menu> {
        use crate::schema::menu;

        let mut connection = self.pool.get()?;

        let new_db_menu = NewDbMenu::from(new_menu); // Convert to DbNewMenu
        let menu = diesel::insert_into(menu::table)
            .values(&new_db_menu)
            .get_result::<DbMenu>(&mut connection)
            .map(|db_menu| db_menu.into())?; // Convert DbMenu to DomainMenu
        Ok(menu)
    }

    fn delete(&self, menu_id: i32) -> RepositoryResult<usize> {
        use crate::schema::menu;

        let mut connection = self.pool.get()?;

        let result =
            diesel::delete(menu::table.filter(menu::id.eq(menu_id))).execute(&mut connection)?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(result)
    }
}

impl MenuReader for DieselMenuRepository<'_> {
    fn list(&self, hub_id: i32) -> RepositoryResult<Vec<Menu>> {
        use crate::schema::menu;

        let mut connection = self.pool.get()?;

        let results = menu::table
            .filter(menu::hub_id.eq(hub_id))
            .load::<DbMenu>(&mut connection)?;

        Ok(results.into_iter().map(|db_menu| db_menu.into()).collect()) // Convert DbMenu to DomainMenu
    }
}

impl MenuRepository for DieselMenuRepository<'_> {}
