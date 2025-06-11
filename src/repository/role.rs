use anyhow::Context;
use diesel::prelude::*;

use crate::db::DbPool;
use crate::domain::role::{NewRole, Role};
use crate::models::role::{NewRole as NewDbRole, Role as DbRole};
use crate::repository::RoleRepository;

pub struct DieselRoleRepository<'a> {
    pub pool: &'a DbPool,
}

impl<'a> DieselRoleRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

impl RoleRepository for DieselRoleRepository<'_> {
    fn get_by_id(&self, id: i32) -> anyhow::Result<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let result = roles::table
            .filter(roles::id.eq(id))
            .first::<DbRole>(&mut connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn get_by_name(&self, name: &str) -> anyhow::Result<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let result = roles::table
            .filter(roles::name.eq(name))
            .first::<DbRole>(&mut connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn create(&self, new_role: &NewRole) -> anyhow::Result<Role> {
        use crate::schema::roles;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let new_db_role = NewDbRole::from(new_role); // Convert to DbNewRole
        diesel::insert_into(roles::table)
            .values(&new_db_role)
            .get_result::<DbRole>(&mut connection)
            .map(|db_role| db_role.into()) // Convert DbRole to DomainRole
            .map_err(|e| anyhow::anyhow!(e))
    }

    fn list(&self) -> anyhow::Result<Vec<Role>> {
        use crate::schema::roles;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        let results = roles::table.load::<DbRole>(&mut connection)?;

        Ok(results.into_iter().map(|db_role| db_role.into()).collect()) // Convert DbRole to DomainRole
    }

    fn delete(&self, role_id: i32) -> anyhow::Result<usize> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut connection = self
            .pool
            .get()
            .context("couldn't get db connection from pool")?;

        diesel::delete(user_roles::table.filter(user_roles::role_id.eq(role_id)))
            .execute(&mut connection)
            .map_err(|e| anyhow::anyhow!(e))?;

        diesel::delete(roles::table.filter(roles::id.eq(role_id)))
            .execute(&mut connection)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
