use anyhow::Result;
use diesel::prelude::*;

use crate::db::DbConnection;
use crate::domain::user::{NewUser, User};
use crate::models::user::{NewUser as NewDbUser, User as DbUser};
use crate::repository::UserRepository;

pub struct DieselUserRepository<'a> {
    pub connection: &'a mut DbConnection,
}

impl<'a> DieselUserRepository<'a> {
    pub fn new(connection: &'a mut DbConnection) -> Self {
        Self { connection }
    }
}

impl<'a> UserRepository for DieselUserRepository<'a> {
    fn get_by_id(&mut self, id: i32) -> anyhow::Result<Option<User>> {
        use crate::schema::users;

        let result = users::table
            .filter(users::id.eq(id))
            .first::<DbUser>(self.connection)
            .optional()?;

        Ok(result.map(|db_user| db_user.into())) // Convert DbUser to DomainUser
    }

    fn get_by_email(&mut self, email: &str, hub_id: i32) -> Result<Option<User>> {
        use crate::schema::users;

        let result = users::table
            .filter(users::email.eq(email))
            .filter(users::hub_id.eq(hub_id))
            .first::<DbUser>(self.connection)
            .optional()?;

        Ok(result.map(|db_user| db_user.into())) // Convert DbUser to DomainUser
    }

    fn create(&mut self, new_user: &NewUser) -> Result<User> {
        use crate::schema::users;

        let new_db_user = NewDbUser::from(new_user); // Convert to DbNewUser
        diesel::insert_into(users::table)
            .values(&new_db_user)
            .get_result::<DbUser>(self.connection)
            .map(|db_user| db_user.into()) // Convert DbUser to DomainUser
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn list(&mut self) -> Result<Vec<User>> {
        use crate::schema::users;

        let results = users::table.load::<DbUser>(self.connection)?;

        Ok(results.into_iter().map(|db_user| db_user.into()).collect()) // Convert DbUser to DomainUser
    }
}
