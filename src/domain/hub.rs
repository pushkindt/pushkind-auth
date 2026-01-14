//! Domain models representing hubs and their creation input.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::types::{HubId, HubName, TypeConstraintError};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// A business entity representing a hub which groups users and menus.
pub struct Hub {
    pub id: HubId,
    pub name: HubName,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Hub {
    /// Constructs a hub from validated domain types.
    pub fn new(
        id: HubId,
        name: HubName,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            name,
            created_at,
            updated_at,
        }
    }

    /// Validates raw values before constructing a hub.
    pub fn try_new(
        id: i32,
        name: impl Into<String>,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    ) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(
            HubId::try_from(id)?,
            HubName::try_from(name.into())?,
            created_at,
            updated_at,
        ))
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Data used for creating a new [`Hub`].
pub struct NewHub {
    pub name: HubName,
}

impl NewHub {
    /// Constructs a new hub payload from validated domain types.
    pub fn new(name: HubName) -> Self {
        Self { name }
    }

    /// Validates raw values before constructing a new hub payload.
    pub fn try_new(name: impl Into<String>) -> Result<Self, TypeConstraintError> {
        Ok(Self::new(HubName::try_from(name.into())?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hub_try_new_validates_inputs() {
        let ts = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0)
            .unwrap()
            .naive_utc();
        let hub = Hub::try_new(1, "  Hub ", ts, ts).unwrap();
        assert_eq!(hub.id.get(), 1);
        assert_eq!(hub.name.as_str(), "Hub");
    }

    #[test]
    fn hub_try_new_rejects_invalid_id() {
        let ts = chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0)
            .unwrap()
            .naive_utc();
        assert_eq!(
            Hub::try_new(0, "Hub", ts, ts).unwrap_err(),
            TypeConstraintError::NonPositiveId
        );
    }

    #[test]
    fn new_hub_try_new_rejects_empty_name() {
        assert_eq!(
            NewHub::try_new("   ").unwrap_err(),
            TypeConstraintError::EmptyString
        );
    }
}
