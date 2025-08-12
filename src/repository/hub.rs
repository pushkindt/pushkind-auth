use diesel::prelude::*;
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::hub::{Hub, NewHub};
use crate::models::hub::{Hub as DbHub, NewHub as NewDbHub};
use crate::repository::{DieselRepository, HubReader, HubWriter};

impl HubReader for DieselRepository {
    fn get_hub_by_id(&self, id: i32) -> RepositoryResult<Option<Hub>> {
        use crate::schema::hubs;

        let mut connection = self.conn()?;

        let result = hubs::table
            .filter(hubs::id.eq(id))
            .first::<DbHub>(&mut connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn get_hub_by_name(&self, name: &str) -> RepositoryResult<Option<Hub>> {
        use crate::schema::hubs;

        let mut connection = self.conn()?;

        let result = hubs::table
            .filter(hubs::name.eq(name))
            .first::<DbHub>(&mut connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn list_hubs(&self) -> RepositoryResult<Vec<Hub>> {
        use crate::schema::hubs;

        let mut connection = self.conn()?;

        let results = hubs::table.load::<DbHub>(&mut connection)?;

        Ok(results.into_iter().map(|db_hub| db_hub.into()).collect()) // Convert DbHub to DomainHub
    }
}

impl HubWriter for DieselRepository {
    fn create_hub(&self, new_hub: &NewHub) -> RepositoryResult<Hub> {
        use crate::schema::hubs;

        let mut connection = self.conn()?;

        let new_db_hub = NewDbHub::from(new_hub); // Convert to DbNewHub
        let hub = diesel::insert_into(hubs::table)
            .values(&new_db_hub)
            .get_result::<DbHub>(&mut connection)
            .map(|db_hub| db_hub.into())?; // Convert DbHub to DomainHub
        Ok(hub)
    }

    fn delete_hub(&self, hub_id: i32) -> RepositoryResult<usize> {
        use crate::schema::hubs;
        use crate::schema::menu;
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut connection = self.conn()?;

        let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
            // delete menus for hub
            diesel::delete(menu::table.filter(menu::hub_id.eq(hub_id))).execute(conn)?;

            let hub_users = users::table
                .filter(users::hub_id.eq(hub_id))
                .select(users::id)
                .load::<i32>(conn)?;

            // delete user_roles for hub users
            diesel::delete(user_roles::table.filter(user_roles::user_id.eq_any(&hub_users)))
                .execute(conn)?;

            //delete users for hub
            diesel::delete(users::table.filter(users::hub_id.eq(hub_id))).execute(conn)?;

            //delete hub
            diesel::delete(hubs::table.filter(hubs::id.eq(hub_id))).execute(conn)
        })?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(result)
    }
}
