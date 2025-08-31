use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// A business entity representing a hub which groups users and menus.
pub struct Hub {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize)]
/// Data used for creating a new [`Hub`].
pub struct NewHub {
    pub name: String,
}
