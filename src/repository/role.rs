use diesel::prelude::*;
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::role::{NewRole, Role};
use crate::models::role::{NewRole as NewDbRole, Role as DbRole};
use crate::repository::{DieselRepository, RoleReader, RoleWriter};

impl RoleReader for DieselRepository {
    fn get_role_by_id(&self, id: i32) -> RepositoryResult<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self.conn()?;

        let result = roles::table
            .filter(roles::id.eq(id))
            .first::<DbRole>(&mut connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn get_role_by_name(&self, name: &str) -> RepositoryResult<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self.conn()?;

        let result = roles::table
            .filter(roles::name.eq(name))
            .first::<DbRole>(&mut connection)
            .optional()?;

        Ok(result.map(|db_role| db_role.into())) // Convert DbRole to DomainRole
    }

    fn list_roles(&self) -> RepositoryResult<Vec<Role>> {
        use crate::schema::roles;

        let mut connection = self.conn()?;

        let results = roles::table.load::<DbRole>(&mut connection)?;

        Ok(results.into_iter().map(|db_role| db_role.into()).collect()) // Convert DbRole to DomainRole
    }
}

impl RoleWriter for DieselRepository {
    fn create_role(&self, new_role: &NewRole) -> RepositoryResult<Role> {
        use crate::schema::roles;

        let mut connection = self.conn()?;

        let new_db_role = NewDbRole::from(new_role); // Convert to DbNewRole
        let role = diesel::insert_into(roles::table)
            .values(&new_db_role)
            .get_result::<DbRole>(&mut connection)
            .map(|db_role| db_role.into())?; // Convert DbRole to DomainRole
        Ok(role)
    }

    fn delete_role(&self, role_id: i32) -> RepositoryResult<usize> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut connection = self.conn()?;

        let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(user_roles::table.filter(user_roles::role_id.eq(role_id)))
                .execute(conn)?;

            diesel::delete(roles::table.filter(roles::id.eq(role_id))).execute(conn)
        })?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(result)
    }
}
