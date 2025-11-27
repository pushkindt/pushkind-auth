use diesel::prelude::*;
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::hub::{Hub, NewHub};
use crate::domain::types::HubId;
use crate::models::hub::{Hub as DbHub, NewHub as NewDbHub};
use crate::repository::{DieselRepository, HubReader, HubWriter, map_type_error};

impl HubReader for DieselRepository {
    fn get_hub_by_id(&self, id: HubId) -> RepositoryResult<Option<Hub>> {
        use crate::schema::hubs;

        let mut connection = self.conn()?;

        let result = hubs::table
            .filter(hubs::id.eq(id.get()))
            .first::<DbHub>(&mut connection)
            .optional()?;

        result
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_type_error)
    }

    fn get_hub_by_name(&self, name: &str) -> RepositoryResult<Option<Hub>> {
        use crate::schema::hubs;

        let mut connection = self.conn()?;

        let result = hubs::table
            .filter(hubs::name.eq(name))
            .first::<DbHub>(&mut connection)
            .optional()?;

        result
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_type_error)
    }

    fn list_hubs(&self) -> RepositoryResult<Vec<Hub>> {
        use crate::schema::hubs;

        let mut connection = self.conn()?;

        let results = hubs::table.load::<DbHub>(&mut connection)?;

        results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_type_error)
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
            .map_err(Into::into)
            .and_then(|db_hub| TryInto::try_into(db_hub).map_err(map_type_error))?; // Convert DbHub to DomainHub
        Ok(hub)
    }

    fn delete_hub(&self, hub_id: HubId) -> RepositoryResult<usize> {
        use crate::schema::hubs;
        use crate::schema::menu;
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut connection = self.conn()?;

        let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
            // delete menus for hub
            diesel::delete(menu::table.filter(menu::hub_id.eq(hub_id.get()))).execute(conn)?;

            let hub_users = users::table
                .filter(users::hub_id.eq(hub_id.get()))
                .select(users::id)
                .load::<i32>(conn)?;

            // delete user_roles for hub users
            diesel::delete(user_roles::table.filter(user_roles::user_id.eq_any(&hub_users)))
                .execute(conn)?;

            //delete users for hub
            diesel::delete(users::table.filter(users::hub_id.eq(hub_id.get()))).execute(conn)?;

            //delete hub
            diesel::delete(hubs::table.filter(hubs::id.eq(hub_id.get()))).execute(conn)
        })?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(result)
    }
}
