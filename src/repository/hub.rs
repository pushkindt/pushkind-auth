use anyhow::Result;
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

impl<'a> HubRepository for DieselHubRepository<'a> {
    fn get_by_id(&mut self, id: i32) -> Result<Option<Hub>> {
        use crate::schema::hubs;

        let result = hubs::table
            .filter(hubs::id.eq(id))
            .first::<DbHub>(self.connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn get_by_name(&mut self, name: &str) -> Result<Option<Hub>> {
        use crate::schema::hubs;

        let result = hubs::table
            .filter(hubs::name.eq(name))
            .first::<DbHub>(self.connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn create(&mut self, new_hub: &NewHub) -> Result<Hub> {
        use crate::schema::hubs;

        let new_db_hub = NewDbHub::from(new_hub); // Convert to DbNewHub
        diesel::insert_into(hubs::table)
            .values(&new_db_hub)
            .get_result::<DbHub>(self.connection)
            .map(|db_hub| db_hub.into()) // Convert DbHub to DomainHub
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn list(&mut self) -> Result<Vec<Hub>> {
        use crate::schema::hubs;

        let results = hubs::table.load::<DbHub>(self.connection)?;

        Ok(results.into_iter().map(|db_hub| db_hub.into()).collect()) // Convert DbHub to DomainHub
    }
}
