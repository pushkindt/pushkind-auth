use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::types::{HubId, HubName};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// A business entity representing a hub which groups users and menus.
pub struct Hub {
    pub id: HubId,
    pub name: HubName,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize)]
/// Data used for creating a new [`Hub`].
pub struct NewHub {
    pub name: HubName,
}
