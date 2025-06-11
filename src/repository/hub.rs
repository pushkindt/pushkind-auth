use anyhow::Context;
use diesel::prelude::*;

use crate::db::DbPool;
use crate::domain::hub::{Hub, NewHub};
use crate::models::hub::{Hub as DbHub, NewHub as NewDbHub};
use crate::repository::HubRepository;

pub struct DieselHubRepository<'a> {
    pub pool: &'a DbPool,
}

impl<'a> DieselHubRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

impl HubRepository for DieselHubRepository<'_> {
    fn get_by_id(&self, id: i32) -> anyhow::Result<Option<Hub>> {
        use crate::schema::hubs;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let result = hubs::table
            .filter(hubs::id.eq(id))
            .first::<DbHub>(&mut connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn get_by_name(&self, name: &str) -> anyhow::Result<Option<Hub>> {
        use crate::schema::hubs;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let result = hubs::table
            .filter(hubs::name.eq(name))
            .first::<DbHub>(&mut connection)
            .optional()?;

        Ok(result.map(|db_hub| db_hub.into())) // Convert DbHub to DomainHub
    }

    fn create(&self, new_hub: &NewHub) -> anyhow::Result<Hub> {
        use crate::schema::hubs;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let new_db_hub = NewDbHub::from(new_hub); // Convert to DbNewHub
        diesel::insert_into(hubs::table)
            .values(&new_db_hub)
            .get_result::<DbHub>(&mut connection)
            .map(|db_hub| db_hub.into()) // Convert DbHub to DomainHub
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn list(&self) -> anyhow::Result<Vec<Hub>> {
        use crate::schema::hubs;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let results = hubs::table.load::<DbHub>(&mut connection)?;

        Ok(results.into_iter().map(|db_hub| db_hub.into()).collect()) // Convert DbHub to DomainHub
    }

    fn delete(&self, hub_id: i32) -> anyhow::Result<usize> {
        use crate::schema::hubs;
        use crate::schema::user_roles;
        use crate::schema::users;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let hub_users = users::table
            .filter(users::hub_id.eq(hub_id))
            .select(users::id)
            .load::<i32>(&mut connection)?;

        // delete user_roles for hub users
        diesel::delete(user_roles::table.filter(user_roles::user_id.eq_any(&hub_users)))
            .execute(&mut connection)?;

        //delete users for hub
        diesel::delete(users::table.filter(users::hub_id.eq(hub_id))).execute(&mut connection)?;

        //delete hub
        diesel::delete(hubs::table.filter(hubs::id.eq(hub_id)))
            .execute(&mut connection)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
