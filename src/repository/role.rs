use diesel::prelude::*;

use crate::db::DbPool;
use crate::domain::role::{NewRole, Role};
use crate::models::role::{NewRole as NewDbRole, Role as DbRole};
use crate::repository::errors::RepositoryResult;
use crate::repository::{RoleReader, RoleWriter};

/// Diesel implementation of [`RoleReader`] and [`RoleWriter`].
pub struct DieselRoleRepository<'a> {
    pool: &'a DbPool,
}

impl<'a> DieselRoleRepository<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }
}

impl RoleReader for DieselRoleRepository<'_> {
    fn get_by_id(&self, id: i32) -> RepositoryResult<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self.pool.get()?;

        let result = roles::table
            .filter(roles::id.eq(id))
            .first::<DbRole>(&mut connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn get_by_name(&self, name: &str) -> RepositoryResult<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self.pool.get()?;

        let result = roles::table
            .filter(roles::name.eq(name))
            .first::<DbRole>(&mut connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn list(&self) -> RepositoryResult<Vec<Role>> {
        use crate::schema::roles;

        let mut connection = self.pool.get()?;

        let results = roles::table.load::<DbRole>(&mut connection)?;

        Ok(results.into_iter().map(|db_role| db_role.into()).collect()) // Convert DbRole to DomainRole
    }
}

impl RoleWriter for DieselRoleRepository<'_> {
    fn create(&self, new_role: &NewRole) -> RepositoryResult<Role> {
        use crate::schema::roles;

        let mut connection = self.pool.get()?;

        let new_db_role = NewDbRole::from(new_role); // Convert to DbNewRole
        let role = diesel::insert_into(roles::table)
            .values(&new_db_role)
            .get_result::<DbRole>(&mut connection)
            .map(|db_role| db_role.into())?; // Convert DbRole to DomainRole
        Ok(role)
    }

    fn delete(&self, role_id: i32) -> RepositoryResult<usize> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut connection = self.pool.get()?;

        diesel::delete(user_roles::table.filter(user_roles::role_id.eq(role_id)))
            .execute(&mut connection)?;

        let result =
            diesel::delete(roles::table.filter(roles::id.eq(role_id))).execute(&mut connection)?;

        Ok(result)
    }
}
