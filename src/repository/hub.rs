use diesel::prelude::*;

use crate::db::DbConnection;
use crate::domain::hub::{Hub, NewHub};
use crate::models::hub::{Hub as DbHub, NewHub as NewDbHub};
use crate::repository::HubRepository;

pub struct DieselHubRepository<'a> {
    pub connection: &'a mut DbConnection,
}

impl<'a> DieselHubRepository<'a> {
    pub fn new(connection: &'a mut DbConnection) -> Self {
        Self { connection }
    }
}

impl HubRepository for DieselHubRepository<'_> {
    fn get_by_id(&mut self, id: i32) -> anyhow::Result<Option<Hub>> {
        use crate::schema::hubs;

        let result = hubs::table
            .filter(hubs::id.eq(id))
            .first::<DbHub>(self.connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn get_by_name(&mut self, name: &str) -> anyhow::Result<Option<Hub>> {
        use crate::schema::hubs;

        let result = hubs::table
            .filter(hubs::name.eq(name))
            .first::<DbHub>(self.connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn create(&mut self, new_hub: &NewHub) -> anyhow::Result<Hub> {
        use crate::schema::hubs;

        let new_db_hub = NewDbHub::from(new_hub); // Convert to DbNewHub
        diesel::insert_into(hubs::table)
            .values(&new_db_hub)
            .get_result::<DbHub>(self.connection)
            .map(|db_hub| db_hub.into()) // Convert DbHub to DomainHub
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn list(&mut self) -> anyhow::Result<Vec<Hub>> {
        use crate::schema::hubs;

        let results = hubs::table.load::<DbHub>(self.connection)?;

        Ok(results.into_iter().map(|db_hub| db_hub.into()).collect()) // Convert DbHub to DomainHub
    }

    fn delete(&mut self, hub_id: i32) -> anyhow::Result<usize> {
        use crate::schema::hubs;
        use crate::schema::user_roles;
        use crate::schema::users;

        let hub_users = users::table
            .filter(users::hub_id.eq(hub_id))
            .select(users::id)
            .load::<i32>(self.connection)?;

        // delete user_roles for hub users
        diesel::delete(user_roles::table.filter(user_roles::user_id.eq_any(&hub_users)))
            .execute(self.connection)?;

        //delete users for hub
        diesel::delete(users::table.filter(users::hub_id.eq(hub_id))).execute(self.connection)?;

        //delete hub
        diesel::delete(hubs::table.filter(hubs::id.eq(hub_id)))
            .execute(self.connection)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
