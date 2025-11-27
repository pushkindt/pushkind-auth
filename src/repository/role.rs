//! Diesel-backed repository operations for roles.

use diesel::prelude::*;
use pushkind_common::repository::errors::{RepositoryError, RepositoryResult};

use crate::domain::role::{NewRole, Role};
use crate::domain::types::RoleId;
use crate::models::role::{NewRole as NewDbRole, Role as DbRole};
use crate::repository::{DieselRepository, RoleReader, RoleWriter, map_type_error};

impl RoleReader for DieselRepository {
    fn get_role_by_id(&self, id: RoleId) -> RepositoryResult<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self.conn()?;

        let result = roles::table
            .filter(roles::id.eq(id.get()))
            .first::<DbRole>(&mut connection)
            .optional()?;

        result
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_type_error)
    }

    fn get_role_by_name(&self, name: &str) -> RepositoryResult<Option<Role>> {
        use crate::schema::roles;

        let mut connection = self.conn()?;

        let result = roles::table
            .filter(roles::name.eq(name))
            .first::<DbRole>(&mut connection)
            .optional()?;

        result
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_type_error)
    }

    fn list_roles(&self) -> RepositoryResult<Vec<Role>> {
        use crate::schema::roles;

        let mut connection = self.conn()?;

        let results = roles::table.load::<DbRole>(&mut connection)?;

        results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_type_error)
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
            .map_err(Into::into)
            .and_then(|db_role| TryInto::try_into(db_role).map_err(map_type_error))?; // Convert DbRole to DomainRole
        Ok(role)
    }

    fn delete_role(&self, role_id: RoleId) -> RepositoryResult<usize> {
        use crate::schema::roles;
        use crate::schema::user_roles;

        let mut connection = self.conn()?;

        let result = connection.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(user_roles::table.filter(user_roles::role_id.eq(role_id.get())))
                .execute(conn)?;

            diesel::delete(roles::table.filter(roles::id.eq(role_id.get()))).execute(conn)
        })?;

        if result == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(result)
    }
}
