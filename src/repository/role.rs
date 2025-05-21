use diesel::prelude::*;

use crate::db::DbConnection;
use crate::domain::role::{NewRole, Role};
use crate::models::role::{NewRole as NewDbRole, Role as DbRole};
use crate::repository::RoleRepository;

pub struct DieselRoleRepository<'a> {
    pub connection: &'a mut DbConnection,
}

impl<'a> DieselRoleRepository<'a> {
    pub fn new(connection: &'a mut DbConnection) -> Self {
        Self { connection }
    }
}

impl<'a> RoleRepository for DieselRoleRepository<'a> {
    fn get_by_id(&mut self, id: i32) -> anyhow::Result<Option<Role>> {
        use crate::schema::roles;

        let result = roles::table
            .filter(roles::id.eq(id))
            .first::<DbRole>(self.connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn get_by_name(&mut self, name: &str) -> anyhow::Result<Option<Role>> {
        use crate::schema::roles;

        let result = roles::table
            .filter(roles::name.eq(name))
            .first::<DbRole>(self.connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn create(&mut self, new_role: &NewRole) -> anyhow::Result<Role> {
        use crate::schema::roles;

        let new_db_role = NewDbRole::from(new_role); // Convert to DbNewRole
        diesel::insert_into(roles::table)
            .values(&new_db_role)
            .get_result::<DbRole>(self.connection)
            .map(|db_role| db_role.into()) // Convert DbRole to DomainRole
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn list(&mut self) -> anyhow::Result<Vec<Role>> {
        use crate::schema::roles;

        let results = roles::table.load::<DbRole>(self.connection)?;

        Ok(results.into_iter().map(|db_role| db_role.into()).collect()) // Convert DbRole to DomainRole
    }
}
