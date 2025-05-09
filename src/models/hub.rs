use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::{hub::Hub as DomainHub, hub::NewHub as DomainNewHub};

#[derive(Debug, Clone, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::hubs)]
pub struct Hub {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::hubs)]
pub struct NewHub<'a> {
    pub name: &'a str,
}

impl From<Hub> for DomainHub {
    fn from(db: Hub) -> Self {
        Self {
            id: db.id,
            name: db.name,
            created_at: db.created_at,
            updated_at: db.updated_at,
        }
    }
}

impl<'a> From<&'a DomainNewHub> for NewHub<'a> {
    fn from(domain: &'a DomainNewHub) -> Self {
        Self { name: &domain.name }
    }
}
