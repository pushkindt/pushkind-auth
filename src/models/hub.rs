//! Diesel models and conversions for hubs.

use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::types::TypeConstraintError;
use crate::domain::{hub::Hub as DomainHub, hub::NewHub as DomainNewHub};

#[derive(Debug, Clone, Identifiable, Queryable)]
#[diesel(table_name = crate::schema::hubs)]
/// Database representation of a [`crate::domain::hub::Hub`].
pub struct Hub {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::hubs)]
/// Insertable form of [`Hub`].
pub struct NewHub<'a> {
    pub name: &'a str,
}

impl TryFrom<Hub> for DomainHub {
    type Error = TypeConstraintError;

    fn try_from(db: Hub) -> Result<Self, Self::Error> {
        DomainHub::try_new(db.id, db.name, db.created_at, db.updated_at)
    }
}

impl<'a> From<&'a DomainNewHub> for NewHub<'a> {
    fn from(domain: &'a DomainNewHub) -> Self {
        Self {
            name: domain.name.as_str(),
        }
    }
}
